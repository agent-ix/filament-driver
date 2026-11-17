// SPDX-License-Identifier: AGPL-3.0-or-later
//! Parser and command handlers for the user-facing `fsl` binary.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use filament_driver::{
    build_linked_providers, drive, execute, CompileInput, Driven, ExecutionEngine,
    ExecutionFailure, ExecutionRequest, FslCg, GenerationInput, LimitOverrides, ObjectEnvironment,
    ProcessProvider, ProcessProviderLimits, ProcessProviderRequest, ProcessProviderSession,
    ProviderRequest, ProviderResults,
};
use filament_exact::Cancel;
use filament_semantic_value::catalog::Category;
use fsl_command::command::{self, PreparedRun, RunLimits};
use fsl_cst::format::{format_with_limits, FormatLimits, FormatRefusal};
use fsl_foundation::diagnostic::Code;
use fsl_foundation::diagnostic::Staged;
use fsl_foundation::source::{SourceReadCause, SourceReadError, SourceReadRefusal};
use fsl_foundation::{SettingLimits, SettingName, SourceIdentity};
use fsl_replay::spine::{parse, Call, CompileRefusal, DependencyInput, ParseRequest, SpineLimits};
use fsl_replay::{run_with_requirements, CallOutcome, RequirementsRun, RunRefusal};
use fsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument, OutcomeStage};
use ix_cli_kit::exit::{caller_exit_code, RESERVED_COMMAND_NOT_FOUND};
use ix_cli_kit::streams::{self, Diagnostics};

/// Pass FSL's category result through the kit without converting it to an `Outcome`.
#[must_use]
fn process_exit_code(category: Category) -> ExitCode {
    let result = category_exit_result(category);
    match caller_exit_code(result) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            let fallback = Category::InternalFailure.exit_code();
            emit_diagnostic(
                "ExitMappingFailure",
                &format!(
                    "driver exit mapping rejected result {} (reserved code {}); using fallback code {fallback}.",
                    error.rejected_code, error.reserved_code
                ),
            );
            ExitCode::from(fallback)
        }
    }
}

#[cfg(all(feature = "tc018-test-adapter", not(debug_assertions)))]
compile_error!("the TC-018 test adapter is unavailable in release builds");

/// Return FSL's exit result, with a test-only seam for the reserved-code case.
fn category_exit_result(category: Category) -> u8 {
    #[cfg(all(feature = "tc018-test-adapter", debug_assertions))]
    if std::env::var_os("TC018_CASE").as_deref() == Some(std::ffi::OsStr::new("reserved-exit")) {
        return RESERVED_COMMAND_NOT_FOUND;
    }

    category.exit_code()
}

/// Write a diagnostic to a caller-owned sink without panicking on a closed
/// stderr pipe.
fn write_diagnostic<W: Write>(sink: &mut W, kind: &str, message: &str) -> io::Result<()> {
    streams::Record::error(kind, message).write(Diagnostics::default(), sink)
}

/// Best-effort stderr output; a closed diagnostic pipe must not replace the
/// command's already selected FSL exit result with a panic.
fn emit_diagnostic(kind: &str, message: &str) {
    let mut stderr = io::stderr().lock();
    let _ = write_diagnostic(&mut stderr, kind, message);
}

/// Write command output with one policy for closed pipes and output failures.
fn write_primary_output(bytes: &[u8], description: &str, success: Category) -> Category {
    match streams::write_primary_stdout(bytes) {
        Ok(()) => success,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => success,
        Err(error) => {
            emit_diagnostic(
                "OutputFailure",
                &format!("OutputFailure: could not write {description}: {error}"),
            );
            Category::InternalFailure
        }
    }
}

fn version_exit_code() -> ExitCode {
    if category_exit_result(Category::Success) == RESERVED_COMMAND_NOT_FOUND {
        return process_exit_code(Category::Success);
    }

    let version = concat!(env!("CARGO_PKG_VERSION"), "\n");
    process_exit_code(write_primary_output(
        version.as_bytes(),
        "package version",
        Category::Success,
    ))
}

fn help_exit_code() -> ExitCode {
    if category_exit_result(Category::Success) == RESERVED_COMMAND_NOT_FOUND {
        return process_exit_code(Category::Success);
    }

    let mut help = String::from(concat!(
        "Usage: fsl <command> [arguments] [--format text|json|dot]\n",
        "       fsl --help|-h|help\n",
        "       fsl --version|-V\n\n",
        "Commands:\n",
    ));
    for verb in Verb::ALL {
        help.push_str(&format!(
            "  {:<10} {}\n",
            verb.name(),
            verb.help_description()
        ));
    }
    help.push_str(concat!(
        "\nSource operands:\n",
        "  parse <authority> <identity> <file> [--limit name=value] [--format json|text]\n",
        "  format <authority> <identity> <file> [--format json|text]\n",
        "  check <source-path> [--model <file>] [--library <file>]... [--limit name=value]\n",
        "      [--dependent-bytes <bytes>] [--format json|text]\n",
        "  compile <request-file> [--clocks <json-file>] [--output <file>]\n",
        "      [--format json|text]\n",
        "  run <request-file> [--request-bytes <bytes>] [--dependent-bytes <bytes>]\n",
        "      [--limit name=value] [--engine interpreter|aot|jit] [--provider-config <path>]\n",
        "      [--clocks <json-file>] [--format json|text]\n",
        "\nOptions:\n",
        "  --format text|json|dot  Select output format (default: text).\n",
        "                         dot is accepted only for inspect.\n",
        "  --version, -V           Show the package version.\n",
        "  --help, -h, help        Show this help without running an operation.\n",
    ));
    process_exit_code(write_primary_output(
        help.as_bytes(),
        "root help",
        Category::Success,
    ))
}

/// Output selector for FSL ADR-029's CLI grammar.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Dot,
}

/// Dispatch selector for verbs specified by FSL ADR-029.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Verb {
    Parse,
    Format,
    Check,
    Compile,
    Run,
    Analyze,
    Monitor,
    Prove,
    Replay,
    Generate,
    Inspect,
    Render,
}

impl Verb {
    const ALL: [Self; 12] = [
        Self::Parse,
        Self::Format,
        Self::Check,
        Self::Compile,
        Self::Run,
        Self::Analyze,
        Self::Monitor,
        Self::Prove,
        Self::Replay,
        Self::Generate,
        Self::Inspect,
        Self::Render,
    ];

    fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|verb| verb.name() == value)
    }

    fn name(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Format => "format",
            Self::Check => "check",
            Self::Compile => "compile",
            Self::Run => "run",
            Self::Analyze => "analyze",
            Self::Monitor => "monitor",
            Self::Prove => "prove",
            Self::Replay => "replay",
            Self::Generate => "generate",
            Self::Inspect => "inspect",
            Self::Render => "render",
        }
    }

    fn help_description(self) -> &'static str {
        match self {
            Self::Parse | Self::Check | Self::Compile | Self::Run => {
                "Executes with --format json; default text output is unavailable."
            }
            Self::Format => "Executes source formatting; text output is the default.",
            Self::Analyze => "Executes a native-analyze/1 request; --format json or text.",
            Self::Monitor => {
                "Executes a native-monitor/1 request over a record stream; --format json or text."
            }
            Self::Prove | Self::Replay | Self::Generate | Self::Inspect | Self::Render => {
                "Recognized; currently returns an unsupported-capability result."
            }
        }
    }
}

#[derive(Debug)]
enum Invocation {
    Help,
    Version,
    Operation {
        verb: Verb,
        format: OutputFormat,
        raw_arguments: Vec<OsString>,
    },
}

/// Parse the root command surface defined by FSL ADR-029.
fn parse_invocation(arguments: &[OsString]) -> Result<Invocation, String> {
    let Some(command) = arguments.first() else {
        return Ok(Invocation::Help);
    };
    let command = command
        .to_str()
        .ok_or_else(|| "command must be valid UTF-8".to_owned())?;
    if matches!(command, "--help" | "-h" | "help") {
        return Ok(Invocation::Help);
    }
    if matches!(command, "--version" | "-V") {
        return Ok(Invocation::Version);
    }
    let verb = Verb::parse(command).ok_or_else(|| format!("unknown verb `{command}`"))?;
    let mut format = OutputFormat::Text;
    let mut index = 1;
    while index < arguments.len() {
        if arguments[index] == "--format" {
            index += 1;
            let value = arguments
                .get(index)
                .and_then(|value| value.to_str())
                .ok_or_else(|| "--format requires text, json, or dot".to_owned())?;
            format = match value {
                "text" => OutputFormat::Text,
                "json" => OutputFormat::Json,
                "dot" => OutputFormat::Dot,
                other => return Err(format!("unknown output format `{other}`")),
            };
        }
        // Operation-specific options pass through untouched for later handlers.
        index += 1;
    }
    if format == OutputFormat::Dot && verb != Verb::Inspect {
        return Err("the `dot` format is available only for `inspect`".to_owned());
    }
    Ok(Invocation::Operation {
        verb,
        format,
        raw_arguments: arguments[1..].to_vec(),
    })
}

struct RunOptions {
    path: PathBuf,
    intake_limits: RunLimits,
    limits: LimitOverrides,
    engine: ExecutionEngine,
    provider_config: Option<std::path::PathBuf>,
    clocks: Option<PathBuf>,
}

type CliProviderError = crate::process_provider::AdapterError<
    crate::backend_provider::ManifestAdmissionError,
    String,
    serde_json::Value,
>;
#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
type CliProviderEncode =
    for<'a> fn(ProcessProviderRequest<'a>) -> Result<crate::backend_provider::RequestStage, String>;
#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
type CliProviderValidate =
    fn(&crate::backend_provider::ManifestStage, &serde_json::Value) -> Result<(), String>;
#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
type CliProviderConvert =
    fn(
        serde_json::Value,
    )
        -> Result<ProviderResults<crate::process_provider::ProcessProviderOutcome, String>, String>;
#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
type CliProviderSession = crate::process_provider::HostProcessSession<
    crate::backend_provider::ManifestStage,
    crate::backend_provider::RequestStage,
    serde_json::Value,
    serde_json::Value,
    crate::backend_provider::ManifestAdmissionError,
    String,
    crate::process_provider::ProcessProviderOutcome,
    String,
    CliProviderEncode,
    CliProviderValidate,
    CliProviderConvert,
>;

#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
struct ActiveConfiguredProvider {
    deadline_ms: u64,
    session: CliProviderSession,
}

#[derive(Debug)]
#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
struct ConfiguredCall {
    package_name: String,
    outcome: CallOutcome,
    requirements: fsl_replay::CallRequirements,
}

/// The configured run retains both compositions until FSL settlement owns the
/// provider result mapping. No provider result is converted to a terminal.
#[allow(
    dead_code,
    reason = "QSL-655 settlement will consume the retained typed composition"
)]
struct ConfiguredRun {
    call: ConfiguredCall,
    driven: Driven,
    provider_results: Vec<ProviderResults<crate::process_provider::ProcessProviderOutcome, String>>,
    diagnostics: Vec<filament_plugin_host::SessionFinished<crate::backend_provider::ManifestStage>>,
}

#[allow(
    dead_code,
    reason = "typed configured-run failures remain unmapped until their FSL contracts land"
)]
#[derive(Debug)]
enum ConfiguredRunErrorKind {
    Call(Box<RunRefusal>),
    DuplicateIdentity {
        backend: String,
    },
    Start {
        provider: String,
        source: CliProviderError,
    },
    Register(filament_driver::DriverRefusal),
    Drive(filament_driver::DriverRefusal),
    PackageMismatch,
    Pairing(crate::process_provider::CallItemProjectionError),
    Invoke {
        provider: String,
        source: CliProviderError,
        partial_results:
            Vec<ProviderResults<crate::process_provider::ProcessProviderOutcome, String>>,
    },
    Shutdown {
        partial_results:
            Vec<ProviderResults<crate::process_provider::ProcessProviderOutcome, String>>,
    },
}

#[allow(
    dead_code,
    reason = "QSL-655 settlement will consume typed failure and cleanup details"
)]
#[derive(Debug)]
struct ConfiguredRunFailure {
    call: Option<ConfiguredCall>,
    document: OutcomeDocument,
    kind: ConfiguredRunErrorKind,
    cleanup_diagnostics:
        Vec<filament_plugin_host::SessionFinished<crate::backend_provider::ManifestStage>>,
    cleanup_failures: Vec<CliProviderError>,
}

#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
struct ConfiguredRunRequest<'a> {
    path: &'a str,
    package_name: &'a str,
    bytes: &'a [u8],
    source: SourceIdentity,
    limits: SpineLimits,
    call: Call,
    packages: fsl_semantics::model::intake::PackageInput,
    dependencies: DependencyInput,
    config: Vec<crate::provider_config::RuntimeProviderConfig>,
    signals: crate::signals::SignalContext,
}

#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
fn close_configured_providers(
    sessions: Vec<ActiveConfiguredProvider>,
) -> (
    Vec<filament_plugin_host::SessionFinished<crate::backend_provider::ManifestStage>>,
    Vec<CliProviderError>,
) {
    let mut diagnostics = Vec::new();
    let mut failures = Vec::new();
    for active in sessions {
        match active.session.shutdown() {
            Ok(finished) => diagnostics.push(finished),
            Err(error) => failures.push(error),
        }
    }
    (diagnostics, failures)
}

#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
fn configured_failure_after_call(
    call: ConfiguredCall,
    kind: ConfiguredRunErrorKind,
    sessions: Vec<ActiveConfiguredProvider>,
) -> Box<ConfiguredRunFailure> {
    let document = call_outcome_document(&call.outcome);
    let (cleanup_diagnostics, cleanup_failures) = close_configured_providers(sessions);
    Box::new(ConfiguredRunFailure {
        call: Some(call),
        document,
        kind,
        cleanup_diagnostics,
        cleanup_failures,
    })
}

struct CompileOptions {
    path: PathBuf,
    clocks: Option<PathBuf>,
    output: Option<PathBuf>,
}

struct CheckOptions {
    path: String,
    model: Option<PathBuf>,
    libraries: Vec<PathBuf>,
    dependent_bytes: usize,
    limits: LimitOverrides,
}

struct SourceOptions {
    source: SourceIdentity,
    path: PathBuf,
    limits: LimitOverrides,
}

impl SourceOptions {
    fn parse(verb: Verb, arguments: &[OsString]) -> Result<Self, String> {
        let limit_usage = if verb == Verb::Parse {
            " [--limit name=value]"
        } else {
            ""
        };
        let usage = || {
            format!(
                "usage: fsl {} <authority> <identity> <file>{limit_usage} [--format json|text]",
                verb.name(),
            )
        };
        let mut positionals = Vec::new();
        let mut limit_operands = Vec::new();
        let mut index = 0;
        while index < arguments.len() {
            let argument = &arguments[index];
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument == "--limit" && verb == Verb::Parse {
                let value = arguments
                    .get(index + 1)
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| "--limit requires name=value".to_owned())?;
                limit_operands.push(value.to_owned());
                index += 2;
                continue;
            }
            if let Some(option) = argument.to_str().filter(|value| value.starts_with('-')) {
                return Err(format!("unknown {} option `{option}`", verb.name()));
            }
            positionals.push(argument);
            index += 1;
        }
        let [authority, identity, path] = positionals.as_slice() else {
            return Err(usage());
        };
        let authority = authority
            .to_str()
            .ok_or_else(|| format!("{} authority label must be valid UTF-8", verb.name()))?;
        let identity = identity
            .to_str()
            .ok_or_else(|| format!("{} identity label must be valid UTF-8", verb.name()))?;
        let limits = parse_frontend_limit_operands(&limit_operands, &fsl_cst::Limits::default())?;
        Ok(Self {
            source: SourceIdentity::new(authority, identity),
            path: PathBuf::from(*path),
            limits,
        })
    }
}

impl CompileOptions {
    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut clocks = None;
        let mut output: Option<PathBuf> = None;
        let mut index = 0;
        while index < arguments.len() {
            let argument = &arguments[index];
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument == "--clocks" {
                clocks = Some(clocks_operand(arguments.get(index + 1), clocks.is_some())?);
                index += 2;
                continue;
            }
            if argument == "--output" {
                output = Some(output_operand(arguments.get(index + 1), output.is_some())?);
                index += 2;
                continue;
            }
            if let Some(option) = argument.to_str().filter(|value| value.starts_with('-')) {
                return Err(format!("unknown compile option `{option}`"));
            }
            positionals.push(argument);
            index += 1;
        }
        let [path] = positionals.as_slice() else {
            return Err(
                "usage: fsl compile <request-file> [--clocks <json-file>] [--output <file>] [--format json|text]"
                    .to_owned(),
            );
        };
        Ok(Self {
            path: PathBuf::from(*path),
            clocks,
            output,
        })
    }
}

impl CheckOptions {
    const USAGE: &str = "usage: fsl check <source-path> [--model <file>] [--library <file>]... [--dependent-bytes <bytes>] [--limit name=value] [--format json|text]";

    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut model = None;
        let mut libraries = Vec::new();
        let mut limit_operands = Vec::new();
        let mut dependent_bytes = RunLimits::default().dependent_bytes;
        let mut dependent_bytes_supplied = false;
        let mut index = 0;
        while index < arguments.len() {
            let argument = arguments[index]
                .to_str()
                .ok_or_else(|| "check arguments must be valid UTF-8".to_owned())?;
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument == "--model" {
                let path = arguments
                    .get(index + 1)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "--model requires a path".to_owned())?;
                if model.replace(PathBuf::from(path)).is_some() {
                    return Err("--model may be supplied only once".to_owned());
                }
                index += 2;
                continue;
            }
            if argument == "--library" {
                let path = arguments
                    .get(index + 1)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "--library requires a path".to_owned())?;
                libraries.push(PathBuf::from(path));
                index += 2;
                continue;
            }
            if argument == "--dependent-bytes" {
                if dependent_bytes_supplied {
                    return Err("--dependent-bytes may be supplied only once".to_owned());
                }
                let value = arguments
                    .get(index + 1)
                    .and_then(|value| value.to_str())
                    .and_then(|value| value.parse::<usize>().ok())
                    .ok_or_else(|| {
                        "--dependent-bytes requires a non-negative byte count".to_owned()
                    })?;
                dependent_bytes = value;
                dependent_bytes_supplied = true;
                index += 2;
                continue;
            }
            if argument == "--limit" {
                let value = arguments
                    .get(index + 1)
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| "--limit requires name=value".to_owned())?;
                limit_operands.push(value.to_owned());
                index += 2;
                continue;
            }
            if argument.starts_with('-') {
                return Err(format!("unknown check option `{argument}`"));
            }
            positionals.push(argument.to_owned());
            index += 1;
        }
        if positionals.len() != 1 {
            return Err(Self::USAGE.to_owned());
        }
        let limits = parse_frontend_limit_operands(&limit_operands, &SpineLimits::default())?;
        Ok(Self {
            path: positionals.remove(0),
            model,
            libraries,
            dependent_bytes,
            limits,
        })
    }
}

fn parse_frontend_limit_operands(
    operands: &[String],
    bounds: &impl SettingLimits,
) -> Result<LimitOverrides, String> {
    LimitOverrides::parse(operands).map_err(|error| {
        let accepted = bounds
            .bounds()
            .into_iter()
            .map(|(setting, _)| setting.name())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{error}; settings accepted by this command: {accepted}")
    })
}

fn apply_frontend_limits(
    overrides: &LimitOverrides,
    bounds: &mut impl SettingLimits,
    operation: &str,
) -> Result<(), String> {
    overrides
        .apply_to_limits(bounds)
        .map_err(|error| error.message_for(operation))
}

impl RunOptions {
    const USAGE: &str = "usage: fsl run <request-file> [--request-bytes <bytes>] [--dependent-bytes <bytes>] [--limit name=value] [--engine interpreter|aot|jit] [--provider-config <path>] [--clocks <json-file>] [--format json|text]";

    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut limit_operands = Vec::new();
        let mut engine = None;
        let mut intake_limits = RunLimits::default();
        let mut byte_options = BTreeSet::new();
        let mut provider_config: Option<std::path::PathBuf> = None;
        let mut clocks: Option<PathBuf> = None;
        let mut index = 0;
        while index < arguments.len() {
            let arg = arguments[index].to_str();
            match arg {
                Some("--request-bytes" | "--dependent-bytes") => {
                    let option = arg.unwrap_or_default();
                    if !byte_options.insert(option) {
                        return Err(format!("{option} may be supplied only once"));
                    }
                    index += 1;
                    let value = arguments
                        .get(index)
                        .and_then(|value| value.to_str())
                        .and_then(|value| value.parse::<usize>().ok())
                        .ok_or_else(|| format!("{option} requires a non-negative byte count"))?;
                    if option == "--request-bytes" {
                        intake_limits.request_bytes = value;
                    } else {
                        intake_limits.dependent_bytes = value;
                    }
                }
                Some("--limit") => {
                    index += 1;
                    limit_operands.push(
                        arguments
                            .get(index)
                            .and_then(|value| value.to_str())
                            .ok_or_else(|| "--limit requires name=value".to_owned())?
                            .to_owned(),
                    );
                }
                Some("--engine") => {
                    index += 1;
                    let value = arguments
                        .get(index)
                        .and_then(|value| value.to_str())
                        .ok_or_else(|| "--engine requires interpreter, aot, or jit".to_owned())?;
                    if engine.is_some() {
                        return Err("--engine may be supplied only once".to_owned());
                    }
                    engine = Some(ExecutionEngine::parse(value)?);
                }
                Some("--provider-config") => {
                    index += 1;
                    let path = arguments
                        .get(index)
                        .filter(|value| !value.is_empty())
                        .ok_or_else(|| "--provider-config requires a path".to_owned())?;
                    if provider_config
                        .replace(std::path::PathBuf::from(path))
                        .is_some()
                    {
                        return Err("--provider-config may be supplied only once".to_owned());
                    }
                }
                Some("--clocks") => {
                    clocks = Some(clocks_operand(arguments.get(index + 1), clocks.is_some())?);
                    index += 1;
                }
                Some("--format" | "--color") => {
                    index += 1;
                    if arguments
                        .get(index)
                        .and_then(|value| value.to_str())
                        .is_none()
                    {
                        return Err(format!("{} requires a value", arg.unwrap_or_default()));
                    }
                }
                Some(option) if option.starts_with('-') => {
                    return Err(format!("unknown run option `{option}`"));
                }
                _ => positionals.push(arguments[index].clone()),
            }
            index += 1;
        }
        let [path] = positionals.as_slice() else {
            return Err(Self::USAGE.to_owned());
        };
        Ok(Self {
            path: PathBuf::from(path),
            intake_limits,
            limits: LimitOverrides::parse(&limit_operands).map_err(|error| error.to_string())?,
            engine: engine.unwrap_or_default(),
            provider_config,
            clocks,
        })
    }
}

/// One command's FSL category, optional outcome document, and diagnostic.
struct CommandResult {
    category: Category,
    document: Option<OutcomeDocument>,
    diagnostic: Option<String>,
    text_output: Option<String>,
    #[allow(
        dead_code,
        reason = "keep provider results typed until QSL-655 defines their mapping"
    )]
    configured_run: Option<Result<ConfiguredRun, Box<ConfiguredRunFailure>>>,
    /// The FSL FR-621 `analyze` document, written in machine mode in place
    /// of `document`.
    analyze_document: Option<serde_json::Value>,
    /// One plain stderr line, written in place of the document diagnostics.
    stderr_line: Option<String>,
}

impl CommandResult {
    fn document(document: OutcomeDocument) -> Self {
        Self {
            category: document.category(),
            document: Some(document),
            diagnostic: None,
            text_output: None,
            configured_run: None,
            analyze_document: None,
            stderr_line: None,
        }
    }

    fn failure(category: Category, message: impl Into<String>) -> Self {
        Self {
            category,
            document: None,
            diagnostic: Some(message.into()),
            text_output: None,
            configured_run: None,
            analyze_document: None,
            stderr_line: None,
        }
    }

    fn formatted(document: OutcomeDocument, text_output: Option<String>) -> Self {
        Self {
            category: document.category(),
            document: Some(document),
            diagnostic: None,
            text_output,
            configured_run: None,
            analyze_document: None,
            stderr_line: None,
        }
    }

    /// An input refusal raised before FSL runs, recorded with FSL's native code.
    fn input_refusal(operation: Operation, message: impl Into<String>) -> Self {
        Self::catalog_failure_for(operation, Code::InvalidRequest, message)
    }

    fn catalog_failure_for(operation: Operation, code: Code, message: impl Into<String>) -> Self {
        let message = message.into();
        Self::document(
            OutcomeDocument::new(operation, None, code.category()).with_diagnostics(vec![
                OutcomeDiagnostic::new(None, code.as_str(), None, message.clone()),
            ]),
        )
        .with_diagnostic(message)
    }

    fn with_diagnostic(mut self, message: impl Into<String>) -> Self {
        self.diagnostic = Some(message.into());
        self
    }

    #[allow(
        dead_code,
        reason = "provider execution awaits the provider-limit CLI ruling"
    )]
    fn configured(run: Result<ConfiguredRun, Box<ConfiguredRunFailure>>) -> Self {
        let document = match &run {
            Ok(completed) => Some(call_outcome_document(&completed.call.outcome)),
            Err(failure) => Some(failure.document.clone()),
        };
        let category = document
            .as_ref()
            .map_or(Category::InternalFailure, OutcomeDocument::category);
        Self {
            category,
            document,
            diagnostic: None,
            text_output: None,
            configured_run: Some(run),
            analyze_document: None,
            stderr_line: None,
        }
    }
}

#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
fn call_outcome_document(outcome: &CallOutcome) -> OutcomeDocument {
    OutcomeDocument::from_call(outcome)
}

fn write_result_diagnostics<W: Write>(result: &CommandResult, sink: &mut W) -> io::Result<()> {
    if let Some(line) = &result.stderr_line {
        return writeln!(sink, "{line}");
    }
    if let Some(message) = &result.diagnostic {
        return write_diagnostic(sink, "DriverDiagnostic", message);
    }
    if let Some(document) = &result.document {
        for diagnostic in document.diagnostics() {
            write_diagnostic(sink, "FslDiagnostic", diagnostic.message())?;
        }
    }
    Ok(())
}

/// The path operand of `--clocks`, refused when missing or repeated.
fn clocks_operand(operand: Option<&OsString>, repeated: bool) -> Result<PathBuf, String> {
    if repeated {
        return Err("--clocks may be supplied only once".to_owned());
    }
    operand
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "--clocks requires a path".to_owned())
}

/// The path operand of `--output`, refused when missing or repeated.
fn output_operand(operand: Option<&OsString>, repeated: bool) -> Result<PathBuf, String> {
    if repeated {
        return Err("--output may be supplied only once".to_owned());
    }
    operand
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "--output requires a path".to_owned())
}

/// Record a refused `--clocks` as FSL records an invalid request: no stage
/// reached, the catalog code and the cause naming the flag.
fn clocks_refusal(operation: Operation, refusal: crate::clocks::ClocksRefusal) -> CommandResult {
    let message = refusal.message;
    CommandResult::document(
        OutcomeDocument::new(operation, None, refusal.code.category()).with_diagnostics(vec![
            OutcomeDiagnostic::new(
                Some(refusal.cause),
                refusal.code.as_str(),
                None,
                message.clone(),
            ),
        ]),
    )
    .with_diagnostic(message)
}

/// Run one complete-V1 function through FSL's interpreter.
fn run(options: RunOptions) -> CommandResult {
    let provider_config = match options.provider_config.as_deref() {
        Some(path) => match crate::provider_config::load(path) {
            Ok(config) => config,
            Err(error) => {
                if matches!(
                    error,
                    crate::provider_config::ProviderConfigError::UnsupportedFormat { .. }
                ) {
                    let message = format!("unknown_wire/unsupported-wire: {error}");
                    return CommandResult::document(
                        OutcomeDocument::new(
                            Operation::Execute,
                            None,
                            Code::UnknownWire.category(),
                        )
                        .with_diagnostics(vec![OutcomeDiagnostic::new(
                            Some("unsupported-wire"),
                            Code::UnknownWire.as_str(),
                            None,
                            message.clone(),
                        )]),
                    )
                    .with_diagnostic(message);
                } else {
                    return CommandResult::input_refusal(
                        Operation::Execute,
                        format!("invalid provider configuration: {error}"),
                    );
                }
            }
        },
        None => crate::provider_config::ProviderConfig {
            format: crate::provider_config::FORMAT.to_owned(),
            providers: Vec::new(),
        },
    };
    if options.engine != ExecutionEngine::Interpreter {
        let failure = ExecutionFailure::UnsupportedEngine(options.engine);
        return CommandResult::document(failure.into_document());
    }
    // Reject a setting unavailable to every run family before request-file I/O.
    // The selected function or clause still validates its exact bounds below.
    let mut exposed = SpineLimits::default().bounds();
    exposed.extend(fsl_replay::ObservationLimits::default().bounds());
    for (setting, _) in options.limits.values() {
        if let SettingName::Stage(stage) = setting {
            if !exposed.iter().any(|(available, _)| available == stage) {
                return CommandResult::input_refusal(
                    Operation::Execute,
                    filament_driver::LimitError::OperationUnavailable { setting: *setting }
                        .message_for("run"),
                );
            }
        }
    }
    let clocks = match options
        .clocks
        .as_deref()
        .map(|path| {
            crate::clocks::read(
                path,
                options.intake_limits.request_bytes,
                Some("--request-bytes"),
            )
        })
        .transpose()
    {
        Ok(clocks) => clocks,
        Err(refusal) => return clocks_refusal(Operation::Execute, refusal),
    };
    let overrides = command::RequestOverrides {
        clocks: clocks.as_deref(),
        ..command::RequestOverrides::default()
    };
    let prepared =
        match command::prepare_run_with_overrides(&options.path, options.intake_limits, &overrides)
        {
            Ok(prepared) => prepared,
            Err(error) => {
                return CommandResult::document(error.outcome_document(Operation::Execute))
            }
        };
    let input = match prepared {
        PreparedRun::Function(prepared) => prepared.into_input(),
        PreparedRun::Clause(request) => {
            if !provider_config.providers.is_empty() {
                return CommandResult::input_refusal(
                    Operation::Execute,
                    "valid FR-561 providers are loaded, but the driver has no process-provider engine adapter yet",
                );
            }
            return run_clause_request(*request, &options.limits);
        }
        PreparedRun::TlLowering(_) => {
            return CommandResult::input_refusal(
                Operation::Execute,
                "FR-931 tl_lowering requests are not run by the driver yet",
            );
        }
    };
    let command::FunctionRunInput {
        source,
        package_name,
        packages,
        dependencies,
        mut call,
    } = input;
    let mut limits = SpineLimits::default();
    if let Err(error) = options.limits.apply_to(&mut limits, &mut call) {
        return CommandResult::input_refusal(Operation::Execute, error.to_string());
    }
    let identity = source.identity().clone();
    if !provider_config.providers.is_empty() {
        return CommandResult::input_refusal(
            Operation::Execute,
            "valid FR-561 providers are loaded, but the driver has no process-provider engine adapter yet",
        );
    }
    match execute(ExecutionRequest {
        source: identity,
        package_name: &package_name,
        path: source.path(),
        bytes: source.text().as_bytes(),
        packages: &packages,
        dependencies: &dependencies,
        limits,
        call,
        object_environment: &ObjectEnvironment::default(),
        cancel: Cancel::new(),
        engine: options.engine,
    }) {
        Ok(result) => CommandResult::document(result.document),
        Err(failure) => CommandResult::document(failure.into_document()),
    }
}

/// Dispatch typed clause dispositions to FSL's existing document producers.
fn run_clause_request(
    mut request: fsl_replay::spine::ClauseRunRequest,
    limits: &LimitOverrides,
) -> CommandResult {
    use fsl_replay::spine::{
        ClauseDisposition, ClauseRunSelection, ClauseRunStage, TemporalTraceRefusal,
    };
    use fsl_semantics::model::observation::AdmissionFailure;
    if let Err(error) = limits.apply_to_clause(&mut request) {
        return CommandResult::input_refusal(Operation::Monitor, error.to_string());
    }
    let name = match &request.selection {
        ClauseRunSelection::Clause(selection) => selection.name.clone(),
        ClauseRunSelection::Function { name, .. }
        | ClauseRunSelection::Temporal { name, .. }
        | ClauseRunSelection::SampledTemporal { name, .. } => name.clone(),
        ClauseRunSelection::Frame { operation, .. } => operation.to_string(),
    };
    let report = match fsl_replay::spine::run_clause(request) {
        Ok(report) => report,
        Err(error) => return CommandResult::input_refusal(Operation::Monitor, error.to_string()),
    };
    let document = match &report.disposition {
        ClauseDisposition::Compile(refusal) => OutcomeDocument::new(
            Operation::Monitor,
            Some(refusal.stage().into()),
            refusal.code().category(),
        )
        .with_diagnostics(vec![OutcomeDiagnostic::from_compile_refusal(refusal)]),
        ClauseDisposition::ArgumentRefusal(refusal) => {
            OutcomeDocument::from_run_refusal(Operation::Monitor, refusal)
        }
        ClauseDisposition::EvaluateFault(fault)
        | ClauseDisposition::Admit(AdmissionFailure::Fault(fault))
        | ClauseDisposition::TemporalRefusal(TemporalTraceRefusal::Fault(fault)) => {
            OutcomeDocument::from_run_refusal(Operation::Monitor, &RunRefusal::Fault(fault.clone()))
        }
        _ => {
            let last_stage = match report.disposition.stage() {
                ClauseRunStage::Compile => None,
                ClauseRunStage::Select | ClauseRunStage::Admit => Some(OutcomeStage::S4),
                ClauseRunStage::Evaluate => Some(OutcomeStage::S6a),
            };
            match OutcomeDocument::from_clause_runs(last_stage, [(name.as_str(), &report)]) {
                Ok(document) => document,
                Err(error) => {
                    return CommandResult::failure(Category::InternalFailure, error.to_string())
                }
            }
        }
    };
    CommandResult::document(document)
}

#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
fn run_configured_providers(
    request: ConfiguredRunRequest<'_>,
) -> Result<ConfiguredRun, Box<ConfiguredRunFailure>> {
    let ConfiguredRunRequest {
        path,
        package_name,
        bytes,
        source,
        limits,
        call,
        packages,
        dependencies,
        config,
        signals,
    } = request;
    let cancel = Cancel::new();
    let objects = ObjectEnvironment::default();

    // This is the one interpreter call. The driver performs the separate
    // package negotiation/generation pipeline below and does not call execute.
    let call_run = run_with_requirements(&RequirementsRun {
        source: source.clone(),
        package_name: package_name.to_owned(),
        path,
        bytes,
        packages: &packages,
        dependencies: &dependencies,
        limits,
        call: &call,
        objects: &objects,
        cancel: &cancel,
    });
    let call_run = match call_run {
        Ok(run) => run,
        Err(refusal) => {
            let failed_run: Result<CallOutcome, Box<RunRefusal>> = Err(refusal);
            let document = OutcomeDocument::from_run(&failed_run);
            let Err(refusal) = failed_run else {
                return Err(Box::new(ConfiguredRunFailure {
                    call: None,
                    document,
                    kind: ConfiguredRunErrorKind::PackageMismatch,
                    cleanup_diagnostics: Vec::new(),
                    cleanup_failures: Vec::new(),
                }));
            };
            return Err(Box::new(ConfiguredRunFailure {
                call: None,
                document,
                kind: ConfiguredRunErrorKind::Call(refusal),
                cleanup_diagnostics: Vec::new(),
                cleanup_failures: Vec::new(),
            }));
        }
    };
    let selected_package_name = call_run.package_name().as_str().to_owned();
    let (outcome, requirements) = call_run.into_parts();
    let call = ConfiguredCall {
        package_name: selected_package_name,
        outcome,
        requirements,
    };

    // Start and admit every configured process before the driver's negotiation.
    let mut active = Vec::with_capacity(config.len());
    for configured in config {
        let provider_name = configured.executable.display().to_string();
        let max_frame_bytes = configured.limits.max_frame_bytes;
        let provider = crate::process_provider::HostProcessProvider::new(
            signals.clone(),
            move |manifest: &crate::backend_provider::ManifestStage| {
                manifest
                    .admit_process(max_frame_bytes)
                    .map(|admitted| (admitted.descriptor().clone(), admitted.manifest().clone()))
            },
            crate::process_provider::encode_fr331_request as CliProviderEncode,
            crate::process_provider::validate_fr331_response as CliProviderValidate,
            crate::process_provider::convert_fr331_response as CliProviderConvert,
        );
        let arguments = configured
            .argv
            .into_iter()
            .map(OsString::from)
            .collect::<Vec<_>>();
        let process_limits = ProcessProviderLimits {
            max_frame_bytes: configured.limits.max_frame_bytes,
            max_total_bytes: configured.limits.max_total_bytes,
            max_stderr_bytes: configured.limits.max_stderr_bytes,
            cancellation_grace: std::time::Duration::from_millis(
                configured.limits.cancellation_grace_ms,
            ),
        };
        match provider.start(&configured.executable, &arguments, process_limits, &cancel) {
            Ok(session) => active.push(ActiveConfiguredProvider {
                deadline_ms: configured.deadline_ms,
                session,
            }),
            Err(source) => {
                return Err(configured_failure_after_call(
                    call,
                    ConfiguredRunErrorKind::Start {
                        provider: provider_name,
                        source,
                    },
                    active,
                ));
            }
        }
    }

    let admissions = active.iter().map(|provider| {
        (
            provider.session.descriptor().clone(),
            provider.session.manifest().clone(),
        )
    });
    let mut linked = match build_linked_providers(&[]) {
        Ok(linked) => linked,
        Err(source) => {
            return Err(configured_failure_after_call(
                call,
                ConfiguredRunErrorKind::Register(source),
                active,
            ));
        }
    };
    if let Err(source) = linked.register_admitted(admissions) {
        return Err(configured_failure_after_call(
            call,
            ConfiguredRunErrorKind::Register(source),
            active,
        ));
    }

    let mut admitted_identities = BTreeSet::new();
    if let Some(backend) = active.iter().find_map(|provider| {
        let identity = provider.session.descriptor().id().as_str();
        (!admitted_identities.insert(identity.to_owned())).then(|| identity.to_owned())
    }) {
        let message = format!("configured process providers repeat admitted identity `{backend}`");
        let document =
            OutcomeDocument::new(Operation::Execute, None, Code::InvalidRequest.category())
                .with_diagnostics(vec![OutcomeDiagnostic::new(
                    None,
                    Code::InvalidRequest.as_str(),
                    None,
                    message,
                )]);
        let (cleanup_diagnostics, cleanup_failures) = close_configured_providers(active);
        return Err(Box::new(ConfiguredRunFailure {
            call: Some(call),
            document,
            kind: ConfiguredRunErrorKind::DuplicateIdentity { backend },
            cleanup_diagnostics,
            cleanup_failures,
        }));
    }

    let driven = match drive(
        &FslCg,
        CompileInput {
            source,
            package_name,
            path,
            bytes,
            packages: &packages,
            limits,
            cancel: cancel.clone(),
        },
        linked.registry(),
        None,
        GenerationInput::default(),
    ) {
        Ok(driven) => driven,
        Err(source) => {
            return Err(configured_failure_after_call(
                call,
                ConfiguredRunErrorKind::Drive(source),
                active,
            ));
        }
    };
    let projection = match crate::process_provider::project_call_items(
        &call.package_name,
        &call.requirements,
        &driven,
    ) {
        Ok(projection) => projection,
        Err(source) => {
            return Err(configured_failure_after_call(
                call,
                ConfiguredRunErrorKind::Pairing(source),
                active,
            ));
        }
    };
    let manifest_snapshot = linked.manifest_snapshot();
    let mut provider_results = Vec::new();
    let mut invoke_failure = None;
    for provider in &mut active {
        let backend = provider.session.descriptor().id().as_str().to_owned();
        let routed = projection
            .routed
            .iter()
            .filter(|item| item.backend.identity == backend)
            .cloned()
            .collect::<Vec<_>>();
        if routed.is_empty() {
            continue;
        }
        let routed_indices = routed
            .iter()
            .map(|item| item.request_index)
            .collect::<BTreeSet<_>>();
        let fsl_items = projection
            .items
            .iter()
            .filter(|item| routed_indices.contains(&item.index().get()))
            .cloned()
            .collect::<Vec<_>>();
        let request = ProcessProviderRequest {
            emitted_package: driven.emitted(),
            deadline_ms: provider.deadline_ms,
            manifest: &manifest_snapshot,
            provider_request: ProviderRequest::new(&routed),
            fsl_items: &fsl_items,
            cancel: &cancel,
        };
        match provider.session.invoke(request) {
            Ok(results) => provider_results.push(results),
            Err(source) => {
                invoke_failure = Some((backend, source));
                break;
            }
        }
    }
    if let Some((provider, source)) = invoke_failure {
        return Err(configured_failure_after_call(
            call,
            ConfiguredRunErrorKind::Invoke {
                provider,
                source,
                partial_results: provider_results,
            },
            active,
        ));
    }

    let (diagnostics, failures) = close_configured_providers(active);
    if !failures.is_empty() {
        let document = call_outcome_document(&call.outcome);
        return Err(Box::new(ConfiguredRunFailure {
            call: Some(call),
            document,
            kind: ConfiguredRunErrorKind::Shutdown {
                partial_results: provider_results,
            },
            cleanup_diagnostics: diagnostics,
            cleanup_failures: failures,
        }));
    }
    Ok(ConfiguredRun {
        call,
        driven,
        provider_results,
        diagnostics,
    })
}

/// Compile a request through its owner and retain the actual package outcome.
fn compile(options: CompileOptions) -> CommandResult {
    let clocks = match options
        .clocks
        .as_deref()
        .map(|path| crate::clocks::read(path, RunLimits::default().request_bytes, None))
        .transpose()
    {
        Ok(clocks) => clocks,
        Err(refusal) => return clocks_refusal(Operation::Package, refusal),
    };
    let overrides = command::RequestOverrides {
        clocks: clocks.as_deref(),
        output: options.output.as_deref(),
        ..command::RequestOverrides::default()
    };
    match command::compile_outcome_with_overrides(&options.path, &overrides) {
        Ok(outcome) => CommandResult::document(outcome.document().clone()),
        Err(error) if matches!(error.cause, command::RunCause::EmptyCompileProgram) => {
            let code = Code::InvalidPackage;
            CommandResult::document(
                OutcomeDocument::new(Operation::Package, None, code.category()).with_diagnostics(
                    vec![OutcomeDiagnostic::new(
                        Some("invalid-value"),
                        code.as_str(),
                        None,
                        error.to_string(),
                    )],
                ),
            )
        }
        Err(error) => CommandResult::document(error.outcome_document(Operation::Package)),
    }
}

/// Check one source through FSL's typed front-end and serialize its S4 outcome.
fn check_source(options: CheckOptions) -> CommandResult {
    let mut limits = SpineLimits::default();
    if let Err(error) = apply_frontend_limits(&options.limits, &mut limits, "check") {
        return CommandResult::input_refusal(Operation::Check, error);
    }
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure_for(Operation::Check, Code::IoError, message);
        }
    };
    let source = SourceIdentity::new("filament", &options.path);
    let cancel = Cancel::new();
    #[cfg(all(feature = "tc018-test-adapter", debug_assertions))]
    tc018_record_check_request(&options, &source, &bytes, limits);
    let parsed = match parse(
        &ParseRequest {
            source: &source,
            path: &options.path,
            bytes: &bytes,
        },
        limits.source,
        &cancel,
    ) {
        Ok(parsed) => parsed,
        Err(_failure) => {
            let inputs = command::CheckInputs {
                limits,
                dependent_bytes: options.dependent_bytes,
                ..command::CheckInputs::default()
            };
            return CommandResult::document(
                command::check_outcome(Path::new(&options.path), &inputs)
                    .document()
                    .clone(),
            );
        }
    };

    // The CLI accepts files while the typed check API also requires each
    // selected identity. Read those identities from the source first so
    // unused model and library paths are refused before their files open.
    let selected_models = &parsed.value().unit().selections().models;
    let selected_libraries = &parsed.value().unit().selections().imports;
    let models = options
        .model
        .map(|path| {
            let identity = selected_models
                .first()
                .map(|selection| selection.identity.clone())
                .unwrap_or_else(|| "unused-model".to_owned());
            (identity, path)
        })
        .into_iter()
        .collect();
    let libraries = options
        .libraries
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let identity = selected_libraries
                .get(index)
                .map(|selection| selection.identity.clone())
                .unwrap_or_else(|| format!("unused-library-{index}"));
            (identity, path)
        })
        .collect();
    let inputs = command::CheckInputs {
        models,
        libraries,
        limits,
        dependent_bytes: options.dependent_bytes,
        ..command::CheckInputs::default()
    };
    CommandResult::document(
        command::check_outcome(Path::new(&options.path), &inputs)
            .document()
            .clone(),
    )
}

/// Observe the TC-018 refused-range selection at the typed check boundary.
/// The normal FSL parse/check outcome supplies the result; argv parsing,
/// source reading, serialization and exit mapping remain the production paths.
#[cfg(all(feature = "tc018-test-adapter", debug_assertions))]
fn tc018_record_check_request(
    options: &CheckOptions,
    source: &SourceIdentity,
    bytes: &[u8],
    limits: SpineLimits,
) {
    if std::env::var_os("TC018_CASE").as_deref() != Some(std::ffi::OsStr::new("refused-range")) {
        return;
    }

    if let Some(observation_path) = std::env::var_os("TC018_REQUEST_PATH") {
        let profile = bytes
            .split(|byte| *byte == b'\n')
            .find(|line| line.starts_with(b"profile "))
            .map(String::from_utf8_lossy);
        let observation = serde_json::json!({
            "case": "refused-range",
            "source_identity": source,
            "path": options.path,
            "profile": profile.map(|line| line.into_owned()),
            "limits": format!("{limits:?}"),
            "source_bytes": bytes,
        });
        if let Ok(encoded) = serde_json::to_vec(&observation) {
            let _ = std::fs::write(observation_path, encoded);
        }
    }
}

/// Refuse an invalid logical label before opening its physical source path.
fn source_preflight(
    options: &SourceOptions,
    operation: Operation,
    byte_limit: usize,
) -> Option<CommandResult> {
    let label = options.source.first_blank_label()?;
    let diagnostic = fsl_cst::diagnostic::source_refusal(
        SourceReadRefusal {
            cause: SourceReadCause::BlankLabel { label },
            error: SourceReadError {
                source: options.source.clone(),
                path: options.path.to_string_lossy().into_owned(),
                region: None,
                message: format!("source {} label must not be blank", label.as_str()),
            },
        },
        byte_limit,
    );
    let refusal = CompileRefusal::Source(diagnostic);
    Some(CommandResult::document(
        OutcomeDocument::new(
            operation,
            Some(refusal.stage().into()),
            refusal.code().category(),
        )
        .with_diagnostics(vec![OutcomeDiagnostic::from_compile_refusal(&refusal)]),
    ))
}

/// Parse one source through FSL and serialize its typed parse outcome.
fn parse_source(options: SourceOptions) -> CommandResult {
    let mut limits = SpineLimits::default();
    if let Err(error) = apply_frontend_limits(&options.limits, &mut limits.source, "parse") {
        return CommandResult::input_refusal(Operation::Parse, error);
    }
    if let Some(refusal) = source_preflight(&options, Operation::Parse, limits.source.source_bytes)
    {
        return refusal;
    }
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path.display());
            return CommandResult::catalog_failure_for(Operation::Parse, Code::IoError, message);
        }
    };
    let source = options.source;
    let display_path = options.path.to_string_lossy();
    let result = parse(
        &ParseRequest {
            source: &source,
            path: &display_path,
            bytes: &bytes,
        },
        limits.source,
        &Cancel::new(),
    );
    CommandResult::document(OutcomeDocument::from_parse(&result))
}

/// Format one parsed source through FSL and retain its typed outcome.
fn format_source(options: SourceOptions) -> CommandResult {
    let limits = fsl_cst::Limits::default();
    let bytes = match read_source(&options.path, limits.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path.display());
            return CommandResult::catalog_failure_for(Operation::Format, Code::IoError, message);
        }
    };
    let source = options.source;
    let display_path = options.path.to_string_lossy();
    let cancel = Cancel::new();
    let result = fsl_cst::parse_with_cancel(source, display_path.as_ref(), &bytes, limits, &cancel)
        .map_err(FormatRefusal::DiagnosedSource)
        .and_then(|parsed| format_with_limits(&parsed, FormatLimits::default()).map(Staged::new));
    let document = OutcomeDocument::from_format(&result);
    let text_output = result.ok().map(Staged::into_value);
    CommandResult::formatted(document, text_output)
}

/// Read at most one byte beyond the caller's source limit. Passing that
/// sentinel byte to FSL preserves its typed source-limit refusal while
/// keeping the CLI's input buffer bounded by the configured limit.
fn read_source(path: impl AsRef<Path>, source_limit: usize) -> std::io::Result<Vec<u8>> {
    let read_limit = u64::try_from(source_limit)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let mut bytes = Vec::new();
    File::open(path)?.take(read_limit).read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// The operands of `fsl analyze`: one `native-analyze/1` request file. The
/// request document carries every analysis setting (FSL FR-620), so the verb
/// takes no other option than `--format`.
struct AnalyzeOptions {
    path: PathBuf,
}

impl AnalyzeOptions {
    const USAGE: &str = "usage: fsl analyze <request-file> [--format json|text]";

    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut index = 0;
        while index < arguments.len() {
            let argument = &arguments[index];
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument == "--provider-config" {
                return Err(
                    "`--provider-config` is not available for `analyze` yet: FSL's \
                            analyze entry takes no caller registry, so configured providers \
                            cannot be registered before negotiation"
                        .to_owned(),
                );
            }
            if let Some(option) = argument.to_str().filter(|value| value.starts_with('-')) {
                return Err(format!("unknown analyze option `{option}`"));
            }
            positionals.push(argument);
            index += 1;
        }
        let [path] = positionals.as_slice() else {
            return Err(Self::USAGE.to_owned());
        };
        Ok(Self {
            path: PathBuf::from(*path),
        })
    }
}

/// Read the `native-analyze/1` request, have FSL analyze its items under the
/// driver's negotiation, and keep FSL FR-621's document with its exit code,
/// or FR-624's whole-request refusal.
fn analyze(options: &AnalyzeOptions, format: OutputFormat) -> CommandResult {
    let written = command::analyze(
        &options.path,
        &filament_driver::AnalyzeNegotiation,
        &Cancel::new(),
    );
    match written {
        Ok(result) => analyze_result(result.value.as_value().clone(), result.exit_code, format),
        Err(error) => {
            let document = error.outcome_document(Operation::Analyze);
            if format == OutputFormat::Json {
                return CommandResult::document(document);
            }
            let line = document.diagnostics().first().map_or_else(
                || format!("fsl analyze: {}", document.category().as_str()),
                |diagnostic| {
                    format!(
                        "fsl analyze: {}: {}",
                        diagnostic.code(),
                        diagnostic.message()
                    )
                },
            );
            let mut result = CommandResult::document(document);
            result.stderr_line = Some(line);
            result
        }
    }
}

/// The CLI result of an accepted analyze request: the document's category
/// must be the one FSL's exit code names (FR-285), or the result is an
/// internal failure rather than a guessed exit.
fn analyze_result(
    document: serde_json::Value,
    exit_code: u8,
    format: OutputFormat,
) -> CommandResult {
    let label = document["category"].as_str().unwrap_or_default();
    let Some(category) = Category::ALL
        .into_iter()
        .find(|category| category.as_str() == label && category.exit_code() == exit_code)
    else {
        return CommandResult::failure(
            Category::InternalFailure,
            format!("the analyze document's category `{label}` does not give exit {exit_code}"),
        );
    };
    let text_output = (format == OutputFormat::Text).then(|| analyze_text(&document, category));
    CommandResult {
        category,
        document: None,
        diagnostic: None,
        text_output,
        configured_run: None,
        analyze_document: Some(document),
        stderr_line: None,
    }
}

/// FSL FR-621's text rendering: one line per item,
/// `<request_index> <claim_name> <record label>` and the record's cause or
/// proof basis when it has one, then the document category.
fn analyze_text(document: &serde_json::Value, category: Category) -> String {
    let mut text = String::new();
    for item in document["items"].as_array().into_iter().flatten() {
        let index = item["request_index"].as_u64().map_or_else(
            || item["request_index"].to_string(),
            |index| index.to_string(),
        );
        let name = item["claim_name"]
            .as_str()
            .filter(|name| !name.is_empty())
            .unwrap_or("-");
        let record = &item["record"];
        // An `invalid-request` disposition record is labelled `invalid-request`
        // (FSL FR-621), whether it carries no `result` or the `declined`
        // result whose cause is `invalid-request`.
        let invalid_request = record.as_object().is_some_and(|fields| {
            !fields.contains_key("result") || record["cause"]["code"] == "invalid-request"
        });
        let label = if invalid_request {
            "invalid-request"
        } else {
            record["result"].as_str().unwrap_or("-")
        };
        text.push_str(&format!("{index} {name} {label}"));
        if let Some(detail) = analyze_detail(record) {
            text.push(' ');
            text.push_str(&detail);
        }
        text.push('\n');
    }
    text.push_str(category.as_str());
    text.push('\n');
    text
}

/// The record's disposition diagnostic, cause or proof basis, compactly.
fn analyze_detail(record: &serde_json::Value) -> Option<String> {
    if let Some(code) = record["diagnostic"]["code"].as_str() {
        return Some(match record["diagnostic"]["cause"].as_str() {
            Some(cause) => format!("{code}/{cause}"),
            None => code.to_owned(),
        });
    }
    if let Some(cause) = record.get("cause").filter(|cause| !cause.is_null()) {
        return Some(compact_json(cause));
    }
    let basis = record.get("basis").filter(|basis| !basis.is_null())?;
    Some(
        basis["type"]
            .as_str()
            .map_or_else(|| compact_json(basis), str::to_owned),
    )
}

fn compact_json(value: &serde_json::Value) -> String {
    ix_cli_kit::json::encode_canonical(value, false).unwrap_or_else(|_| value.to_string())
}

/// The operands of `fsl monitor` (FR-025): one `native-monitor/1` request file
/// and the record stream's source, `--input <file>` or standard input.
struct MonitorOptions {
    path: PathBuf,
    input: Option<PathBuf>,
}

impl MonitorOptions {
    const USAGE: &str = "usage: fsl monitor <request-file> [--input <file>] [--format json|text]";

    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut input = None;
        let mut index = 0;
        while index < arguments.len() {
            let argument = &arguments[index];
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument == "--provider-config" {
                return Err(
                    "`--provider-config` is not available for `monitor` yet: configured \
                     providers are not built (QSL-2132, FSL FR-635)"
                        .to_owned(),
                );
            }
            if argument == "--input" {
                if input.is_some() {
                    return Err("--input may be supplied only once".to_owned());
                }
                let value = arguments
                    .get(index + 1)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "--input requires a path".to_owned())?;
                input = Some(PathBuf::from(value));
                index += 2;
                continue;
            }
            if let Some(option) = argument.to_str().filter(|value| value.starts_with('-')) {
                return Err(format!("unknown monitor option `{option}`"));
            }
            positionals.push(argument);
            index += 1;
        }
        let [path] = positionals.as_slice() else {
            return Err(Self::USAGE.to_owned());
        };
        Ok(Self {
            path: PathBuf::from(*path),
            input,
        })
    }
}

/// Writes monitor lines to standard output as they arrive. A closed pipe ends
/// the writing quietly; any other failure is kept for the exit code.
struct MonitorOutput {
    format: OutputFormat,
    closed: bool,
    failure: Option<io::Error>,
}

impl MonitorOutput {
    fn line(&mut self, value: &serde_json::Value, text: impl FnOnce() -> String) {
        if self.closed || self.failure.is_some() {
            return;
        }
        let mut line = if self.format == OutputFormat::Json {
            match ix_cli_kit::json::encode_canonical(value, false) {
                Ok(encoded) => encoded,
                Err(error) => {
                    self.failure = Some(io::Error::other(error.to_string()));
                    return;
                }
            }
        } else {
            text()
        };
        line.push('\n');
        match streams::write_primary_stdout(line.as_bytes()) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => self.closed = true,
            Err(error) => self.failure = Some(error),
        }
    }
}

fn monitor_status(settled: &serde_json::Value) -> &'static str {
    if settled == &serde_json::Value::Bool(true) {
        "settled"
    } else {
        "pending"
    }
}

fn monitor_event_text(event: &serde_json::Value) -> String {
    format!(
        "{} {} {} {}",
        event["record"],
        event["clause"].as_str().unwrap_or("-"),
        event["category"].as_str().unwrap_or("-"),
        monitor_status(&event["settled"]),
    )
}

fn monitor_summary_text(summary: &serde_json::Value) -> String {
    let mut lines: Vec<String> = summary["clauses"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|clause| {
            format!(
                "{} {} {}",
                clause["clause"].as_str().unwrap_or("-"),
                clause["category"].as_str().unwrap_or("-"),
                monitor_status(&clause["settled"]),
            )
        })
        .collect();
    lines.push(format!(
        "records {} refused {}",
        summary["records"], summary["refused"]
    ));
    lines.join("\n")
}

/// Run the `native-monitor/1` request over its record stream, writing each
/// event to standard output before the next record is read, then the summary.
/// The exit code is FSL's. A refusal before any record is read is the
/// command-error envelope.
fn monitor(options: &MonitorOptions, format: OutputFormat) -> CommandResult {
    let reader: Box<dyn io::BufRead> = match &options.input {
        Some(path) => match File::open(path) {
            Ok(file) => Box::new(io::BufReader::new(file)),
            Err(error) => {
                return CommandResult::input_refusal(
                    Operation::Monitor,
                    format!("could not open --input `{}`: {error}", path.display()),
                )
            }
        },
        None => Box::new(io::BufReader::new(io::stdin())),
    };
    let mut output = MonitorOutput {
        format,
        closed: false,
        failure: None,
    };
    let summary = command::monitor(
        &options.path,
        io::BufRead::split(reader, b'\n'),
        &mut |event| output.line(event.as_value(), || monitor_event_text(event.as_value())),
        RunLimits::default(),
        &Cancel::new(),
    );
    let summary = match summary {
        Ok(summary) => summary,
        Err(error) => {
            return CommandResult::document(error.outcome_document(Operation::Monitor));
        }
    };
    output.line(summary.as_value(), || {
        monitor_summary_text(summary.as_value())
    });
    if let Some(error) = output.failure {
        return CommandResult::failure(
            Category::InternalFailure,
            format!("could not write the monitor stream: {error}"),
        );
    }
    let Some(category) = Category::ALL
        .into_iter()
        .find(|category| category.exit_code() == summary.exit_code)
    else {
        return CommandResult::failure(
            Category::InternalFailure,
            format!(
                "the monitor exit code {} names no category",
                summary.exit_code
            ),
        );
    };
    CommandResult {
        category,
        document: None,
        diagnostic: None,
        text_output: None,
        configured_run: None,
        analyze_document: None,
        stderr_line: None,
    }
}

/// Dispatch the root verb while preserving its FSL-owned result category.
fn dispatch(verb: Verb, format: OutputFormat, raw_arguments: &[OsString]) -> CommandResult {
    if verb == Verb::Analyze {
        return match AnalyzeOptions::parse(raw_arguments) {
            Ok(options) => analyze(&options, format),
            Err(error) => CommandResult::input_refusal(Operation::Analyze, error),
        };
    }
    if verb == Verb::Monitor {
        return match MonitorOptions::parse(raw_arguments) {
            Ok(options) => monitor(&options, format),
            Err(error) => CommandResult::input_refusal(Operation::Monitor, error),
        };
    }
    if verb == Verb::Run {
        return match RunOptions::parse(raw_arguments) {
            Ok(_options) if format == OutputFormat::Text => {
                text_rendering_unavailable(Operation::Execute, "run")
            }
            Ok(options) => run(options),
            Err(error) => CommandResult::input_refusal(Operation::Execute, error),
        };
    }
    if verb == Verb::Compile {
        return match CompileOptions::parse(raw_arguments) {
            Ok(_options) if format == OutputFormat::Text => {
                text_rendering_unavailable(Operation::Package, "compile")
            }
            Ok(options) => compile(options),
            Err(error) => CommandResult::input_refusal(Operation::Package, error),
        };
    }
    if verb == Verb::Check {
        return match CheckOptions::parse(raw_arguments) {
            Ok(_options) if format == OutputFormat::Text => {
                text_rendering_unavailable(Operation::Check, "check")
            }
            Ok(options) => check_source(options),
            Err(error) => CommandResult::input_refusal(Operation::Check, error),
        };
    }
    if verb == Verb::Parse {
        return match SourceOptions::parse(verb, raw_arguments) {
            Ok(options) if format == OutputFormat::Text => {
                if let Some(refusal) = source_preflight(
                    &options,
                    Operation::Parse,
                    SpineLimits::default().source.source_bytes,
                ) {
                    refusal
                } else {
                    text_rendering_unavailable(Operation::Parse, "parse")
                }
            }
            Ok(options) => parse_source(options),
            Err(error) => CommandResult::input_refusal(Operation::Parse, error),
        };
    }
    if verb == Verb::Format {
        return match SourceOptions::parse(verb, raw_arguments) {
            Ok(options) => source_preflight(
                &options,
                Operation::Format,
                fsl_cst::Limits::default().source_bytes,
            )
            .unwrap_or_else(|| format_source(options)),
            Err(error) => CommandResult::input_refusal(Operation::Format, error),
        };
    }
    if raw_arguments.iter().any(|argument| {
        matches!(
            argument.to_str(),
            Some("--engine" | "--limit" | "--provider-config")
        )
    }) {
        let option = raw_arguments
            .iter()
            .find_map(|argument| match argument.to_str() {
                Some(option @ ("--engine" | "--limit" | "--provider-config")) => Some(option),
                _ => None,
            })
            .unwrap_or("--engine or --limit");
        return CommandResult::failure(
            Category::Refusal,
            format!("option `{option}` is only accepted by `run`"),
        );
    }
    CommandResult {
        category: Category::Unsupported,
        document: None,
        diagnostic: None,
        text_output: None,
        configured_run: None,
        analyze_document: None,
        stderr_line: None,
    }
}

/// Refuse a text selection whose FSL outcome renderer is not available.
/// Machine mode remains fully FSL-serialized; the diagnostic includes the
/// native FSL code because text mode has no outcome-document renderer.
fn text_rendering_unavailable(operation: Operation, verb: &str) -> CommandResult {
    let code = Code::UnimplementedCapability;
    CommandResult::catalog_failure_for(
        operation,
        code,
        format!(
            "text rendering for `{verb}` is unavailable ({})",
            code.as_str()
        ),
    )
}

/// Emit one result document with the kit's canonical JSON and stream APIs.
fn emit_result(result: &CommandResult, format: OutputFormat) -> Category {
    let primary = if format == OutputFormat::Json {
        let encoded = match &result.analyze_document {
            Some(document) => ix_cli_kit::json::encode_canonical(document, false).map(Some),
            None => result
                .document
                .as_ref()
                .map(|document| ix_cli_kit::json::encode_canonical(document, false))
                .transpose(),
        };
        match encoded {
            Ok(encoded) => encoded.map(|mut encoded| {
                encoded.push('\n');
                encoded
            }),
            Err(error) => {
                emit_diagnostic(
                    "OutputFailure",
                    &format!("could not encode outcome document: {error}"),
                );
                return Category::InternalFailure;
            }
        }
    } else {
        result.text_output.clone()
    };
    let mut category = result.category;
    if let Some(primary) = primary {
        match streams::write_primary_stdout(primary.as_bytes()) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => return result.category,
            Err(error) => {
                emit_diagnostic(
                    "OutputFailure",
                    &format!(
                        "OutputFailure: could not write {}: {error}",
                        if format == OutputFormat::Json {
                            "outcome document"
                        } else {
                            "formatted source"
                        }
                    ),
                );
                category = Category::InternalFailure;
            }
        }
    }
    let mut stderr = io::stderr().lock();
    let _ = write_result_diagnostics(result, &mut stderr);
    category
}

pub(super) fn entrypoint() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    match parse_invocation(&arguments) {
        Ok(Invocation::Help) => help_exit_code(),
        Ok(Invocation::Version) => version_exit_code(),
        Ok(Invocation::Operation {
            verb,
            format,
            raw_arguments,
        }) => {
            // Check the exit invariant before output so a rejected reserved
            // result cannot accompany an outcome document.
            let result = dispatch(verb, format, &raw_arguments);
            let exit_result = category_exit_result(result.category);
            if exit_result == RESERVED_COMMAND_NOT_FOUND {
                if format == OutputFormat::Text {
                    let mut stderr = io::stderr().lock();
                    let _ = write_result_diagnostics(&result, &mut stderr);
                }
                process_exit_code(result.category)
            } else {
                process_exit_code(emit_result(&result, format))
            }
        }
        Err(message) => {
            let result = CommandResult::failure(Category::Refusal, message);
            process_exit_code(emit_result(&result, OutputFormat::Text))
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod outcome_tests {
    use std::io;
    use std::process::ExitCode;

    use filament_semantic_value::catalog::Category;
    use fsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};

    use super::{process_exit_code, write_diagnostic, write_result_diagnostics, CommandResult};

    /// Trace: FR-002-AC-2, FR-002-AC-4; one FSL exit result reaches process exit unchanged through the kit.
    #[test]
    fn process_code_is_fsl_category_exit_code_without_remapping() {
        let expected = [0, 10, 10, 20, 21, 22, 0, 11, 22, 30];
        assert_eq!(Category::ALL.len(), expected.len());
        for (category, expected_code) in Category::ALL.into_iter().zip(expected) {
            assert_eq!(process_exit_code(category), ExitCode::from(expected_code));
        }
    }

    struct ClosedPipe;

    impl io::Write for ClosedPipe {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }

    /// Trace: FR-002-AC-8; a closed diagnostic sink returns an error without panicking.
    #[test]
    fn diagnostic_write_reports_a_closed_stderr_pipe_without_panicking() {
        let mut sink = ClosedPipe;
        let error = write_diagnostic(&mut sink, "DriverDiagnostic", "refused request")
            .expect_err("closed pipe is reported to its caller");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }

    /// Trace: FR-002-AC-8; FSL document diagnostics are retained on stderr.
    #[test]
    fn typed_outcome_diagnostics_are_written_to_stderr() {
        let document = OutcomeDocument::new(Operation::Execute, None, Category::Refusal)
            .with_diagnostics(vec![OutcomeDiagnostic::new(
                None,
                "invalid-request",
                None,
                "the selected function has no parameter named x",
            )]);
        let result = CommandResult::document(document);
        let mut stderr = Vec::new();
        write_result_diagnostics(&result, &mut stderr).expect("write FSL diagnostic");
        assert_eq!(stderr, b"the selected function has no parameter named x\n");
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use super::{
        parse_invocation, read_source, CheckOptions, ExecutionEngine, Invocation, OutputFormat,
        RunOptions, Verb,
    };
    use fsl_foundation::{Setting, SettingName};
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    #[test]
    fn unavailable_limit_error_is_generic_and_cli_message_names_operation() {
        let error = filament_driver::LimitError::OperationUnavailable {
            setting: SettingName::Stage(Setting::ExploreStates),
        };
        assert_eq!(
            error.to_string(),
            "limit setting `explore.states` is unavailable for this operation"
        );
        assert_eq!(
            error.message_for("run"),
            "limit setting `explore.states` is unavailable for the `run` operation"
        );
    }

    /// Trace: FR-002-AC-1, root command surface.
    #[test]
    fn root_parser_recognizes_every_lifecycle_verb() {
        for verb in [
            "parse", "format", "check", "compile", "run", "analyze", "monitor", "prove", "replay",
            "generate", "inspect", "render",
        ] {
            let args = vec![OsString::from(verb)];
            assert!(
                matches!(parse_invocation(&args), Ok(Invocation::Operation { .. })),
                "{verb}"
            );
        }
    }

    /// Trace: FR-002-AC-1, inspect's graph format.
    #[test]
    fn inspect_accepts_dot_and_other_verbs_reject_it() {
        let inspect = vec![
            OsString::from("inspect"),
            OsString::from("--format"),
            OsString::from("dot"),
        ];
        assert!(matches!(
            parse_invocation(&inspect),
            Ok(Invocation::Operation {
                verb: Verb::Inspect,
                format: OutputFormat::Dot,
                ..
            })
        ));
        let check = vec![
            OsString::from("check"),
            OsString::from("--format"),
            OsString::from("dot"),
        ];
        assert!(parse_invocation(&check).is_err());
    }

    /// Trace: FR-002-AC-1
    #[test]
    fn accepts_shared_format_without_resolving_config_or_output() {
        let args = vec![
            OsString::from("check"),
            OsString::from("--format"),
            OsString::from("json"),
        ];
        assert!(matches!(
            parse_invocation(&args),
            Ok(Invocation::Operation {
                format: OutputFormat::Json,
                ..
            })
        ));
    }

    /// Trace: FR-002, preserve the raw operation arguments for handlers.
    #[test]
    fn retains_operation_arguments_at_the_dispatch_boundary() {
        let args = vec![
            OsString::from("run"),
            OsString::from("--limit"),
            OsString::from("steps=12"),
            OsString::from("unit.fsl"),
        ];
        assert!(matches!(
            parse_invocation(&args),
            Ok(Invocation::Operation { raw_arguments, .. }) if raw_arguments == args[1..]
        ));
    }

    /// Trace: QSL-1972 / FR-866-AC-5; check parses its named model and library inputs.
    #[test]
    fn check_options_accept_model_libraries_and_dependent_byte_limit() {
        let args = vec![
            OsString::from("unit.fsl"),
            OsString::from("--model"),
            OsString::from("model.json"),
            OsString::from("--library"),
            OsString::from("geometry.native"),
            OsString::from("--library"),
            OsString::from("units.native"),
            OsString::from("--dependent-bytes"),
            OsString::from("1234"),
        ];
        let options = CheckOptions::parse(&args).expect("valid check inputs");
        assert_eq!(options.path, "unit.fsl");
        assert_eq!(options.model.as_deref(), Some(Path::new("model.json")));
        assert_eq!(
            options.libraries,
            [
                PathBuf::from("geometry.native"),
                PathBuf::from("units.native")
            ]
        );
        assert_eq!(options.dependent_bytes, 1234);
    }

    /// Trace: QSL-1972 / FR-866-AC-5; malformed named check inputs refuse during argv parsing.
    #[test]
    fn check_options_reject_missing_or_duplicate_model_and_invalid_byte_limit() {
        for args in [
            vec![OsString::from("unit.fsl"), OsString::from("--model")],
            vec![OsString::from("unit.fsl"), OsString::from("--library")],
            vec![
                OsString::from("unit.fsl"),
                OsString::from("--dependent-bytes"),
                OsString::from("many"),
            ],
            vec![
                OsString::from("unit.fsl"),
                OsString::from("--model"),
                OsString::from("one.json"),
                OsString::from("--model"),
                OsString::from("two.json"),
            ],
        ] {
            assert!(CheckOptions::parse(&args).is_err(), "accepted {args:?}");
        }
    }

    /// Trace: FR-003-AC-1 / FR-004-AC-1, parse both options into the run call.
    #[test]
    fn run_options_keep_the_requested_engine_and_limits() {
        let args = vec![
            OsString::from("unit.fsl"),
            OsString::from("--engine"),
            OsString::from("interpreter"),
            OsString::from("--limit"),
            OsString::from("s1.input_bytes=1024"),
        ];
        let options = RunOptions::parse(&args).expect("valid run options");
        assert_eq!(options.path, std::path::Path::new("unit.fsl"));
        assert_eq!(options.engine, ExecutionEngine::Interpreter);
        assert_eq!(
            options.limits.values(),
            &[(
                fsl_foundation::SettingName::Stage(fsl_foundation::Setting::S1InputBytes),
                1024,
            )]
        );
        assert_eq!(options.provider_config, None);
    }

    /// Trace: FR-006-AC-24, FR-014-AC-6; accept only one explicitly named config path.
    #[test]
    fn run_options_accept_one_provider_config_path() {
        let args = vec![
            OsString::from("unit.fsl"),
            OsString::from("--provider-config"),
            OsString::from("providers.json"),
        ];
        let options = RunOptions::parse(&args).expect("valid provider config path");
        assert_eq!(
            options.provider_config.as_deref(),
            Some(std::path::Path::new("providers.json"))
        );
    }

    /// Trace: FR-006-AC-24, FR-014-AC-6; malformed caller argv is rejected before loading.
    #[test]
    fn run_options_reject_missing_or_duplicate_provider_config_path() {
        let missing = vec![
            OsString::from("unit.fsl"),
            OsString::from("--provider-config"),
        ];
        assert!(RunOptions::parse(&missing)
            .err()
            .expect("provider config path is required")
            .contains("--provider-config requires a path"));

        let duplicate = vec![
            OsString::from("unit.fsl"),
            OsString::from("--provider-config"),
            OsString::from("one.json"),
            OsString::from("--provider-config"),
            OsString::from("two.json"),
        ];
        assert!(RunOptions::parse(&duplicate)
            .err()
            .expect("provider config may be selected once")
            .contains("only once"));
    }

    /// Trace: FR-004-AC-2, an unknown engine fails before any source stage.
    #[test]
    fn run_options_refuse_unknown_engine_values() {
        let args = vec![
            OsString::from("unit.fsl"),
            OsString::from("--engine"),
            OsString::from("gpu"),
        ];
        assert!(RunOptions::parse(&args).is_err());
    }

    /// Trace: FR-003-AC-3, the bounded read includes input exactly at its limit.
    #[test]
    fn bounded_source_read_accepts_bytes_at_the_limit() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.fsl");
        let expected = std::fs::read(path).expect("fixture source");
        assert_eq!(
            read_source(path, expected.len()).expect("source at limit"),
            expected
        );
    }

    /// Trace: FR-003-AC-3, over-limit read buffers only the one-byte sentinel.
    #[test]
    fn bounded_source_read_captures_one_sentinel_byte_over_limit() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.fsl");
        let expected = std::fs::read(path).expect("fixture source");
        let limit = 8;
        assert!(
            expected.len() > limit * 10,
            "fixture exceeds the buffer bound"
        );
        let actual = read_source(path, limit).expect("source over limit");
        assert_eq!(
            actual.len(),
            limit + 1,
            "read is bounded to the limit plus one sentinel byte"
        );
        assert_eq!(actual, expected[..=limit]);
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "configured provider integration assertions are the test failure report"
)]
mod configured_run_tests {
    use std::path::{Path, PathBuf};

    use filament_exact::{Integer, Value};
    use fsl_foundation::SourceIdentity;
    use fsl_replay::spine::{
        default_accounting, Call, CallArgument, SpineLimits, DEFAULT_WORK_UNITS,
    };

    use super::{run_configured_providers, Code, CommandResult, ConfiguredRunRequest};

    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn python_path() -> PathBuf {
        std::env::split_paths(&std::env::var_os("PATH").expect("PATH is set"))
            .map(|directory| directory.join("python3"))
            .find(|candidate| candidate.is_file())
            .expect("the process fixture requires Python 3")
    }

    /// Trace: FR-006-AC-1, FR-006-AC-3, FR-006-AC-8, FR-006-AC-10..FR-006-AC-11, FR-006-AC-15, FR-006-AC-25.
    #[test]
    fn configured_run_invokes_only_the_selected_process_and_retains_its_typed_result() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        let directory = TempDir(std::env::temp_dir().join(format!(
            "fsl-cli-configured-run-{}-{unique}",
            std::process::id()
        )));
        std::fs::create_dir(&directory.0).expect("create temporary test directory");
        let bytes = format!(
            "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{}\";\n\
             function eq using v(x: Int[0, 9], y: Int[0, 9]): Boolean pure {{ x = y }}\n\
             function negate using v(z: Int[0, 9]): Int[-9, 0] pure {{ -z }}\n",
            "filament.value.complete/v1"
        );
        let path = "configured-run.native";
        let source = SourceIdentity::new("agent-ix", path);
        let packages = std::collections::BTreeMap::new();
        let dependencies = fsl_replay::spine::DependencyInput::default();
        let limits = SpineLimits::default();
        let call = Call {
            function: "eq".to_owned(),
            arguments: vec![
                CallArgument {
                    parameter: "x".to_owned(),
                    value: Value::Integer(Integer::from(2_i64)),
                },
                CallArgument {
                    parameter: "y".to_owned(),
                    value: Value::Integer(Integer::from(3_i64)),
                },
            ],
            accounting: default_accounting(DEFAULT_WORK_UNITS),
            supplied: Default::default(),
            backend: Default::default(),
            clocks: Default::default(),
            wire_arguments: Vec::new(),
            snapshot: None,
        };
        let executable = python_path();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/configured-fr331-provider.py");
        let identities = [
            "agent-ix/provider/configured-alpha",
            "agent-ix/provider/configured-zeta",
        ];
        let capabilities = ["value-validity", "operation-contract"];
        let markers = identities
            .iter()
            .enumerate()
            .map(|(index, _)| directory.0.join(format!("provider-{index}.json")))
            .collect::<Vec<_>>();
        let providers = identities
            .iter()
            .zip(&markers)
            .zip(capabilities)
            .map(
                |((identity, marker), capability)| crate::provider_config::RuntimeProviderConfig {
                    executable: executable.clone(),
                    argv: vec![
                        fixture.display().to_string(),
                        (*identity).to_owned(),
                        marker.display().to_string(),
                        capability.to_owned(),
                    ],
                    deadline_ms: 4096,
                    limits: crate::provider_config::RuntimeProviderLimitsConfig {
                        max_frame_bytes: 262_144,
                        max_total_bytes: 1_048_576,
                        max_stderr_bytes: 4096,
                        cancellation_grace_ms: 500,
                    },
                },
            )
            .collect();

        let configured = match run_configured_providers(ConfiguredRunRequest {
            path,
            package_name: path,
            bytes: bytes.as_bytes(),
            source,
            limits,
            call,
            packages,
            dependencies,
            config: providers,
            signals: crate::signals::SignalContext::default(),
        }) {
            Ok(configured) => configured,
            Err(failure) => panic!("configured provider flow failed: {failure:?}"),
        };
        assert_eq!(configured.call.requirements.members().len(), 1);
        assert_eq!(configured.driven.generation().items.len(), 2);
        assert_eq!(configured.provider_results.len(), 1);
        let invoked = markers
            .iter()
            .enumerate()
            .filter(|(_, marker)| marker.is_file())
            .collect::<Vec<_>>();
        assert_eq!(
            invoked.len(),
            1,
            "only the selected provider receives a request"
        );
        let (selected, marker) = invoked[0];
        let request: serde_json::Value = serde_json::from_slice(
            &std::fs::read(marker).expect("the selected provider recorded its request"),
        )
        .expect("the provider request is JSON");
        let member = &configured.call.requirements.members()[0];
        assert_eq!(request["request"]["deadline_ms"], 4096);
        assert_eq!(
            request["request"]["items"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            request["request"]["items"][0]["backend"],
            identities[selected]
        );
        assert_eq!(
            request["request"]["items"][0]["request_index"],
            member.request_index().get()
        );
        assert_eq!(request["request"]["items"][0]["kind"], "value-validity");
        assert!(request["request"]["items"][0]["claim"].is_object());
        assert_eq!(request["request"]["items"][0]["candidates"]["type"], "set");
        assert!(request["request"]["items"][0]["candidates"]["backends"]
            .as_array()
            .expect("candidate set is an array")
            .iter()
            .any(|candidate| candidate == identities[selected]));
        assert!(request["request"]["items"][0]["extent"].is_object());

        let command = CommandResult::configured(Ok(configured));
        assert!(
            command.document.is_some(),
            "existing FSL call result is retained"
        );
        assert!(
            command
                .document
                .as_ref()
                .expect("FSL call document remains available")
                .items()
                .is_empty(),
            "provider results are not fabricated as FSL terminals"
        );
        let Some(Ok(retained)) = command.configured_run.as_ref() else {
            panic!("typed configured result remains owned by the command result");
        };
        assert_eq!(retained.provider_results.len(), 1);
        assert_eq!(retained.provider_results[0].items.len(), 1);
        let outcome = retained.provider_results[0].items[0]
            .result
            .as_ref()
            .expect("the fixture returned a typed provider result");
        assert!(matches!(
            &outcome.item,
            crate::process_provider::ProcessProviderItemOutcome::Result {
                kind: crate::process_provider::Fr331ResultKind::Tested,
                ..
            }
        ));
        assert_eq!(retained.diagnostics.len(), 2);
        assert!(retained
            .diagnostics
            .iter()
            .all(|finished| String::from_utf8_lossy(&finished.stderr)
                .contains("configured provider completed")));
        assert_eq!(retained.call.package_name, retained.driven.emitted().name());
    }

    /// Trace: FR-006-AC-26; identical admitted hellos fail closed before negotiation and close both sessions.
    #[test]
    fn configured_run_refuses_idempotent_duplicate_identities_before_dispatch() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        let directory = TempDir(std::env::temp_dir().join(format!(
            "fsl-cli-configured-duplicate-{}-{unique}",
            std::process::id()
        )));
        std::fs::create_dir(&directory.0).expect("create temporary test directory");
        let bytes = format!(
            "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{}\";\n\
             function eq using v(x: Int[0, 9], y: Int[0, 9]): Boolean pure {{ x = y }}\n\
             function negate using v(z: Int[0, 9]): Int[-9, 0] pure {{ -z }}\n",
            "filament.value.complete/v1"
        );
        let path = "configured-duplicate.native";
        let source = SourceIdentity::new("agent-ix", path);
        let call = Call {
            function: "eq".to_owned(),
            arguments: vec![
                CallArgument {
                    parameter: "x".to_owned(),
                    value: Value::Integer(Integer::from(2_i64)),
                },
                CallArgument {
                    parameter: "y".to_owned(),
                    value: Value::Integer(Integer::from(3_i64)),
                },
            ],
            accounting: default_accounting(DEFAULT_WORK_UNITS),
            supplied: Default::default(),
            backend: Default::default(),
            clocks: Default::default(),
            wire_arguments: Vec::new(),
            snapshot: None,
        };
        let executable = python_path();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/configured-fr331-provider.py");
        let identity = "agent-ix/provider/configured-duplicate";
        let markers = [
            directory.0.join("duplicate-one.json"),
            directory.0.join("duplicate-two.json"),
        ];
        let shutdown_markers = [
            directory.0.join("duplicate-one-shutdown"),
            directory.0.join("duplicate-two-shutdown"),
        ];
        let providers = markers
            .iter()
            .zip(shutdown_markers.iter())
            .map(
                |(marker, shutdown)| crate::provider_config::RuntimeProviderConfig {
                    executable: executable.clone(),
                    argv: vec![
                        fixture.display().to_string(),
                        identity.to_owned(),
                        marker.display().to_string(),
                        "value-validity".to_owned(),
                        "normal".to_owned(),
                        shutdown.display().to_string(),
                    ],
                    deadline_ms: 4096,
                    limits: crate::provider_config::RuntimeProviderLimitsConfig {
                        max_frame_bytes: 262_144,
                        max_total_bytes: 1_048_576,
                        max_stderr_bytes: 4096,
                        cancellation_grace_ms: 500,
                    },
                },
            )
            .collect();

        let failure = match run_configured_providers(ConfiguredRunRequest {
            path,
            package_name: path,
            bytes: bytes.as_bytes(),
            source,
            limits: SpineLimits::default(),
            call,
            packages: std::collections::BTreeMap::new(),
            dependencies: fsl_replay::spine::DependencyInput::default(),
            config: providers,
            signals: crate::signals::SignalContext::default(),
        }) {
            Err(failure) => failure,
            Ok(_) => panic!("idempotent duplicate identities are an input refusal"),
        };
        assert!(matches!(
            failure.kind,
            super::ConfiguredRunErrorKind::DuplicateIdentity { ref backend }
                if backend == identity
        ));
        assert_eq!(
            failure.document.category(),
            filament_semantic_value::catalog::Category::Refusal
        );
        assert!(failure
            .document
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == Code::InvalidRequest.as_str()));
        assert_eq!(failure.cleanup_diagnostics.len(), 2);
        assert!(failure.cleanup_failures.is_empty());
        assert!(markers.iter().all(|marker| !marker.exists()));
        assert!(shutdown_markers.iter().all(|marker| marker.is_file()));
    }
}

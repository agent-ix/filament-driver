// SPDX-License-Identifier: AGPL-3.0-or-later
//! Parser and command handlers for the user-facing `fsl` binary.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use filament_driver::{
    build_linked_providers, execute, ExecutionEngine, ExecutionFailure, ExecutionRequest,
    LimitOverrides, ObjectEnvironment, ProcessProvider, ProcessProviderLimits,
    ProcessProviderRequest, ProviderResults,
};
use filament_exact::Cancel;
use filament_semantic_value::catalog::Category;
use fsl_command::command::{self, PreparedRun, RunLimits};
use fsl_cst::format::{format_with_limits, FormatLimits, FormatRefusal};
use fsl_foundation::diagnostic::Code;
use fsl_foundation::diagnostic::Staged;
use fsl_foundation::source::{SourceReadCause, SourceReadError, SourceReadRefusal};
use fsl_foundation::wire_format::WireFormat;
use fsl_foundation::{Setting, SettingLimits, SettingName, Source, SourceIdentity};
use fsl_replay::spine::{
    check_native_fences, parse, CheckLimits, CompileRefusal, DependencyInput, NativeFenceRequest,
    ParseRequest, SpineLimits,
};
use fsl_replay::RunRefusal;
use fsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument, OutcomeStage};
use ix_cli_kit::exit::{caller_exit_code, RESERVED_COMMAND_NOT_FOUND};
use ix_cli_kit::streams::{self, Diagnostics};

use crate::provider_config::{OptionRefusal, ProviderArguments};

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
        "  compile <request-file> [--clocks <json-file>] [--output <file>] [--limit name=value]\n",
        "      [--format json|text]\n",
        "  run <request-file> [--request-bytes <bytes>] [--dependent-bytes <bytes>]\n",
        "      [--limit name=value] [--engine interpreter|aot|jit] [--provider-config <path>]\n",
        "      [--clocks <json-file>] [--format json|text]\n",
        "  replay <request-file> [--provider-config <path>] [--provider-limit name=value]\n",
        "      [--format json|text]\n",
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
            Self::Check | Self::Parse | Self::Compile | Self::Run => {
                "Executes and writes its outcome document as text by default; --format json selects JSON."
            }
            Self::Format => "Executes source formatting; text output is the default.",
            Self::Analyze => "Executes a native-analyze/1 request; --format json or text.",
            Self::Monitor => {
                "Executes a native-monitor/1 request over a record stream; --format json or text."
            }
            Self::Replay => "Executes a native-replay/1 request; --format json or text.",
            Self::Prove | Self::Generate | Self::Inspect | Self::Render => {
                "Recognized; this build does not route it and returns an unsupported_operation result."
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
    clocks: Option<PathBuf>,
    /// `--provider-config` and `--provider-limit` (FSL FR-287; FR-006).
    providers: crate::provider_config::ProviderArguments,
    /// The driver-side run options FSL's `run_with_providers` does not take,
    /// in the order the caller gave them.
    local_options: Vec<&'static str>,
}

type CliProviderError = crate::process_provider::AdapterError<
    crate::backend_provider::ManifestAdmissionError,
    String,
    serde_json::Value,
>;
pub(crate) type CliProviderEncode =
    for<'a> fn(ProcessProviderRequest<'a>) -> Result<crate::backend_provider::RequestStage, String>;
pub(crate) type CliProviderValidate =
    fn(&crate::backend_provider::ManifestStage, &serde_json::Value) -> Result<(), String>;
pub(crate) type CliProviderConvert =
    fn(
        serde_json::Value,
    )
        -> Result<ProviderResults<crate::process_provider::ProcessProviderOutcome, String>, String>;
pub(crate) type CliProviderSession = crate::process_provider::HostProcessSession<
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

struct CompileOptions {
    path: PathBuf,
    clocks: Option<PathBuf>,
    output: Option<PathBuf>,
    limits: LimitOverrides,
}

struct CheckOptions {
    path: String,
    source_identity: Option<SourceIdentity>,
    markdown: bool,
    model: Option<PathBuf>,
    libraries: Vec<PathBuf>,
    dependent_bytes: usize,
    limits: LimitOverrides,
    /// The `--integer-division <law>` value, unread until the check runs
    /// (FSL FR-767).
    integer_division: Option<String>,
}

/// The integer-division law `value` names, admitted as FSL FR-767 states:
/// exactly `euclidean`, `floor` or `truncating`; any other value refuses
/// `unknown_profile`/`unsupported-selection` naming `source` and the value.
fn admit_integer_division_law(
    value: &str,
    source: &str,
) -> Result<fsl_replay::AdmittedIntegerDivision, Box<CommandResult>> {
    use fsl_replay::DefinitionRole;
    let role = match value {
        "euclidean" => DefinitionRole::IntegerDivisionEuclidean,
        "floor" => DefinitionRole::IntegerDivisionFloor,
        "truncating" => DefinitionRole::IntegerDivisionTruncating,
        _ => {
            let message = format!(
                "integer-division law `{value}` supplied by `{source}` is not one of \
                 `euclidean`, `floor`, `truncating`"
            );
            return Err(Box::new(
                CommandResult::document(
                    OutcomeDocument::new(Operation::Check, None, Code::UnknownProfile.category())
                        .with_diagnostics(vec![OutcomeDiagnostic::new(
                            Some("unsupported-selection"),
                            Code::UnknownProfile.as_str(),
                            None,
                            message.clone(),
                        )]),
                )
                .with_diagnostic(message),
            ));
        }
    };
    fsl_replay::admit_integer_division(&[role.reference()], None).map_err(|refusal| {
        Box::new(CommandResult::failure(
            Category::InternalFailure,
            format!("FSL did not admit its own `{value}` integer-division law: {refusal:?}"),
        ))
    })
}

struct SourceOptions {
    source: SourceIdentity,
    path: PathBuf,
    limits: LimitOverrides,
}

impl SourceOptions {
    fn parse(verb: Verb, arguments: &[OsString]) -> Result<Self, String> {
        let limit_usage = if matches!(verb, Verb::Parse | Verb::Format) {
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
            if argument == "--limit" && matches!(verb, Verb::Parse | Verb::Format) {
                let value = arguments
                    .get(index + 1)
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| "--limit requires a setting=value operand".to_owned())?;
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
        let cst_limits = fsl_cst::Limits::default();
        let mut accepted_bounds = cst_limits.bounds();
        if verb == Verb::Format {
            accepted_bounds.extend(FormatLimits::default().bounds());
        }
        let limits = parse_frontend_limit_operands_with_bounds(&limit_operands, accepted_bounds)?;
        match verb {
            Verb::Parse => {
                let mut cst_limits = cst_limits;
                limits
                    .apply_to_limits(&mut cst_limits, "parse")
                    .map_err(|error| error.message_for("parse"))?;
            }
            Verb::Format => {
                let mut cst_limits = cst_limits;
                let mut format_limits = FormatLimits::default();
                limits
                    .apply_to_setting_group(&mut [&mut cst_limits, &mut format_limits], "format")
                    .map_err(|error| error.message_for("format"))?;
            }
            _ => {}
        }
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
        let mut limit_operands = Vec::new();
        let mut index = 0;
        while index < arguments.len() {
            let argument = &arguments[index];
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument == "--limit" {
                let value = arguments
                    .get(index + 1)
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| "--limit requires a setting=value operand".to_owned())?;
                limit_operands.push(value.to_owned());
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
            if argument == "--limit" {
                let value = arguments
                    .get(index + 1)
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| "--limit requires name=value".to_owned())?;
                limit_operands.push(value.to_owned());
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
                "usage: fsl compile <request-file> [--clocks <json-file>] [--output <file>] [--limit name=value] [--format json|text]"
                    .to_owned(),
            );
        };
        let limits = parse_frontend_limit_operands(&limit_operands, &SpineLimits::default())?;
        Ok(Self {
            path: PathBuf::from(*path),
            clocks,
            output,
            limits,
        })
    }
}

impl CheckOptions {
    const USAGE: &str = "usage: fsl check [--markdown] <source-path> | [--markdown] <authority> <identity> <source-path> [--model <file>] [--library <file>]... [--dependent-bytes <bytes>] [--limit name=value] [--integer-division euclidean|floor|truncating] [--format json|text]";

    fn parse(arguments: &[OsString]) -> Result<Self, OptionRefusal> {
        let mut positionals = Vec::new();
        let mut markdown = false;
        let mut integer_division = None;
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
            if argument == "--markdown" {
                if markdown {
                    return Err(OptionRefusal::Quiet(Self::USAGE.to_owned()));
                }
                markdown = true;
                index += 1;
                continue;
            }
            if argument == "--model" {
                let path = arguments
                    .get(index + 1)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "--model requires a path".to_owned())?;
                if model.replace(PathBuf::from(path)).is_some() {
                    return Err("--model may be supplied only once".to_owned().into());
                }
                index += 2;
                continue;
            }
            if argument == "--integer-division" {
                let law = arguments
                    .get(index + 1)
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| "--integer-division requires a law".to_owned())?;
                if integer_division.replace(law.to_owned()).is_some() {
                    return Err(OptionRefusal::Quiet(Self::USAGE.to_owned()));
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
                    return Err("--dependent-bytes may be supplied only once"
                        .to_owned()
                        .into());
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
                return Err(format!("unknown check option `{argument}`").into());
            }
            positionals.push(argument.to_owned());
            index += 1;
        }
        let (source_identity, path) = match positionals.as_slice() {
            [path] => (None, path.clone()),
            [authority, identity, path] => {
                (Some(SourceIdentity::new(authority, identity)), path.clone())
            }
            _ => return Err(Self::USAGE.to_owned().into()),
        };
        if markdown && source_identity.is_none() {
            return Err(Self::USAGE.to_owned().into());
        }
        let limits = parse_frontend_limit_operands(&limit_operands, &SpineLimits::default())?;
        Ok(Self {
            path,
            source_identity,
            markdown,
            model,
            libraries,
            dependent_bytes,
            limits,
            integer_division,
        })
    }
}

fn parse_frontend_limit_operands(
    operands: &[String],
    bounds: &impl SettingLimits,
) -> Result<LimitOverrides, String> {
    parse_frontend_limit_operands_with_bounds(operands, bounds.bounds())
}

fn parse_frontend_limit_operands_with_bounds(
    operands: &[String],
    bounds: Vec<(Setting, u64)>,
) -> Result<LimitOverrides, String> {
    LimitOverrides::parse(operands).map_err(|error| {
        let accepted = bounds
            .into_iter()
            .map(|(setting, _)| setting.name())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{error}; settings accepted by this command: {accepted}")
    })
}

fn accepted_frontend_limit_settings(bounds: &impl SettingLimits) -> String {
    bounds
        .bounds()
        .into_iter()
        .map(|(setting, _)| setting.name())
        .collect::<Vec<_>>()
        .join(", ")
}

fn apply_frontend_limits(
    overrides: &LimitOverrides,
    bounds: &mut impl SettingLimits,
    operation: &'static str,
) -> Result<(), String> {
    overrides.apply_to_limits(bounds, operation).map_err(|error| {
        let accepted = accepted_frontend_limit_settings(bounds);
        format!(
            "{}; settings accepted by this command: {accepted}",
            error.message_for(operation)
        )
    })
}

impl RunOptions {
    const USAGE: &str = "usage: fsl run <request-file> [--request-bytes <bytes>] [--dependent-bytes <bytes>] [--limit name=value] [--engine interpreter|aot|jit] [--clocks <json-file>] [--provider-config <path>] [--provider-limit name=value] [--format json|text]";

    fn parse(arguments: &[OsString]) -> Result<Self, OptionRefusal> {
        let mut positionals = Vec::new();
        let mut limit_operands = Vec::new();
        let mut engine = None;
        let mut intake_limits = RunLimits::default();
        let mut byte_options = BTreeSet::new();
        let mut clocks: Option<PathBuf> = None;
        let mut providers = crate::provider_config::ProviderArguments::default();
        let mut local_options = Vec::new();
        let mut index = 0;
        while index < arguments.len() {
            if let Some(taken) = providers.take(arguments, index)? {
                index += taken;
                continue;
            }
            let arg = arguments[index].to_str();
            if let Some(option) = [
                "--request-bytes",
                "--dependent-bytes",
                "--limit",
                "--clocks",
            ]
            .into_iter()
            .find(|option| arg == Some(*option))
            {
                local_options.push(option);
            }
            match arg {
                Some("--request-bytes" | "--dependent-bytes") => {
                    let option = arg.unwrap_or_default();
                    if !byte_options.insert(option) {
                        return Err(format!("{option} may be supplied only once").into());
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
                        return Err("--engine may be supplied only once".to_owned().into());
                    }
                    engine = Some(ExecutionEngine::parse(value)?);
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
                        return Err(format!("{} requires a value", arg.unwrap_or_default()).into());
                    }
                }
                Some(option) if option.starts_with('-') => {
                    return Err(format!("unknown run option `{option}`").into());
                }
                _ => positionals.push(arguments[index].clone()),
            }
            index += 1;
        }
        let [path] = positionals.as_slice() else {
            return Err(Self::USAGE.to_owned().into());
        };
        Ok(Self {
            path: PathBuf::from(path),
            intake_limits,
            limits: LimitOverrides::parse(&limit_operands).map_err(|error| error.to_string())?,
            engine: engine.unwrap_or_default(),
            clocks,
            providers,
            local_options,
        })
    }
}

/// One command's FSL category, optional outcome document, and diagnostic.
struct CommandResult {
    category: Category,
    document: Option<OutcomeDocument>,
    diagnostic: Option<String>,
    diagnostic_on_stderr: bool,
    text_output: Option<String>,
    /// The FSL FR-621 `analyze` document, written in machine mode in place
    /// of `document`.
    analyze_document: Option<serde_json::Value>,
    /// One plain stderr line, written in place of the document diagnostics.
    stderr_line: Option<String>,
    /// The admitted sources the command read, for locating text diagnostics (FR-865).
    render_sources: Option<fsl_inspect::RenderSources>,
}

impl CommandResult {
    fn document(document: OutcomeDocument) -> Self {
        Self {
            category: document.category(),
            document: Some(document),
            diagnostic: None,
            diagnostic_on_stderr: false,
            text_output: None,
            analyze_document: None,
            stderr_line: None,
            render_sources: None,
        }
    }

    fn failure(category: Category, message: impl Into<String>) -> Self {
        Self {
            category,
            document: None,
            diagnostic: Some(message.into()),
            diagnostic_on_stderr: true,
            text_output: None,
            analyze_document: None,
            stderr_line: None,
            render_sources: None,
        }
    }

    fn formatted(document: OutcomeDocument, text_output: Option<String>) -> Self {
        Self {
            category: document.category(),
            document: Some(document),
            diagnostic: None,
            diagnostic_on_stderr: false,
            text_output,
            analyze_document: None,
            stderr_line: None,
            render_sources: None,
        }
    }

    /// An input refusal raised before FSL runs, recorded with FSL's native code.
    fn input_refusal(operation: Operation, message: impl Into<String>) -> Self {
        let mut result = Self::catalog_failure_for(operation, Code::InvalidRequest, message);
        result.diagnostic_on_stderr = true;
        result
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

    fn with_sources(mut self, sources: fsl_inspect::RenderSources) -> Self {
        self.render_sources = Some(sources);
        self
    }

    fn with_diagnostic(mut self, message: impl Into<String>) -> Self {
        self.diagnostic = Some(message.into());
        self
    }
}

fn write_result_diagnostics<W: Write>(
    result: &CommandResult,
    format: OutputFormat,
    sink: &mut W,
) -> io::Result<()> {
    if let Some(line) = &result.stderr_line {
        return writeln!(sink, "{line}");
    }
    if let Some(message) = &result.diagnostic {
        if result.diagnostic_on_stderr
            || (format != OutputFormat::Json && result.text_output.is_none())
        {
            return write_diagnostic(sink, "DriverDiagnostic", message);
        }
    }
    // The selected primary output already carries FSL's diagnostics: as
    // structured members in JSON or as rendered lines in text mode.
    if format == OutputFormat::Json || result.text_output.is_some() {
        return Ok(());
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

/// Run one native-run/1 request. With `--provider-config` or
/// `--provider-limit` the run goes through FSL's `run_with_providers` with
/// the configured process providers registered beside FSL's engines and the
/// run's one limit set (FSL FR-961, FR-964); otherwise the driver runs the
/// request through FSL's interpreter with its own run options.
fn run(options: RunOptions) -> CommandResult {
    let provider_config = match load_provider_config(&options.providers) {
        Ok(config) => config,
        Err(refusal) => return *refusal,
    };
    if options.engine != ExecutionEngine::Interpreter {
        let failure = ExecutionFailure::UnsupportedEngine(options.engine);
        return CommandResult::document(failure.into_document());
    }
    if options.providers.given() {
        if let Some(option) = options.local_options.first() {
            return CommandResult::input_refusal(
                Operation::Execute,
                format!(
                    "`{option}` cannot be combined with --provider-config or --provider-limit: \
                     FSL's provider run entry takes no {option}"
                ),
            );
        }
        return run_with_providers(&options.path, provider_config, options.providers.limits);
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
                    filament_driver::LimitError::OperationUnavailable {
                        setting: *setting,
                        operation: "run",
                    }
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
            return run_clause_request(*request.into_request(), &options.limits)
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
    let render_sources = fsl_inspect::RenderSources::from_sources([source.clone()]);
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
        Ok(result) => CommandResult::document(result.document).with_sources(render_sources),
        Err(failure) => {
            CommandResult::document(failure.into_document()).with_sources(render_sources)
        }
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
        ClauseRunSelection::Clause(selection)
        | ClauseRunSelection::ClauseWithPath { selection, .. } => selection.name.clone(),
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

/// Compile a request through its owner and retain the actual package outcome.
fn compile(options: CompileOptions) -> CommandResult {
    let mut limits = SpineLimits::default();
    if let Err(error) = apply_frontend_limits(&options.limits, &mut limits, "compile") {
        return CommandResult::input_refusal(Operation::Package, error);
    }
    let clocks = match options
        .clocks
        .as_deref()
        .map(|path| crate::clocks::read(path, RunLimits::default().request_bytes, None))
        .transpose()
    {
        Ok(clocks) => clocks,
        Err(refusal) => return clocks_refusal(Operation::Package, refusal),
    };
    let mut limits = SpineLimits::default();
    if let Err(error) = apply_frontend_limits(&options.limits, &mut limits, "compile") {
        return CommandResult::input_refusal(Operation::Package, error);
    }
    let overrides = command::RequestOverrides {
        clocks: clocks.as_deref(),
        output: options.output.as_deref(),
        limits: Some(&limits),
        ..command::RequestOverrides::default()
    };
    match command::compile_outcome_with_overrides(&options.path, &overrides) {
        Ok(outcome) => CommandResult::document(outcome.document().clone())
            .with_sources(fsl_command::render_sources(outcome.sources())),
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
    // FSL FR-767: an unknown law refuses before any source, model or
    // library file is read.
    let integer_division = match options
        .integer_division
        .as_deref()
        .map(|law| admit_integer_division_law(law, "--integer-division"))
        .transpose()
    {
        Ok(law) => law,
        Err(refusal) => return *refusal,
    };
    let mut limits = SpineLimits::default();
    if let Err(error) = apply_frontend_limits(&options.limits, &mut limits, "check") {
        return CommandResult::input_refusal(Operation::Check, error);
    }
    if options.markdown {
        return check_markdown_source(options, limits, integer_division);
    }
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure_for(Operation::Check, Code::IoError, message);
        }
    };
    let source = options
        .source_identity
        .clone()
        .unwrap_or_else(|| SourceIdentity::new("filament", &options.path));
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
                integer_division,
                ..command::CheckInputs::default()
            };
            return check_result(&command::check_outcome(Path::new(&options.path), &inputs));
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
        integer_division,
        ..command::CheckInputs::default()
    };
    check_result(&command::check_outcome(Path::new(&options.path), &inputs))
}

/// Check each native fence in an explicitly named Markdown document (FR-662).
fn check_markdown_source(
    options: CheckOptions,
    limits: SpineLimits,
    integer_division: Option<fsl_replay::AdmittedIntegerDivision>,
) -> CommandResult {
    if options.model.is_some() || !options.libraries.is_empty() {
        return CommandResult::input_refusal(
            Operation::Check,
            "--model and --library are not supported with --markdown".to_owned(),
        );
    }
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure_for(Operation::Check, Code::IoError, message);
        }
    };
    let identity = options
        .source_identity
        .unwrap_or_else(|| SourceIdentity::new("filament", &options.path));
    let original = match Source::read_typed(
        identity,
        options.path.clone(),
        &bytes,
        limits.source.source_bytes,
    ) {
        Ok(source) => source,
        Err(refusal) => {
            let diagnostic =
                fsl_cst::diagnostic::source_refusal(*refusal, limits.source.source_bytes);
            return CommandResult::document(
                OutcomeDocument::new(Operation::Check, None, diagnostic.code.category())
                    .with_diagnostics(vec![OutcomeDiagnostic::from_source_diagnostic(&diagnostic)]),
            );
        }
    };
    let packages = fsl_semantics::model::intake::PackageInput::new();
    let dependencies = DependencyInput::default();
    let cancel = Cancel::new();
    let request = NativeFenceRequest {
        original: &original,
        packages: &packages,
        dependencies: &dependencies,
        extensions: None,
        integer_division: integer_division.as_ref(),
    };
    match check_native_fences(
        &request,
        CheckLimits {
            spine: limits,
            ..CheckLimits::default()
        },
        &cancel,
    ) {
        Ok(report) => CommandResult::document(OutcomeDocument::from_native_fences(report.value())),
        Err(failure) => {
            let (category, code) = match &failure {
                fsl_foundation::diagnostic::StageFailure::Refused(cause) => {
                    let code = cause.code();
                    (code.category(), code)
                }
                fsl_foundation::diagnostic::StageFailure::RefusedWithPayload(cause, _) => {
                    let code = cause.code();
                    (code.category(), code)
                }
                fsl_foundation::diagnostic::StageFailure::Limit(_)
                | fsl_foundation::diagnostic::StageFailure::LimitWithPayload(_, _) => {
                    (Category::Incomplete, Code::ResourceExhausted)
                }
                fsl_foundation::diagnostic::StageFailure::Cancelled(_) => {
                    (Category::Incomplete, Code::Cancelled)
                }
                fsl_foundation::diagnostic::StageFailure::Fault(_) => {
                    (Category::InternalFailure, Code::RuntimeInvariant)
                }
            };
            CommandResult::document(
                OutcomeDocument::new(Operation::Check, None, category).with_diagnostics(vec![
                    OutcomeDiagnostic::new(None, code.as_str(), None, format!("{failure:?}")),
                ]),
            )
        }
    }
}

/// The check result with the sources the check admitted.
fn check_result(outcome: &command::CheckOutcome) -> CommandResult {
    CommandResult::document(outcome.document().clone())
        .with_sources(fsl_command::render_sources(outcome.sources()))
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
    let mut limits = fsl_cst::Limits::default();
    let mut format_limits = FormatLimits::default();
    if let Err(error) = options
        .limits
        .apply_to_setting_group(&mut [&mut limits, &mut format_limits], "format")
    {
        return CommandResult::input_refusal(Operation::Format, error.to_string());
    }
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
        .and_then(|parsed| format_with_limits(&parsed, format_limits).map(Staged::new));
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

/// The operands of `fsl analyze` and `fsl replay`: one request file
/// (`native-analyze/1` or `native-replay/1`), an optional
/// `--provider-config` file and the `--provider-limit` set. The request
/// document carries every analysis or replay setting (FSL FR-620, FR-862);
/// the provider configuration names the process providers registered beside
/// FSL's engines (FSL FR-287, FR-635; FSpec FR-561).
struct RequestOptions {
    path: PathBuf,
    providers: ProviderArguments,
}

impl RequestOptions {
    fn parse(verb: Verb, arguments: &[OsString]) -> Result<Self, OptionRefusal> {
        let name = verb.name();
        let mut positionals = Vec::new();
        let mut providers = ProviderArguments::default();
        let mut index = 0;
        while index < arguments.len() {
            if let Some(taken) = providers.take(arguments, index)? {
                index += taken;
                continue;
            }
            let argument = &arguments[index];
            if argument == "--format" {
                index += 2;
                continue;
            }
            if let Some(option) = argument.to_str().filter(|value| value.starts_with('-')) {
                return Err(format!("unknown {name} option `{option}`").into());
            }
            positionals.push(argument);
            index += 1;
        }
        let [path] = positionals.as_slice() else {
            return Err(format!(
                "usage: fsl {name} <request-file> [--provider-config <path>] \
                 [--provider-limit name=value] [--format json|text]"
            )
            .into());
        };
        Ok(Self {
            path: PathBuf::from(*path),
            providers,
        })
    }
}

/// The CLI result of a refused provider option: a usage refusal is the
/// verb's typed `invalid-request` document; a refused `--provider-limit`
/// writes empty stdout and names the option and argument on stderr (FSL
/// FR-287; FR-006).
fn option_refusal(operation: Operation, error: &OptionRefusal) -> CommandResult {
    match error {
        OptionRefusal::Usage(message) => CommandResult::input_refusal(operation, message),
        OptionRefusal::Quiet(message) => {
            CommandResult::failure(Code::InvalidRequest.category(), message)
        }
    }
}

/// Read the caller's FSpec FR-561 document, or the empty one when none is
/// named. A refusal exits 20 with empty stdout and its diagnostic, naming
/// the file and the member, on stderr (FSpec FR-561; FR-006).
fn load_provider_config(
    arguments: &ProviderArguments,
) -> Result<crate::provider_config::ProviderConfig, Box<CommandResult>> {
    let Some(path) = arguments.config.as_deref() else {
        return Ok(crate::provider_config::ProviderConfig {
            format: crate::provider_config::FORMAT.to_owned(),
            providers: Vec::new(),
        });
    };
    crate::provider_config::load(path).map_err(|error| {
        let message = match error {
            crate::provider_config::ProviderConfigError::UnsupportedFormat { .. } => {
                format!("unknown_wire/unsupported-wire: {error}")
            }
            _ => format!("invalid-request: invalid provider configuration: {error}"),
        };
        Box::new(CommandResult::failure(Category::Refusal, message))
    })
}

/// The started process providers of one run: FSL's engines over them, the
/// sessions the driver shuts down after the run, and their admitted
/// descriptors for the driver's negotiation.
struct StartedProviders {
    engines: Vec<fsl_analyze::analyze::Engine>,
    sessions: Vec<crate::analyze_provider::SharedSession>,
    descriptors: Vec<fsl_route::BackendDescriptor>,
}

impl StartedProviders {
    /// Shut every started session down, whether or not FSL called it.
    fn close(self) {
        drop(self.engines);
        for session in self.sessions {
            let taken = session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if let Some(session) = taken {
                let _ = filament_driver::ProcessProviderSession::shutdown(session);
            }
        }
    }
}

/// Start each configured provider and admit its `hello` manifest with
/// `Process` origin (FSL FR-635 "Where provider manifests come from"). A
/// provider that does not start, or whose manifest does not advertise
/// (`value-validity`, `unbounded`), is a registration failure naming the
/// entry's executable; the providers already started are shut down. Every
/// session works under the run's one limit set (FSpec FR-777).
fn start_providers(
    config: crate::provider_config::ProviderConfig,
    limits: &fsl_analyze::provider::ProviderLimits,
    signals: &crate::signals::SignalContext,
    cancel: &Cancel,
) -> Result<StartedProviders, String> {
    let mut started: Vec<CliProviderSession> = Vec::new();
    let shut = |started: Vec<CliProviderSession>| {
        for session in started {
            let _ = filament_driver::ProcessProviderSession::shutdown(session);
        }
    };
    for configured in config.providers {
        let name = configured.path.display().to_string();
        match start_configured_provider(&configured, limits, signals, cancel) {
            Ok(session) => started.push(session),
            Err(error) => {
                shut(started);
                return Err(format!(
                    "process provider `{name}` did not register: {error:?}"
                ));
            }
        }
    }
    // FR-006-AC-26: distinct sessions with one admitted identity stop before
    // negotiation, close every session, and return the CLI's InvalidRequest
    // input refusal naming the identity.
    let mut identities = BTreeSet::new();
    let repeated = started.iter().find_map(|session| {
        let identity = filament_driver::ProcessProviderSession::descriptor(session)
            .id()
            .as_str()
            .to_owned();
        (!identities.insert(identity.clone())).then_some(identity)
    });
    if let Some(identity) = repeated {
        shut(started);
        return Err(format!(
            "configured process providers repeat admitted identity `{identity}`"
        ));
    }
    let admissions = started.iter().map(|session| {
        (
            filament_driver::ProcessProviderSession::descriptor(session).clone(),
            filament_driver::ProcessProviderSession::manifest(session).clone(),
        )
    });
    let registered = build_linked_providers(&[])
        .map_err(|error| format!("{error:?}"))
        .and_then(|mut linked| {
            linked
                .register_admitted(admissions)
                .map_err(|error| format!("{error:?}"))?;
            Ok(linked.manifest_snapshot())
        });
    let manifest = match registered {
        Ok(manifest) => std::sync::Arc::<[filament_driver::ProviderManifest]>::from(manifest),
        Err(error) => {
            shut(started);
            return Err(format!(
                "configured process providers did not register: {error}"
            ));
        }
    };
    let mut providers = StartedProviders {
        engines: Vec::new(),
        sessions: Vec::new(),
        descriptors: Vec::new(),
    };
    let mut refusal = None;
    for session in started {
        let descriptor = filament_driver::ProcessProviderSession::descriptor(&session).clone();
        let tool = manifest_tool(filament_driver::ProcessProviderSession::manifest(&session));
        let shared = std::sync::Arc::new(std::sync::Mutex::new(Some(session)));
        providers.sessions.push(std::sync::Arc::clone(&shared));
        if refusal.is_some() {
            continue;
        }
        let Some(tool) = tool else {
            refusal = Some(format!(
                "process provider `{}` did not register: its hello manifest names no `tool` \
                 (FSpec FR-331-AC-37)",
                descriptor.id().as_str()
            ));
            continue;
        };
        let provider = crate::analyze_provider::SessionProvider {
            session: shared,
            manifest: std::sync::Arc::clone(&manifest),
        };
        match fsl_analyze::analyze::Engine::external(descriptor.clone(), tool, Box::new(provider)) {
            Ok(engine) => {
                providers.engines.push(engine);
                providers.descriptors.push(descriptor);
            }
            Err(error) => refusal = Some(format!("process provider did not register: {error}")),
        }
    }
    match refusal {
        Some(refusal) => {
            providers.close();
            Err(refusal)
        }
        None => Ok(providers),
    }
}

/// The FSpec FR-331 `tool` of an admitted hello manifest: the retained
/// descriptor's `{name, version}`, or `None` when it names none.
fn manifest_tool(
    manifest: &filament_driver::ProviderManifest,
) -> Option<fsl_analyze::analyze::BackendTool> {
    let descriptor: serde_json::Value = serde_json::from_slice(manifest.wire.as_deref()?).ok()?;
    let tool = descriptor.get("tool")?;
    fsl_analyze::analyze::BackendTool::new(
        tool.get("name")?.as_str()?,
        tool.get("version")?.as_str()?,
    )
    .ok()
}

/// Start one configured process provider under the run's transport limits
/// and admit its `hello`. The process memory and CPU-time limits are not
/// applied: the plugin host offers neither (FSpec FR-462 sets them "where
/// the platform provides one").
fn start_configured_provider(
    configured: &crate::provider_config::ProviderEntry,
    limits: &fsl_analyze::provider::ProviderLimits,
    signals: &crate::signals::SignalContext,
    cancel: &Cancel,
) -> Result<CliProviderSession, CliProviderError> {
    let max_frame_bytes = limits.max_frame_bytes;
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
        .arguments
        .iter()
        .map(OsString::from)
        .collect::<Vec<_>>();
    let limits = ProcessProviderLimits {
        max_frame_bytes: limits.max_frame_bytes,
        max_total_bytes: limits.max_total_bytes,
        max_stderr_bytes: limits.max_stderr_bytes,
        cancellation_grace: std::time::Duration::from_millis(limits.grace_ms),
    };
    provider.start(&configured.path, &arguments, limits, cancel)
}

/// Read the `native-analyze/1` request, have FSL analyze its items under the
/// driver's negotiation, and keep FSL FR-621's document with its exit code,
/// or FR-624's whole-request refusal.
///
/// The run is one signal-reporting session: SIGINT or SIGTERM cancels FSL's
/// handle with cause `requested`, and the verb writes the outcome FSL settles
/// for it (FSL FR-624 `cancelled`, exit 22) rather than dying by the signal.
fn analyze(options: &RequestOptions, format: OutputFormat) -> CommandResult {
    let config = match load_provider_config(&options.providers) {
        Ok(config) => config,
        Err(refusal) => return *refusal,
    };
    let path = options.path.clone();
    let ran = with_started_providers(
        config,
        options.providers.limits,
        true,
        move |negotiation, engines, provider_options, cancel| {
            command::analyze_with_providers(&path, negotiation, engines, provider_options, cancel)
        },
    );
    let written = match ran {
        Ok(Ok(written)) => written,
        Ok(Err(message)) => return CommandResult::input_refusal(Operation::Analyze, message),
        Err(error) => {
            return CommandResult::failure(
                Category::InternalFailure,
                format!("could not initialize process signal handling: {error}"),
            )
        }
    };
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

/// Inside one signal session, start the configured providers under `limits`,
/// build the driver's negotiation over them and FSL's engines, run `entry`
/// with them, and shut every session down once it returns. A `reporting`
/// session consumes a received signal so the command writes FSL's
/// cancellation outcome (FR-022); otherwise the signal is re-raised after
/// cleanup (FR-006-AC-4).
fn with_started_providers<T: Send + 'static>(
    config: crate::provider_config::ProviderConfig,
    limits: fsl_analyze::provider::ProviderLimits,
    reporting: bool,
    entry: impl FnOnce(
            &filament_driver::AnalyzeNegotiation,
            Vec<fsl_analyze::analyze::Engine>,
            &fsl_analyze::provider::ProviderOptions,
            &Cancel,
        ) -> T
        + Send
        + 'static,
) -> io::Result<Result<T, String>> {
    crate::signals::run(move |signals| {
        let cancel = Cancel::new();
        let _session = if reporting {
            signals.activate_reporting(cancel.clone())
        } else {
            signals.activate(cancel.clone())
        };
        let mut providers = start_providers(config, &limits, &signals, &cancel)?;
        let negotiation =
            match filament_driver::AnalyzeNegotiation::with_providers(&providers.descriptors) {
                Ok(negotiation) => negotiation,
                Err(error) => {
                    providers.close();
                    return Err(format!("{error:?}"));
                }
            };
        let engines = std::mem::take(&mut providers.engines);
        let options = fsl_analyze::provider::ProviderOptions {
            provider_limits: limits,
        };
        let written = entry(&negotiation, engines, &options, &cancel);
        providers.close();
        Ok(written)
    })
}

/// `fsl run` with configured providers: FSL's `run_with_providers` runs the
/// request and settles its call's requirement items against FSL's engines
/// and the started process providers (FSL FR-961, FR-963), under the run's
/// one limit set (FSL FR-964).
fn run_with_providers(
    path: &Path,
    config: crate::provider_config::ProviderConfig,
    limits: fsl_analyze::provider::ProviderLimits,
) -> CommandResult {
    let path = path.to_path_buf();
    let ran = with_started_providers(
        config,
        limits,
        false,
        move |negotiation, engines, options, cancel| {
            command::run_with_providers(&path, negotiation, engines, options, cancel)
        },
    );
    let written = match ran {
        Ok(Ok(written)) => written,
        Ok(Err(message)) => return CommandResult::input_refusal(Operation::Execute, message),
        Err(error) => {
            return CommandResult::failure(
                Category::InternalFailure,
                format!("could not initialize process signal handling: {error}"),
            )
        }
    };
    match written {
        Ok(result) => match result.outcome {
            Some(document) => CommandResult::document(document),
            None => native_result(result.value.as_value().clone(), result.exit_code),
        },
        Err(error) => CommandResult::document(error.outcome_document(Operation::Execute)),
    }
}

/// `fsl replay`: FSL replays the `native-replay/1` request's FSpec FR-336
/// replay request (FSL FR-862, FSpec FR-715) and the verb writes the FR-323
/// `results` document in machine mode and FSL's FR-865 text of the replay
/// outcome otherwise, with FSL's exit code. The request is read only by FSL's
/// one replay reader, which refuses a `clocks` member at stage `request`
/// because a replay takes its clocks from the proving package (FSL FR-640).
/// A request file of another format is refused before FSL runs, so `replay`
/// never runs or analyzes it. `--provider-config` and `--provider-limit` are
/// read and the providers started and shut down as for `run` (FR-006).
fn replay(options: &RequestOptions, format: OutputFormat) -> CommandResult {
    let config = match load_provider_config(&options.providers) {
        Ok(config) => config,
        Err(refusal) => return *refusal,
    };
    if let Some(other) = other_request_format(&options.path) {
        return with_format(
            CommandResult::input_refusal(
                Operation::Replay,
                format!(
                    "`replay` reads a `{}` request; `{}` has format `{other}`",
                    WireFormat::ReplayRequest.as_str(),
                    options.path.display()
                ),
            ),
            format,
        );
    }
    let path = options.path.clone();
    let ran = with_started_providers(
        config,
        options.providers.limits,
        false,
        move |negotiation, engines, provider_options, cancel| {
            command::run_with_providers(&path, negotiation, engines, provider_options, cancel)
        },
    );
    let written = match ran {
        Ok(Ok(written)) => written,
        Ok(Err(message)) => return CommandResult::input_refusal(Operation::Replay, message),
        Err(error) => {
            return CommandResult::failure(
                Category::InternalFailure,
                format!("could not initialize process signal handling: {error}"),
            )
        }
    };
    match written {
        Ok(result) => match (format, result.outcome) {
            (OutputFormat::Text, Some(document)) => {
                with_format(CommandResult::document(document), format)
            }
            _ => native_result(result.value.as_value().clone(), result.exit_code),
        },
        Err(error) => with_format(
            CommandResult::document(error.outcome_document(Operation::Replay)),
            format,
        ),
    }
}

/// The `format` of the request file at `path` when it is a JSON object
/// naming a format other than `native-replay/1`. A file that cannot be read
/// or is not such an object is left to FSL's reader, which refuses it with
/// its own cause.
fn other_request_format(path: &Path) -> Option<String> {
    let limit = RunLimits::default().request_bytes;
    let file = File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(u64::try_from(limit).ok()?.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > limit {
        return None;
    }
    let document: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let format = document.get("format")?.as_str()?;
    (format != WireFormat::ReplayRequest.as_str()).then(|| format.to_owned())
}

/// A run result FSL writes as its native document rather than an FR-286
/// outcome: the document unchanged in machine mode and its JSON text
/// otherwise, under the category FSL's exit code names (FR-285).
fn native_result(document: serde_json::Value, exit_code: u8) -> CommandResult {
    let Some(category) = Category::ALL
        .into_iter()
        .find(|category| category.exit_code() == exit_code)
    else {
        return CommandResult::failure(
            Category::InternalFailure,
            format!("FSL's run exit code {exit_code} names no category"),
        );
    };
    CommandResult {
        category,
        document: None,
        diagnostic: None,
        diagnostic_on_stderr: false,
        text_output: Some(document.to_string()),
        analyze_document: Some(document),
        stderr_line: None,
        render_sources: None,
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
        diagnostic_on_stderr: false,
        text_output,
        analyze_document: Some(document),
        stderr_line: None,
        render_sources: None,
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
    /// `--provider-config` and `--provider-limit`, as every provider-routing
    /// verb takes them (FSL FR-287, FR-644).
    providers: ProviderArguments,
}

impl MonitorOptions {
    const USAGE: &str = "usage: fsl monitor <request-file> [--input <file>] [--provider-config <path>] [--provider-limit name=value] [--format json|text]";

    fn parse(arguments: &[OsString]) -> Result<Self, OptionRefusal> {
        let mut positionals = Vec::new();
        let mut input = None;
        let mut providers = ProviderArguments::default();
        let mut index = 0;
        while index < arguments.len() {
            if let Some(taken) = providers.take(arguments, index)? {
                index += taken;
                continue;
            }
            let argument = &arguments[index];
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument == "--input" {
                if input.is_some() {
                    return Err("--input may be supplied only once".to_owned().into());
                }
                let value = arguments
                    .get(index + 1)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| OptionRefusal::from("--input requires a path".to_owned()))?;
                input = Some(PathBuf::from(value));
                index += 2;
                continue;
            }
            if let Some(option) = argument.to_str().filter(|value| value.starts_with('-')) {
                return Err(format!("unknown monitor option `{option}`").into());
            }
            positionals.push(argument);
            index += 1;
        }
        let [path] = positionals.as_slice() else {
            return Err(Self::USAGE.to_owned().into());
        };
        Ok(Self {
            path: PathBuf::from(*path),
            input,
            providers,
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

/// The monitor's record stream, read on its own thread so a cancelled run
/// ends the stream without waiting for a blocked read (FR-025).
struct CancellableRecords {
    records: std::sync::mpsc::Receiver<io::Result<Vec<u8>>>,
    cancel: Cancel,
}

impl CancellableRecords {
    /// How often a waiting read looks at the cancellation handle.
    const POLL: std::time::Duration = std::time::Duration::from_millis(20);

    fn new(reader: Box<dyn io::BufRead + Send>, cancel: Cancel) -> Self {
        let (sender, records) = std::sync::mpsc::sync_channel(1);
        // The reader thread ends at end of stream, or when the receiver is
        // gone; a read still blocked then ends with the process.
        let _ = std::thread::Builder::new()
            .name("fsl-monitor-records".to_owned())
            .spawn(move || {
                for record in io::BufRead::split(reader, b'\n') {
                    if sender.send(record).is_err() {
                        break;
                    }
                }
            });
        Self { records, cancel }
    }
}

impl Iterator for CancellableRecords {
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.cancel.cause().is_some() {
                return None;
            }
            match self.records.recv_timeout(Self::POLL) {
                Ok(record) => return Some(record),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return None,
            }
        }
    }
}

/// Run the `native-monitor/1` request over its record stream, writing each
/// event to standard output before the next record is read, then the summary.
/// The exit code is FSL's. A refusal before any record is read is the
/// command-error envelope.
fn monitor(options: &MonitorOptions, format: OutputFormat) -> CommandResult {
    let reader: Box<dyn io::BufRead + Send> = match &options.input {
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
    let config = match load_provider_config(&options.providers) {
        Ok(config) => config,
        Err(refusal) => return *refusal,
    };
    let path = options.path.clone();
    // The run is one signal-reporting session: SIGINT or SIGTERM cancels the
    // handle with cause `requested`, the record stream ends at once even
    // while a read blocks, and FSL settles the cancelled session (exit 22).
    // The configured providers start inside it and FSL routes the request's
    // `items` to them before the first record (FSL FR-644, FR-635).
    let ran = with_started_providers(
        config,
        options.providers.limits,
        true,
        move |negotiation, engines, provider_options, cancel| {
            let mut output = MonitorOutput {
                format,
                closed: false,
                failure: None,
            };
            let records = CancellableRecords::new(reader, cancel.clone());
            let summary = command::monitor(
                &path,
                records,
                &mut |event| output.line(event.as_value(), || monitor_event_text(event.as_value())),
                command::MonitorProviders {
                    negotiate: negotiation,
                    providers: engines,
                    options: provider_options,
                },
                RunLimits::default(),
                cancel,
            );
            // A stream the cancellation ended is not the complete stream, so its
            // summary does not stand: the run is FSL's cancelled monitor run.
            let summary = match summary {
                Ok(_) if cancel.cause().is_some() => Err(Box::new(command::RunError {
                    cause: command::RunCause::MonitorCancelled,
                    sources: Default::default(),
                })),
                summary => summary,
            };
            (summary, output)
        },
    );
    let (summary, mut output) = match ran {
        Ok(Ok(ran)) => ran,
        Ok(Err(message)) => return CommandResult::input_refusal(Operation::Monitor, message),
        Err(error) => {
            return CommandResult::failure(
                Category::InternalFailure,
                format!("could not initialize process signal handling: {error}"),
            )
        }
    };
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
        diagnostic_on_stderr: false,
        text_output: None,
        analyze_document: None,
        stderr_line: None,
        render_sources: None,
    }
}

/// Dispatch the root verb while preserving its FSL-owned result category.
fn dispatch(verb: Verb, format: OutputFormat, raw_arguments: &[OsString]) -> CommandResult {
    if verb == Verb::Analyze {
        return match RequestOptions::parse(verb, raw_arguments) {
            Ok(options) => analyze(&options, format),
            Err(error) => option_refusal(Operation::Analyze, &error),
        };
    }
    if verb == Verb::Replay {
        return match RequestOptions::parse(verb, raw_arguments) {
            Ok(options) => replay(&options, format),
            Err(error) => option_refusal(Operation::Replay, &error),
        };
    }
    if verb == Verb::Monitor {
        return match MonitorOptions::parse(raw_arguments) {
            Ok(options) => monitor(&options, format),
            Err(error) => option_refusal(Operation::Monitor, &error),
        };
    }
    if verb == Verb::Run {
        return match RunOptions::parse(raw_arguments) {
            Ok(options) => with_format(run(options), format),
            Err(error) => option_refusal(Operation::Execute, &error),
        };
    }
    if verb == Verb::Compile {
        return match CompileOptions::parse(raw_arguments) {
            Ok(options) => with_format(compile(options), format),
            Err(error) => CommandResult::input_refusal(Operation::Package, error),
        };
    }
    if verb == Verb::Check {
        return match CheckOptions::parse(raw_arguments) {
            Ok(options) => with_format(check_source(options), format),
            Err(error) => option_refusal(Operation::Check, &error),
        };
    }
    if verb == Verb::Parse {
        return match SourceOptions::parse(verb, raw_arguments) {
            Ok(options) => with_format(parse_source(options), format),
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
    unrouted_verb(verb, format)
}

/// The library operation an accepted but unrouted verb names, `None` for a
/// routed verb.
fn unrouted_operation(verb: Verb) -> Option<Operation> {
    match verb {
        Verb::Prove => Some(Operation::Prove),
        Verb::Generate => Some(Operation::Generate),
        Verb::Inspect => Some(Operation::Inspect),
        Verb::Render => Some(Operation::Render),
        Verb::Parse
        | Verb::Format
        | Verb::Check
        | Verb::Compile
        | Verb::Run
        | Verb::Analyze
        | Verb::Monitor
        | Verb::Replay => None,
    }
}

/// In text mode, replace the result's outcome document with FSL's FR-865 text
/// form of it, built from the same document and the sources the command read.
/// Machine mode keeps the document untouched and the exit category never moves.
fn with_format(mut result: CommandResult, format: OutputFormat) -> CommandResult {
    if format != OutputFormat::Text {
        return result;
    }
    let Some(document) = result.document.as_ref() else {
        return result;
    };
    let sources = result
        .render_sources
        .take()
        .unwrap_or_else(|| fsl_inspect::RenderSources::from_sources([]));
    match fsl_inspect::render_outcome(
        document,
        fsl_inspect::RenderForm::Text,
        &sources,
        fsl_replay::RenderLimits::default(),
        &Cancel::new(),
    ) {
        Ok(staged) => {
            result.text_output = Some(staged.into_value().as_str().to_owned());
            result
        }
        Err(_) => CommandResult::failure(
            Category::InternalFailure,
            "could not render the outcome document as text",
        ),
    }
}

/// Refuse a verb this build accepts but does not route (FSpec FR-522): an
/// `unsupported_operation` document with cause `operation-not-routed` whose
/// detail names the verb under machine output, or the named stderr line
/// otherwise. Neither stream stays empty.
fn unrouted_verb(verb: Verb, format: OutputFormat) -> CommandResult {
    const CODE: &str = "unsupported_operation";
    const CAUSE: &str = "operation-not-routed";
    let Some(operation) = unrouted_operation(verb) else {
        return CommandResult::failure(
            Category::InternalFailure,
            format!("verb `{}` reached the unrouted-verb refusal", verb.name()),
        );
    };
    let name = verb.name();
    let line = format!("fsl {name}: this build does not route the {name} operation ({CODE})");
    let document =
        OutcomeDocument::new(operation, None, Category::Unsupported).with_diagnostics(vec![
            OutcomeDiagnostic::new(Some(CAUSE), CODE, None, line.clone()),
        ]);
    let mut result = CommandResult::document(document.clone());
    if format != OutputFormat::Json {
        result.stderr_line = Some(line);
        return result;
    }
    match unrouted_wire(&document, name) {
        Some(wire) => result.analyze_document = Some(wire),
        None => result.stderr_line = Some(line),
    }
    result
}

/// The wire form of `document` with the verb as the first diagnostic's
/// `detail` payload.
fn unrouted_wire(document: &OutcomeDocument, verb: &str) -> Option<serde_json::Value> {
    let mut wire = serde_json::to_value(document).ok()?;
    let diagnostic = wire.get_mut("diagnostics")?.get_mut(0)?.as_object_mut()?;
    diagnostic.insert("detail".to_owned(), serde_json::json!({ "verb": verb }));
    Some(wire)
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
    let _ = write_result_diagnostics(result, format, &mut stderr);
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
                    let _ = write_result_diagnostics(&result, format, &mut stderr);
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
        parse_invocation, read_source, CheckOptions, ExecutionEngine, Invocation, OptionRefusal,
        OutputFormat, RunOptions, Verb,
    };
    use fsl_foundation::{Setting, SettingName};
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    #[test]
    fn unavailable_limit_error_is_generic_and_cli_message_names_operation() {
        let error = filament_driver::LimitError::OperationUnavailable {
            setting: SettingName::Stage(Setting::ExploreStates),
            operation: "run",
        };
        assert_eq!(
            error.to_string(),
            "limit setting `explore.states` is unavailable for the `run` operation"
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
        assert!(!options.providers.given());
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
            options.providers.config.as_deref(),
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
        assert_eq!(
            RunOptions::parse(&missing).err(),
            Some(OptionRefusal::Usage(
                "--provider-config requires a path".to_owned()
            ))
        );

        let duplicate = vec![
            OsString::from("unit.fsl"),
            OsString::from("--provider-config"),
            OsString::from("one.json"),
            OsString::from("--provider-config"),
            OsString::from("two.json"),
        ];
        assert_eq!(
            RunOptions::parse(&duplicate).err(),
            Some(OptionRefusal::Usage(
                "--provider-config may be supplied only once".to_owned()
            ))
        );
    }

    /// Trace: FR-006-AC-29; a refused `--provider-limit` is its own refusal,
    /// with empty stdout, not a usage refusal.
    #[test]
    fn run_options_refuse_a_malformed_provider_limit() {
        let args = vec![
            OsString::from("unit.fsl"),
            OsString::from("--provider-limit"),
            OsString::from("deadline_ms=010"),
        ];
        assert!(matches!(
            RunOptions::parse(&args).err(),
            Some(OptionRefusal::Quiet(message))
                if message.contains("--provider-limit `deadline_ms=010`")
        ));
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

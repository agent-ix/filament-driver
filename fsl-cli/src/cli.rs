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
    check_native_fences, parse, parse_selecting_extensions, CheckLimits, CompileRefusal,
    DependencyInput, NativeFenceRequest, ParseRequest, ProfileCatalogInput, SpineLimits,
};
use fsl_replay::RunRefusal;
use fsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument, OutcomeStage};
use fsl_semantics::library::bundle::PackageLimits;
use ix_cli_kit::exit::{caller_exit_code, RESERVED_COMMAND_NOT_FOUND};
use ix_cli_kit::streams::{self, Diagnostics};

use crate::extension_set;
use crate::options::{self, Scan, Scanned};
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

    let commands: Vec<(&str, &str)> = Verb::ALL
        .into_iter()
        .map(|verb| (verb.name(), verb.help_description()))
        .collect();
    let help = options::root_help(options::table(), &commands);
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
    Export,
    Analyze,
    Monitor,
    Prove,
    Replay,
    Generate,
    Inspect,
    Render,
}

impl Verb {
    const ALL: [Self; 13] = [
        Self::Parse,
        Self::Format,
        Self::Check,
        Self::Compile,
        Self::Run,
        Self::Export,
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
            Self::Export => "export",
            Self::Analyze => "analyze",
            Self::Monitor => "monitor",
            Self::Prove => "prove",
            Self::Replay => "replay",
            Self::Generate => "generate",
            Self::Inspect => "inspect",
            Self::Render => "render",
        }
    }

    /// The library operation the verb runs.
    fn operation(self) -> Operation {
        match self {
            Self::Parse => Operation::Parse,
            Self::Format => Operation::Format,
            Self::Check => Operation::Check,
            Self::Compile => Operation::Package,
            Self::Run => Operation::Execute,
            Self::Export => Operation::Export,
            Self::Analyze => Operation::Analyze,
            Self::Monitor => Operation::Monitor,
            Self::Prove => Operation::Prove,
            Self::Replay => Operation::Replay,
            Self::Generate => Operation::Generate,
            Self::Inspect => Operation::Inspect,
            Self::Render => Operation::Render,
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
            Self::Export => "Executes an export request; --output names its target file.",
            Self::Replay => "Executes a native-replay/1 request; --format json or text.",
            Self::Inspect => "Verifies a checked-package handoff; --handoff names its directory.",
            Self::Prove | Self::Generate | Self::Render => {
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

/// The operands of `fsl export` (FSpec FR-817).
struct ExportOptions {
    output: Option<PathBuf>,
    request: Option<PathBuf>,
    limits: LimitOverrides,
}

impl ExportOptions {
    fn from_scanned(scanned: &Scanned) -> Result<Self, String> {
        let output = scanned.value(options::OUTPUT).map(PathBuf::from);
        let evidence = scanned.value(options::EVIDENCE).map(PathBuf::from);
        let limit_operands = limit_operands(scanned)?;
        let usage = entry_usage(Verb::Export);
        let request = match (scanned.positionals.as_slice(), evidence.as_ref()) {
            ([request], None) => Some(PathBuf::from(request)),
            ([], Some(_)) => None,
            ([], None) => return Err(usage),
            (_, Some(_)) => {
                return Err(format!(
                    "{} cannot be combined with a request file",
                    options::EVIDENCE
                ));
            }
            (_, None) => return Err(usage),
        };
        if evidence.is_some() && output.is_some() {
            return Err(format!(
                "{} cannot be combined with {}",
                options::EVIDENCE,
                options::OUTPUT
            ));
        }
        let request = request.or_else(|| evidence.clone());
        let limits = LimitOverrides::parse(&limit_operands).map_err(|error| error.to_string())?;
        Ok(Self {
            output,
            request,
            limits,
        })
    }
}

/// The usage refusal text of `verb`'s table entry.
fn entry_usage(verb: Verb) -> String {
    options::entry(verb.name()).map_or_else(String::new, options::VerbEntry::usage_refusal)
}

/// Each `<identity>=<file>` operand of `option` (FSL FR-866): split at the first
/// `=`, refused when it has no `=`, an empty identity or path, or repeats an
/// identity, before any file is read.
fn identity_operands(scanned: &Scanned, option: &str) -> Result<Vec<(String, PathBuf)>, String> {
    let mut selections: Vec<(String, PathBuf)> = Vec::new();
    for value in scanned.values(option) {
        let operand = value.to_string_lossy();
        let (identity, file) = match value.to_str().and_then(|text| text.split_once('=')) {
            Some((identity, file)) if !identity.is_empty() && !file.is_empty() => (identity, file),
            _ => {
                return Err(format!(
                    "{option} operand `{operand}` must be <identity>=<file>"
                ));
            }
        };
        if selections.iter().any(|(given, _)| given == identity) {
            return Err(format!(
                "{option} names the identity `{identity}` more than once"
            ));
        }
        selections.push((identity.to_owned(), PathBuf::from(file)));
    }
    Ok(selections)
}

/// Each `--limit` operand, as UTF-8 text, in the order given.
fn limit_operands(scanned: &Scanned) -> Result<Vec<String>, String> {
    scanned
        .values(options::LIMIT)
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{} requires name=value", options::LIMIT))
        })
        .collect()
}

/// Parse the root command surface defined by FSL ADR-029.
fn parse_invocation(arguments: &[OsString]) -> Result<Invocation, String> {
    let Some(command) = arguments.first() else {
        return Ok(Invocation::Help);
    };
    let command = command
        .to_str()
        .ok_or_else(|| "command must be valid UTF-8".to_owned())?;
    if options::HELP_OPTIONS.contains(&command) || command == options::HELP_WORD {
        return Ok(Invocation::Help);
    }
    if options::VERSION_OPTIONS.contains(&command) {
        return Ok(Invocation::Version);
    }
    let verb = Verb::parse(command).ok_or_else(|| format!("unknown verb `{command}`"))?;
    let mut format = OutputFormat::Text;
    let mut format_given = false;
    let mut index = 1;
    while index < arguments.len() {
        if arguments[index] == options::FORMAT {
            if format_given {
                let usage = options::entry(verb.name())
                    .map(options::VerbEntry::usage_refusal)
                    .unwrap_or_else(|| format!("usage: fsl {}", verb.name()));
                return Err(format!(
                    "{} may be supplied only once; {usage}",
                    options::FORMAT
                ));
            }
            format_given = true;
            index += 1;
            let value = arguments
                .get(index)
                .and_then(|value| value.to_str())
                .ok_or_else(|| format!("{} requires text, json, or dot", options::FORMAT))?;
            format = match value {
                "text" => OutputFormat::Text,
                "json" => OutputFormat::Json,
                "dot" => OutputFormat::Dot,
                other => {
                    return Err(format!(
                        "unknown output format `{other}` ({} accepts text, json, or dot)",
                        options::FORMAT
                    ));
                }
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

/// The operands of `fsl inspect` (FSpec FR-301): `--handoff <directory>`.
struct InspectOptions {
    handoff: Option<PathBuf>,
}

impl InspectOptions {
    /// Apply FR-002-AC-23's operand shape: `--handoff` takes its directory
    /// and no other operand; without `--handoff` the form is not routed.
    fn from_scanned(scanned: &Scanned) -> Result<Self, String> {
        let handoff = scanned.value(options::HANDOFF).map(PathBuf::from);
        if handoff.is_some() {
            if let Some(extra) = scanned.positionals.first() {
                return Err(format!(
                    "unexpected operand `{}` after {} <directory>",
                    extra.to_string_lossy(),
                    options::HANDOFF
                ));
            }
        }
        Ok(Self { handoff })
    }
}

/// Write the document FSL's `inspect_handoff` returns for the directory
/// exactly as the caller gave it (FSL FR-665).
fn inspect(options: &InspectOptions, format: OutputFormat) -> CommandResult {
    let Some(directory) = &options.handoff else {
        return unrouted_verb(Verb::Inspect, format);
    };
    if format == OutputFormat::Dot {
        return CommandResult::input_refusal(
            Operation::Inspect,
            format!(
                "{} `dot` is not available with {}",
                options::FORMAT,
                options::HANDOFF
            ),
        );
    }
    with_format(
        CommandResult::document(command::inspect_handoff(directory)),
        format,
    )
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
    /// Each `--model <identity>=<file>` (FSL FR-866), in option order.
    model: Vec<(String, PathBuf)>,
    libraries: Vec<(String, PathBuf)>,
    dependent_bytes: usize,
    limits: LimitOverrides,
    /// The `--integer-division <law>` value, unread until the check runs
    /// (FSL FR-767).
    integer_division: Option<String>,
    /// `--extensions <file>` (FSpec FR-562).
    extensions: Option<PathBuf>,
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
    /// `parse --extensions <file>` (FSpec FR-562).
    extensions: Option<PathBuf>,
}

impl SourceOptions {
    fn from_scanned(verb: Verb, scanned: &Scanned) -> Result<Self, String> {
        let limit_operands = limit_operands(scanned)?;
        let extensions = scanned.value(options::EXTENSIONS).map(PathBuf::from);
        let [authority, identity, path] = scanned.positionals.as_slice() else {
            return Err(entry_usage(verb));
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
            path: PathBuf::from(path),
            limits,
            extensions,
        })
    }
}

impl CompileOptions {
    fn from_scanned(scanned: &Scanned) -> Result<Self, String> {
        let clocks = scanned.value(options::CLOCKS).map(PathBuf::from);
        let output = scanned.value(options::OUTPUT).map(PathBuf::from);
        let limit_operands = limit_operands(scanned)?;
        let [path] = scanned.positionals.as_slice() else {
            return Err(entry_usage(Verb::Compile));
        };
        let limits = parse_frontend_limit_operands(&limit_operands, &SpineLimits::default())?;
        Ok(Self {
            path: PathBuf::from(path),
            clocks,
            output,
            limits,
        })
    }
}

impl CheckOptions {
    fn from_scanned(scanned: &Scanned) -> Result<Self, OptionRefusal> {
        let markdown = scanned.has(options::MARKDOWN);
        let integer_division = scanned
            .value(options::INTEGER_DIVISION)
            .map(|law| {
                law.to_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("{} requires a law", options::INTEGER_DIVISION))
            })
            .transpose()?;
        let model = identity_operands(scanned, options::MODEL)?;
        let extensions = scanned.value(options::EXTENSIONS).map(PathBuf::from);
        let libraries = identity_operands(scanned, options::LIBRARY)?;
        let dependent_bytes = match scanned.value(options::DEPENDENT_BYTES) {
            Some(value) => value
                .to_str()
                .and_then(|value| value.parse::<usize>().ok())
                .ok_or_else(|| {
                    format!(
                        "{} requires a non-negative byte count",
                        options::DEPENDENT_BYTES
                    )
                })?,
            None => RunLimits::default().dependent_bytes,
        };
        let limit_operands = limit_operands(scanned)?;
        let positionals = scanned
            .positionals
            .iter()
            .map(|value| {
                value
                    .to_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "check arguments must be valid UTF-8".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (source_identity, path) = match positionals.as_slice() {
            [path] => (None, path.clone()),
            [authority, identity, path] => {
                (Some(SourceIdentity::new(authority, identity)), path.clone())
            }
            _ => return Err(entry_usage(Verb::Check).into()),
        };
        if markdown && source_identity.is_none() {
            return Err(entry_usage(Verb::Check).into());
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
            extensions,
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
    overrides
        .apply_to_limits(bounds, operation)
        .map_err(|error| {
            let accepted = accepted_frontend_limit_settings(bounds);
            format!(
                "{}; settings accepted by this command: {accepted}",
                error.message_for(operation)
            )
        })
}

fn apply_export_limits(overrides: &LimitOverrides, limits: &mut SpineLimits) -> Result<(), String> {
    overrides.apply_to_export(limits).map_err(|error| {
        let accepted = accepted_frontend_limit_settings(limits);
        format!(
            "{}; settings accepted by this command: {accepted}",
            error.message_for("export")
        )
    })
}

impl RunOptions {
    fn from_scanned(scanned: &Scanned) -> Result<Self, OptionRefusal> {
        let mut intake_limits = RunLimits::default();
        for (name, slot) in [
            (options::REQUEST_BYTES, &mut intake_limits.request_bytes),
            (options::DEPENDENT_BYTES, &mut intake_limits.dependent_bytes),
        ] {
            if let Some(value) = scanned.value(name) {
                *slot = value
                    .to_str()
                    .and_then(|value| value.parse::<usize>().ok())
                    .ok_or_else(|| format!("{name} requires a non-negative byte count"))?;
            }
        }
        let limit_operands = limit_operands(scanned)?;
        let engine = match scanned.value(options::ENGINE) {
            Some(value) => {
                let value = value.to_str().ok_or_else(|| {
                    format!("{} requires interpreter, aot, or jit", options::ENGINE)
                })?;
                ExecutionEngine::parse(value)?
            }
            None => ExecutionEngine::default(),
        };
        let clocks = scanned.value(options::CLOCKS).map(PathBuf::from);
        let mut providers = crate::provider_config::ProviderArguments::default();
        providers.take_all(scanned)?;
        let local_options = scanned
            .options
            .iter()
            .map(|(name, _)| *name)
            .find(|name| {
                [
                    options::REQUEST_BYTES,
                    options::DEPENDENT_BYTES,
                    options::LIMIT,
                    options::CLOCKS,
                ]
                .contains(name)
            })
            .into_iter()
            .collect();
        let [path] = scanned.positionals.as_slice() else {
            return Err(entry_usage(Verb::Run).into());
        };
        Ok(Self {
            path: PathBuf::from(path),
            intake_limits,
            limits: LimitOverrides::parse(&limit_operands).map_err(|error| error.to_string())?,
            engine,
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

    /// A successful result whose only output is `text` on stdout in every
    /// output format, such as a verb's help.
    fn plain(text: String) -> Self {
        Self {
            category: Category::Success,
            document: None,
            diagnostic: None,
            diagnostic_on_stderr: false,
            text_output: Some(text),
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

/// Convert accepted observation settings to FSL's typed request override.
/// Other run settings continue through the selected run path.
fn observation_limits_override(
    limits: &LimitOverrides,
) -> (Option<fsl_replay::ObservationLimits>, bool) {
    let mut observation = fsl_replay::ObservationLimits::default();
    let observation_bounds = observation
        .bounds()
        .into_iter()
        .map(|(setting, _)| setting)
        .collect::<Vec<_>>();
    let mut supplied_observation = false;
    let mut only_observation = true;

    for (name, value) in limits.values() {
        let SettingName::Stage(setting) = name else {
            only_observation = false;
            continue;
        };
        if !observation_bounds.contains(setting) {
            only_observation = false;
            continue;
        }
        let updated = observation.set_bound(*setting, *value);
        debug_assert!(
            updated,
            "an observation bound listed by bounds() must be settable"
        );
        supplied_observation |= updated;
    }

    (
        supplied_observation.then_some(observation),
        only_observation,
    )
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
                Some(options::REQUEST_BYTES),
            )
        })
        .transpose()
    {
        Ok(clocks) => clocks,
        Err(refusal) => return clocks_refusal(Operation::Execute, refusal),
    };
    let (observation_limits, only_observation_limits) =
        observation_limits_override(&options.limits);
    let overrides = command::RequestOverrides {
        clocks: clocks.as_deref(),
        observation: observation_limits.as_ref(),
        ..command::RequestOverrides::default()
    };
    let request = native_run_request(&options.path, options.intake_limits.request_bytes);
    if request.as_ref().is_some_and(carries_extraction) {
        if !only_observation_limits {
            return CommandResult::input_refusal(
                Operation::Execute,
                "only observation limits are supported for an extracted Markdown clause run",
            );
        }
        let negotiation = match filament_driver::AnalyzeNegotiation::fsl() {
            Ok(negotiation) => negotiation,
            Err(error) => {
                return CommandResult::failure(
                    Category::InternalFailure,
                    format!("could not initialize FSL engine negotiation: {error:?}"),
                );
            }
        };
        return match command::run_with_providers_and_limits_and_overrides(
            &options.path,
            options.intake_limits,
            &overrides,
            &negotiation,
            match filament_driver::run_engines(Vec::new()) {
                Ok(engines) => engines,
                Err(error) => {
                    return CommandResult::failure(
                        Category::InternalFailure,
                        format!("could not initialize FSL engines: {error:?}"),
                    );
                }
            },
            &fsl_analyze::provider::ProviderOptions::default(),
            &Cancel::new(),
        ) {
            Ok(result) => {
                let render_sources = fsl_command::render_sources(result.sources());
                match result.outcome {
                    Some(document) => {
                        CommandResult::document(document).with_sources(render_sources)
                    }
                    None => native_result(result.value.as_value().clone(), result.exit_code),
                }
            }
            Err(error) => CommandResult::document(error.outcome_document(Operation::Execute)),
        };
    }
    let prepared =
        match command::prepare_run_with_overrides(&options.path, options.intake_limits, &overrides)
        {
            Ok(prepared) => prepared,
            Err(error) => {
                return CommandResult::document(error.outcome_document(Operation::Execute));
            }
        };
    let input = match prepared {
        PreparedRun::Function(prepared) => {
            if request.as_ref().is_some_and(carries_call_requirements) {
                let spine_limits = match requirement_run_limits(&options.limits) {
                    Ok(limits) => limits,
                    Err(message) => {
                        return CommandResult::input_refusal(Operation::Execute, message);
                    }
                };
                let render_sources =
                    fsl_inspect::RenderSources::from_sources([prepared.into_input().source]);
                return run_with_requirement_items(
                    &options.path,
                    render_sources,
                    provider_config,
                    options.providers.limits,
                    options.intake_limits,
                    spine_limits,
                    clocks,
                );
            }
            prepared.into_input()
        }
        PreparedRun::Clause(request) => {
            return run_clause_request(*request.into_request(), &options.limits);
        }
        PreparedRun::TlLowering(_) => {
            return CommandResult::input_refusal(
                Operation::Execute,
                "FR-931 tl_lowering requests are not run by the driver yet",
            );
        }
        PreparedRun::Package(_) => {
            let negotiation = match filament_driver::AnalyzeNegotiation::fsl() {
                Ok(negotiation) => negotiation,
                Err(error) => {
                    return CommandResult::failure(
                        Category::InternalFailure,
                        format!("could not initialize FSL engine negotiation: {error:?}"),
                    );
                }
            };
            return match command::run_with_providers_and_limits_and_overrides(
                &options.path,
                options.intake_limits,
                &overrides,
                &negotiation,
                match filament_driver::run_engines(Vec::new()) {
                    Ok(engines) => engines,
                    Err(error) => {
                        return CommandResult::failure(
                            Category::InternalFailure,
                            format!("could not initialize FSL engines: {error:?}"),
                        );
                    }
                },
                &fsl_analyze::provider::ProviderOptions::default(),
                &Cancel::new(),
            ) {
                Ok(result) => {
                    let render_sources = fsl_command::render_sources(result.sources());
                    match result.outcome {
                        Some(document) => {
                            CommandResult::document(document).with_sources(render_sources)
                        }
                        None => native_result(result.value.as_value().clone(), result.exit_code),
                    }
                }
                Err(error) => CommandResult::document(error.outcome_document(Operation::Execute)),
            };
        }
    };
    let command::FunctionRunInput {
        source,
        package_name,
        packages,
        dependencies,
        integer_division,
        mut call,
    } = input;
    // The execution request carries no division law, so a law in the request is refused
    // rather than dropped.
    if integer_division.is_some() {
        return CommandResult::input_refusal(
            Operation::Execute,
            "the request carries an integer_division law, which this run path does not apply",
        );
    }
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
        integer_division,
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

/// Run FSL's checked-package exporter, emit its outcome, and write target
/// text only after the request assembled successfully.
fn export(options: &ExportOptions, format: OutputFormat) -> CommandResult {
    let mut limits = SpineLimits::default();
    if let Err(error) = apply_export_limits(&options.limits, &mut limits) {
        return CommandResult::input_refusal(Operation::Export, error);
    }
    let overrides = command::RequestOverrides {
        output: options.output.as_deref(),
        limits: Some(&limits),
        ..command::RequestOverrides::default()
    };
    let Some(request_path) = options.request.as_deref() else {
        return CommandResult::input_refusal(Operation::Export, entry_usage(Verb::Export));
    };
    if has_export_request_member(request_path, RunLimits::default().request_bytes) == Some(false) {
        return CommandResult::input_refusal(
            Operation::Export,
            "the request must include an `export` member",
        );
    }
    let runner = match command::run_with_overrides(request_path, &overrides, &DriverNegotiation) {
        Ok(result) => result,
        Err(error) if matches!(error.cause, command::RunCause::OutputGivenTwice) => {
            return CommandResult::input_refusal(
                Operation::Export,
                "the request's export.output cannot be combined with --output",
            );
        }
        Err(error) => {
            return CommandResult::document(error.outcome_document(Operation::Export));
        }
    };
    let Some(document) = runner.outcome else {
        return CommandResult::failure(
            Category::InternalFailure,
            "FSL returned an export result without its outcome document",
        );
    };
    with_format(CommandResult::document(document), format)
}

/// Detect a valid native-run request that lacks the export member without
/// consuming an unbounded or malformed request before FSL handles it.
fn has_export_request_member(path: &Path, maximum_bytes: usize) -> Option<bool> {
    let file = File::open(path).ok()?;
    let read_limit = u64::try_from(maximum_bytes).ok()?.checked_add(1)?;
    let mut bytes = Vec::new();
    file.take(read_limit).read_to_end(&mut bytes).ok()?;
    if bytes.len() > maximum_bytes {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if value.get("format").and_then(serde_json::Value::as_str) != Some("native-run/1") {
        return None;
    }
    Some(value.get("request")?.as_object()?.contains_key("export"))
}

/// The export request does not settle routed capability items. This
/// negotiation is present only because the shared run adapter owns that
/// parameter; if it is called, preserve the candidate result deterministically.
struct DriverNegotiation;

impl fsl_analyze::analyze::document::Negotiate for DriverNegotiation {
    fn negotiate(
        &self,
        item: &fsl_analyze::analyze::ClaimItem,
        candidates: &fsl_route::CandidateOutcome,
    ) -> (
        fsl_route::routing::Disposition,
        Option<fsl_analyze::analyze::Diagnostic>,
    ) {
        use fsl_analyze::analyze::{Diagnostic, Payload};
        use fsl_route::routing::Disposition;
        use fsl_route::NegotiationOutcome;

        match candidates {
            fsl_route::CandidateOutcome::UnknownBackend(backend) => (
                Disposition::InvalidRequest,
                Some(Diagnostic {
                    code: NegotiationOutcome::UnknownBackend.code(),
                    payload: Payload::Backend(backend.clone()),
                }),
            ),
            fsl_route::CandidateOutcome::Candidates(set) => match set.candidates() {
                [candidate] => (Disposition::Supported(candidate.clone()), None),
                [] => (
                    Disposition::Unsupported,
                    Some(Diagnostic {
                        code: NegotiationOutcome::UnsupportedRequestedCapability.code(),
                        payload: Payload::Kind(item.kind),
                    }),
                ),
                _ => (
                    Disposition::InvalidRequest,
                    Some(Diagnostic::new(NegotiationOutcome::AmbiguousBackend.code())),
                ),
            },
        }
    }
}
/// Select FSL's extraction-aware run entry for a native-run request carrying
/// `program.extraction`. The bounded dispatch read does not open dependent
/// observation or source files; FSL remains their sole reader.
fn carries_extraction(request: &serde_json::Value) -> bool {
    request
        .get("program")
        .and_then(|program| program.get("extraction"))
        .is_some()
}

/// Whether the `native-run/1` request carries a call-level `requirements`
/// member.
fn carries_call_requirements(request: &serde_json::Value) -> bool {
    request
        .get("call")
        .and_then(|call| call.get("requirements"))
        .is_some()
}

/// The run's spine limits for a call with `requirements`. FSL's provider run
/// entry carries stage limits only: it reads no observation override for a
/// call and carries no accounting settings, so those are refused here rather
/// than dropped.
fn requirement_run_limits(limits: &LimitOverrides) -> Result<SpineLimits, String> {
    let mut spine = SpineLimits::default();
    for (name, value) in limits.values() {
        let accepted = match name {
            SettingName::Stage(setting) => spine.set_bound(*setting, *value),
            SettingName::Accounting(_) => false,
        };
        if !accepted {
            let error = filament_driver::LimitError::OperationUnavailable {
                setting: *name,
                operation: "run",
            };
            return Err(format!(
                "{}: FSL's provider run entry does not carry this setting on a call with `requirements`",
                error.message_for("run")
            ));
        }
    }
    Ok(spine)
}

/// The `request` member of the bounded `native-run/1` document at `path`.
fn native_run_request(path: &Path, request_limit: usize) -> Option<serde_json::Value> {
    let read_limit = u64::try_from(request_limit).ok()?;
    let file = File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(read_limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > request_limit {
        return None;
    }
    let mut document = serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
    if document.get("format")?.as_str() != Some("native-run/1") {
        return None;
    }
    Some(document.get_mut("request")?.take())
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
                    return CommandResult::failure(Category::InternalFailure, error.to_string());
                }
            }
        }
    };
    CommandResult::document(document)
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
        .map(|law| admit_integer_division_law(law, options::INTEGER_DIVISION))
        .transpose()
    {
        Ok(law) => law,
        Err(refusal) => return *refusal,
    };
    let mut limits = SpineLimits::default();
    if let Err(error) = apply_frontend_limits(&options.limits, &mut limits, "check") {
        return CommandResult::input_refusal(Operation::Check, error);
    }
    let extensions = match options
        .extensions
        .as_deref()
        .map(extension_set::read)
        .transpose()
    {
        Ok(entries) => entries.unwrap_or_default(),
        Err(refusal) => return set_refusal(Operation::Check, refusal),
    };
    if options.markdown {
        return check_markdown_source(options, limits, integer_division, &extensions);
    }
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure_for(Operation::Check, Code::IoError, message);
        }
    };
    #[cfg(all(feature = "tc018-test-adapter", debug_assertions))]
    {
        let source = options
            .source_identity
            .clone()
            .unwrap_or_else(|| SourceIdentity::new("filament", &options.path));
        tc018_record_check_request(&options, &source, &bytes, limits);
    }
    drop(bytes);
    // FSL reads the model, library and extension files itself and refuses an
    // identity the unit does not select (FSL FR-866).
    let inputs = command::CheckInputs {
        models: options.model,
        libraries: options.libraries,
        limits,
        dependent_bytes: options.dependent_bytes,
        integer_division,
        extensions,
    };
    check_result(&command::check_outcome(Path::new(&options.path), &inputs))
}

/// Check each native fence in an explicitly named Markdown document (FR-662).
fn check_markdown_source(
    options: CheckOptions,
    limits: SpineLimits,
    integer_division: Option<fsl_replay::AdmittedIntegerDivision>,
    extensions: &[command::CheckExtensionInput],
) -> CommandResult {
    if !options.model.is_empty() || !options.libraries.is_empty() {
        return CommandResult::input_refusal(
            Operation::Check,
            format!(
                "{} and {} are not supported with {}",
                options::MODEL,
                options::LIBRARY,
                options::MARKDOWN
            ),
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
    let catalog = if extensions.is_empty() {
        None
    } else {
        match extension_set::catalog(extensions) {
            Ok(catalog) => Some(catalog),
            Err(refusal) => return set_refusal(Operation::Check, refusal),
        }
    };
    let request = NativeFenceRequest {
        original: &original,
        packages: &packages,
        dependencies: &dependencies,
        extensions: catalog.as_ref().map(|catalog| ProfileCatalogInput {
            catalog,
            limits: PackageLimits::default(),
        }),
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

/// The refusal of an `--extensions` set document or a definition it names.
fn set_refusal(operation: Operation, refusal: extension_set::SetRefusal) -> CommandResult {
    CommandResult::catalog_failure_for(operation, refusal.code, refusal.message)
}

/// Parse one source through FSL and serialize its typed parse outcome.
fn parse_source(options: SourceOptions) -> CommandResult {
    let mut limits = SpineLimits::default();
    if let Err(error) = apply_frontend_limits(&options.limits, &mut limits.source, "parse") {
        return CommandResult::input_refusal(Operation::Parse, error);
    }
    let catalog = match options
        .extensions
        .as_deref()
        .map(|path| extension_set::read(path).and_then(|entries| extension_set::catalog(&entries)))
        .transpose()
    {
        Ok(catalog) => catalog,
        Err(refusal) => return set_refusal(Operation::Parse, refusal),
    };
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
    let request = ParseRequest {
        source: &source,
        path: &display_path,
        bytes: &bytes,
    };
    let cancel = Cancel::new();
    let result = match &catalog {
        Some(catalog) => parse_selecting_extensions(
            &request,
            ProfileCatalogInput {
                catalog,
                limits: PackageLimits::default(),
            },
            limits.source,
            &cancel,
        ),
        None => parse(&request, limits.source, &cancel),
    };
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
    fn from_scanned(verb: Verb, scanned: &Scanned) -> Result<Self, OptionRefusal> {
        let mut providers = ProviderArguments::default();
        providers.take_all(scanned)?;
        let [path] = scanned.positionals.as_slice() else {
            return Err(entry_usage(verb).into());
        };
        Ok(Self {
            path: PathBuf::from(path),
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
            );
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
        let engines = match filament_driver::run_engines(std::mem::take(&mut providers.engines)) {
            Ok(engines) => engines,
            Err(error) => {
                providers.close();
                return Err(format!("{error:?}"));
            }
        };
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
            );
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

/// `fsl run` with a call-level `requirements` member: use FSL FR-961's
/// provider entry even when the caller supplied no process providers, and
/// carry the same run limits and request overrides as the ordinary run path.
fn run_with_requirement_items(
    path: &Path,
    render_sources: fsl_inspect::RenderSources,
    config: crate::provider_config::ProviderConfig,
    provider_limits: fsl_analyze::provider::ProviderLimits,
    intake_limits: RunLimits,
    spine_limits: SpineLimits,
    clocks: Option<Box<serde_json::value::RawValue>>,
) -> CommandResult {
    let path = path.to_path_buf();
    let ran = with_started_providers(
        config,
        provider_limits,
        false,
        move |negotiation, engines, options, cancel| {
            let overrides = command::RequestOverrides {
                clocks: clocks.as_deref(),
                limits: Some(&spine_limits),
                ..command::RequestOverrides::default()
            };
            command::run_with_providers_and_limits_and_overrides(
                &path,
                intake_limits,
                &overrides,
                negotiation,
                engines,
                options,
                cancel,
            )
        },
    );
    let written = match ran {
        Ok(Ok(written)) => written,
        Ok(Err(message)) => return CommandResult::input_refusal(Operation::Execute, message),
        Err(error) => {
            return CommandResult::failure(
                Category::InternalFailure,
                format!("could not initialize process signal handling: {error}"),
            );
        }
    };
    match written {
        Ok(result) => match result.outcome {
            Some(document) => CommandResult::document(document).with_sources(render_sources),
            None => native_result(result.value.as_value().clone(), result.exit_code),
        },
        Err(error) => CommandResult::document(error.outcome_document(Operation::Execute))
            .with_sources(render_sources),
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
            );
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
    fn from_scanned(scanned: &Scanned) -> Result<Self, OptionRefusal> {
        let input = scanned.value(options::INPUT).map(PathBuf::from);
        let mut providers = ProviderArguments::default();
        providers.take_all(scanned)?;
        let [path] = scanned.positionals.as_slice() else {
            return Err(entry_usage(Verb::Monitor).into());
        };
        Ok(Self {
            path: PathBuf::from(path),
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
                );
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
            );
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
///
/// FR-002 decides a verb's invocation in order: the root `--format` checks
/// ran in [`parse_invocation`]; then an option the verb's table entry does
/// not list refuses, a help option answers, the verb's operand and value
/// checks refuse, and only then does the operation run.
fn dispatch(verb: Verb, format: OutputFormat, raw_arguments: &[OsString]) -> CommandResult {
    let Some(entry) = options::entry(verb.name()) else {
        return unrouted_verb(verb, format);
    };
    let operation = verb.operation();
    let scanned = match options::scan(entry, raw_arguments) {
        Ok(Scan::Help) => return CommandResult::plain(entry.help()),
        Ok(Scan::Options(scanned)) => scanned,
        Err(options::Refusal::Usage(message)) => {
            return CommandResult::input_refusal(operation, message);
        }
        Err(options::Refusal::Repeated(message)) => {
            return CommandResult::failure(Code::InvalidRequest.category(), message);
        }
    };
    match verb {
        Verb::Analyze | Verb::Replay => match RequestOptions::from_scanned(verb, &scanned) {
            Ok(options) if verb == Verb::Analyze => analyze(&options, format),
            Ok(options) => replay(&options, format),
            Err(error) => option_refusal(operation, &error),
        },
        Verb::Monitor => match MonitorOptions::from_scanned(&scanned) {
            Ok(options) => monitor(&options, format),
            Err(error) => option_refusal(operation, &error),
        },
        Verb::Run => match RunOptions::from_scanned(&scanned) {
            Ok(options) => with_format(run(options), format),
            Err(error) => option_refusal(operation, &error),
        },
        Verb::Export => match ExportOptions::from_scanned(&scanned) {
            Ok(options) => with_format(export(&options, format), format),
            Err(error) => CommandResult::input_refusal(operation, error),
        },
        Verb::Compile => match CompileOptions::from_scanned(&scanned) {
            Ok(options) => with_format(compile(options), format),
            Err(error) => CommandResult::input_refusal(operation, error),
        },
        Verb::Check => match CheckOptions::from_scanned(&scanned) {
            Ok(options) => with_format(check_source(options), format),
            Err(error) => option_refusal(operation, &error),
        },
        Verb::Parse => match SourceOptions::from_scanned(verb, &scanned) {
            Ok(options) => with_format(parse_source(options), format),
            Err(error) => CommandResult::input_refusal(operation, error),
        },
        Verb::Format => match SourceOptions::from_scanned(verb, &scanned) {
            Ok(options) => {
                source_preflight(&options, operation, fsl_cst::Limits::default().source_bytes)
                    .unwrap_or_else(|| format_source(options))
            }
            Err(error) => CommandResult::input_refusal(operation, error),
        },
        Verb::Inspect => match InspectOptions::from_scanned(&scanned) {
            Ok(options) => inspect(&options, format),
            Err(error) => CommandResult::input_refusal(operation, error),
        },
        Verb::Prove | Verb::Generate | Verb::Render => unrouted_verb(verb, format),
    }
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
        | Verb::Export
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
            let mut rendered = staged.into_value().as_str().to_owned();
            if let Some(hint) = observation_input_limit_hint(document) {
                if !rendered.is_empty() {
                    rendered.push('\n');
                }
                rendered.push_str(&hint);
            }
            result.text_output = Some(rendered);
            result
        }
        Err(_) => CommandResult::failure(
            Category::InternalFailure,
            "could not render the outcome document as text",
        ),
    }
}

/// Add the driver's accepted CLI spelling when a request-provided observation
/// input limit stops a clause run. The structured FSL result remains unchanged.
fn observation_input_limit_hint(document: &OutcomeDocument) -> Option<String> {
    let value = serde_json::to_value(document).ok()?;
    let items = value.get("items")?.as_array()?;
    let cause = items.iter().find_map(|item| {
        let cause = item.get("cause")?;
        (cause.get("code")?.as_str()? == "stage_limit_exceeded"
            && cause.get("kind")?.as_str()? == "input-bytes-exceeded"
            && cause.get("setting")?.as_str()? == "observation.input_bytes")
            .then(|| cause.get("bound")?.as_str())
            .flatten()
    })?;
    Some(format!(
        "Hint: retry with --limit observation.input_bytes={cause}"
    ))
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
    let plain = result.document.is_none() && result.analyze_document.is_none();
    let primary = if plain {
        result.text_output.clone()
    } else if format == OutputFormat::Json {
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
        write_result_diagnostics(&result, super::OutputFormat::Text, &mut stderr)
            .expect("write FSL diagnostic");
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
        options, parse_invocation, read_source, CheckOptions, ExecutionEngine, ExportOptions,
        Invocation, LimitOverrides, OptionRefusal, OutputFormat, RunOptions, Scan, SpineLimits,
        Verb,
    };
    use fsl_foundation::{Setting, SettingName};
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    fn scanned(verb: &str, arguments: &[OsString]) -> Result<crate::options::Scanned, String> {
        let entry = crate::options::entry(verb).expect("verb has an option table entry");
        match crate::options::scan(entry, arguments)
            .map_err(|refusal| refusal.message().to_owned())?
        {
            crate::options::Scan::Options(scanned) => Ok(scanned),
            crate::options::Scan::Help => Err("help requested".to_owned()),
        }
    }

    fn parse_export(arguments: &[OsString]) -> Result<ExportOptions, String> {
        ExportOptions::from_scanned(&scanned("export", arguments)?)
    }

    fn parse_check(arguments: &[OsString]) -> Result<CheckOptions, OptionRefusal> {
        let scanned = scanned("check", arguments)?;
        CheckOptions::from_scanned(&scanned)
    }

    fn parse_run(arguments: &[OsString]) -> Result<RunOptions, OptionRefusal> {
        let scanned = scanned("run", arguments)?;
        RunOptions::from_scanned(&scanned)
    }

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

    #[test]
    fn export_options_accept_output_and_one_request_file() {
        let parsed = parse_export(&[
            OsString::from("--output"),
            OsString::from("model.tla"),
            OsString::from("request.json"),
        ])
        .expect("valid export arguments");
        assert_eq!(parsed.output.as_deref(), Some(Path::new("model.tla")));
        assert_eq!(parsed.request.as_deref(), Some(Path::new("request.json")));
        assert!(parsed.limits.values().is_empty());
        let request_only = parse_export(&[OsString::from("request.json")])
            .expect("the request may name its own output");
        assert_eq!(request_only.output, None);
        assert!(parse_export(&[
            OsString::from("--output"),
            OsString::from("model.tla"),
            OsString::from("--output"),
            OsString::from("other.tla"),
            OsString::from("request.json"),
        ])
        .is_err());
    }

    #[test]
    fn export_options_accept_repeated_export_and_compile_limits() {
        let parsed = parse_export(&[
            OsString::from("--output"),
            OsString::from("model.tla"),
            OsString::from("request.json"),
            OsString::from("--limit"),
            OsString::from("export.output_bytes=4096"),
            OsString::from("--limit"),
            OsString::from("export.nodes=20"),
            OsString::from("--limit"),
            OsString::from("s1.input_bytes=1024"),
        ])
        .expect("valid export limits");
        let mut limits = SpineLimits::default();
        parsed
            .limits
            .apply_to_export(&mut limits)
            .expect("supported export and compile limits");
        assert_eq!(limits.export.output_bytes, Some(4096));
        assert_eq!(limits.export.nodes, Some(20));
        assert_eq!(limits.source.source_bytes, 1024);
        assert!(parse_export(&[
            OsString::from("--output"),
            OsString::from("model.tla"),
            OsString::from("request.json"),
            OsString::from("--limit"),
            OsString::from("export.output_bytes=-1"),
        ])
        .is_err());
        assert!(parse_export(&[
            OsString::from("request.json"),
            OsString::from("--limit"),
            OsString::from("unknown.limit=5"),
        ])
        .is_err());
    }

    #[test]
    fn export_options_route_evidence_and_reject_conflicting_inputs() {
        let evidence = parse_export(&[
            OsString::from("--evidence"),
            OsString::from("evidence.json"),
        ])
        .expect("evidence request");
        assert_eq!(
            evidence.request.as_deref(),
            Some(Path::new("evidence.json"))
        );
        assert!(parse_export(&[
            OsString::from("--evidence"),
            OsString::from("evidence.json"),
            OsString::from("--output"),
            OsString::from("model.tla"),
        ])
        .is_err());
        assert!(parse_export(&[
            OsString::from("request.json"),
            OsString::from("--evidence"),
            OsString::from("evidence.json"),
        ])
        .is_err());
    }

    #[test]
    fn export_without_an_export_member_is_invalid_input() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let directory = tempfile::tempdir().expect("temporary directory");
        let output = directory.path().join("model.tla");
        let result = super::export(
            &ExportOptions {
                output: Some(output.clone()),
                request: Some(fixtures.join("frontend-run.json")),
                limits: LimitOverrides::default(),
            },
            OutputFormat::Json,
        );
        assert_eq!(result.category, super::Category::Refusal);
        assert!(result
            .diagnostic
            .as_deref()
            .is_some_and(|message| message.contains("must include an `export` member")));
        assert!(!output.exists());
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
            OsString::from("example/one=model.json"),
            OsString::from("--model"),
            OsString::from("example/two=other=model.json"),
            OsString::from("--library"),
            OsString::from("test/geometry=geometry.native"),
            OsString::from("--library"),
            OsString::from("test/units=units.native"),
            OsString::from("--dependent-bytes"),
            OsString::from("1234"),
        ];
        let options = parse_check(&args).expect("valid check inputs");
        assert_eq!(options.path, "unit.fsl");
        assert_eq!(
            options.model,
            [
                ("example/one".to_owned(), PathBuf::from("model.json")),
                ("example/two".to_owned(), PathBuf::from("other=model.json")),
            ]
        );
        assert_eq!(
            options.libraries,
            [
                ("test/geometry".to_owned(), PathBuf::from("geometry.native")),
                ("test/units".to_owned(), PathBuf::from("units.native")),
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
                OsString::from("example/one=one.json"),
                OsString::from("--model"),
                OsString::from("example/one=two.json"),
            ],
            vec![
                OsString::from("unit.fsl"),
                OsString::from("--model"),
                OsString::from("one.json"),
            ],
            vec![
                OsString::from("unit.fsl"),
                OsString::from("--model"),
                OsString::from("=one.json"),
            ],
            vec![
                OsString::from("unit.fsl"),
                OsString::from("--model"),
                OsString::from("example/one="),
            ],
        ] {
            assert!(parse_check(&args).is_err(), "accepted {args:?}");
        }
    }

    /// Trace: FR-002-AC-24; the table refuses a repeat of an option it does not
    /// mark repeatable, for flags and valued options, before the verb checks run.
    #[test]
    fn the_table_refuses_a_repeated_non_repeatable_option() {
        for arguments in [
            vec!["--markdown", "--markdown", "a", "b", "x.md"],
            vec![
                "--integer-division",
                "floor",
                "--integer-division",
                "floor",
                "u.fsl",
            ],
            vec!["--extensions", "s.json", "--extensions", "s.json", "u.fsl"],
            vec!["--dependent-bytes", "1", "--dependent-bytes", "2", "u.fsl"],
        ] {
            let arguments: Vec<OsString> = arguments.into_iter().map(OsString::from).collect();
            let entry = options::entry("check").expect("check entry");
            let refusal = options::scan(entry, &arguments).expect_err("a repeated option");
            assert_eq!(
                matches!(refusal, options::Refusal::Repeated(_)),
                arguments[0] == "--integer-division",
                "{arguments:?}"
            );
            assert!(refusal.message().contains("may be supplied only once"));
        }
        let repeatable = [
            "--model",
            "a=1",
            "--model",
            "b=2",
            "--library",
            "c=3",
            "--library",
            "d=4",
            "--limit",
            "x=1",
            "--limit",
            "y=2",
            "--format",
            "json",
            "--format",
            "text",
        ];
        let arguments: Vec<OsString> = repeatable.into_iter().map(OsString::from).collect();
        let entry = options::entry("check").expect("check entry");
        assert!(matches!(
            options::scan(entry, &arguments),
            Ok(Scan::Options(_))
        ));
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
        let options = parse_run(&args).expect("valid run options");
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
        let options = parse_run(&args).expect("valid provider config path");
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
            parse_run(&missing).err(),
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
        assert!(matches!(
            parse_run(&duplicate).err(),
            Some(OptionRefusal::Usage(message))
                if message.starts_with("--provider-config may be supplied only once")
        ));
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
            parse_run(&args).err(),
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
        assert!(parse_run(&args).is_err());
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

    fn commands() -> Vec<(&'static str, &'static str)> {
        Verb::ALL
            .into_iter()
            .map(|verb| (verb.name(), verb.help_description()))
            .collect()
    }

    /// Trace: FR-002-AC-24; a table copy without an option makes the parser
    /// refuse it and the generated help omit it.
    #[test]
    fn removing_a_table_option_removes_it_from_parser_and_help() {
        for entry in options::table() {
            for option in &entry.options {
                let mut arguments = vec![OsString::from(option.name)];
                if option.value.is_some() {
                    arguments.push(OsString::from("value"));
                }
                assert!(
                    matches!(options::scan(entry, &arguments), Ok(Scan::Options(_))),
                    "{} rejects its own {}",
                    entry.verb,
                    option.name
                );
                let copy = entry.without(option.name);
                let refusal = options::scan(&copy, &arguments)
                    .expect_err("the parser refuses an option outside its entry");
                assert!(refusal.message().contains(option.name), "{refusal:?}");
                let tables: Vec<options::VerbEntry> = options::table()
                    .iter()
                    .map(|other| {
                        if other.verb == entry.verb {
                            copy.clone()
                        } else {
                            other.clone()
                        }
                    })
                    .collect();
                let help = options::root_help(&tables, &commands());
                let heading = format!("{} options:", entry.verb);
                let section = help
                    .split("\n\n")
                    .find(|section| section.starts_with(&heading))
                    .expect("the verb section");
                assert!(
                    section.lines().all(|line| line != option.help_line()),
                    "{} help still lists {}",
                    entry.verb,
                    option.name
                );
            }
        }
    }

    /// Trace: FR-002-AC-24; no option name is spelled outside the table.
    #[test]
    fn no_option_literal_appears_outside_the_table() {
        let literal = regex::Regex::new(r#""--?[A-Za-z]"#).expect("pattern");
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for entry in std::fs::read_dir(&directory).expect("read src") {
            let path = entry.expect("directory entry").path();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if name == "options.rs" || path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("read source");
            let production = source.split("\n#[cfg(test)]").next().unwrap_or(&source);
            for (number, line) in production.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                assert!(
                    !literal.is_match(line),
                    "{name}:{} spells an option outside the table: {line}",
                    number + 1
                );
            }
        }
    }
}

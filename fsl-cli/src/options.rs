// SPDX-License-Identifier: AGPL-3.0-or-later
//! The one option table of the `fsl` driver (FR-002-AC-24).
//!
//! Every verb that executes operations has one [`VerbEntry`]: its usage
//! lines and its options. The verb parsers scan their arguments against the
//! entry and root and per-verb help are generated from it, so a parser
//! cannot accept an option the help omits or the reverse. No other part of
//! `fsl-cli` spells an option name; they use the constants below.
//!
//! This file depends on nothing else in the crate: the integration tests
//! include it by path to read the same table the binary reads.

use std::ffi::{OsStr, OsString};
use std::sync::OnceLock;

/// `check --markdown`.
pub const MARKDOWN: &str = "--markdown";
/// `check --model <identity>=<file>` (FSL FR-866).
pub const MODEL: &str = "--model";
/// `check --library <identity>=<file>` (FSL FR-866).
pub const LIBRARY: &str = "--library";
/// `--dependent-bytes <bytes>`.
pub const DEPENDENT_BYTES: &str = "--dependent-bytes";
/// `--request-bytes <bytes>`.
pub const REQUEST_BYTES: &str = "--request-bytes";
/// `--limit name=value`.
pub const LIMIT: &str = "--limit";
/// `check --integer-division <law>`.
pub const INTEGER_DIVISION: &str = "--integer-division";
/// `parse` and `check` `--extensions <file>` (FSpec FR-562).
pub const EXTENSIONS: &str = "--extensions";
/// The root output selector.
pub const FORMAT: &str = "--format";
/// `compile` and `run` `--clocks <json-file>`.
pub const CLOCKS: &str = "--clocks";
/// `compile` and `export` `--output <file>`.
pub const OUTPUT: &str = "--output";
/// `export --evidence <request-file>`.
pub const EVIDENCE: &str = "--evidence";
/// `run --engine interpreter|aot|jit`.
pub const ENGINE: &str = "--engine";
/// `--provider-config <path>`.
pub const PROVIDER_CONFIG: &str = "--provider-config";
/// `--provider-limit name=value`.
pub const PROVIDER_LIMIT: &str = "--provider-limit";
/// `monitor --input <file>`.
pub const INPUT: &str = "--input";
/// `inspect --handoff <directory>`.
pub const HANDOFF: &str = "--handoff";

/// The root help spellings, accepted as the first argument and, after a
/// verb, as a help request.
pub const HELP_OPTIONS: [&str; 2] = ["--help", "-h"];
/// The root-only version spellings.
pub const VERSION_OPTIONS: [&str; 2] = ["--version", "-V"];
/// The word form of root help.
pub const HELP_WORD: &str = "help";

/// One option of a verb: its name, value form and whether it may repeat.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptionSpec {
    /// The option as typed, such as `--model`.
    pub name: &'static str,
    /// The value form shown in help, such as `<file>`; `None` for a flag.
    pub value: Option<&'static str>,
    /// What a refusal of a missing value says is required.
    pub needs: &'static str,
    /// Whether the option may be given more than once.
    pub repeatable: bool,
    /// Whether a repeat refuses with no stdout byte (FSL FR-767) rather than as a
    /// typed `invalid-request` document.
    pub quiet_repeat: bool,
}

impl OptionSpec {
    const fn flag(name: &'static str) -> Self {
        Self {
            name,
            value: None,
            needs: "",
            repeatable: false,
            quiet_repeat: false,
        }
    }

    const fn valued(name: &'static str, value: &'static str, needs: &'static str) -> Self {
        Self {
            name,
            value: Some(value),
            needs,
            repeatable: false,
            quiet_repeat: false,
        }
    }

    const fn repeated(mut self) -> Self {
        self.repeatable = true;
        self
    }

    const fn quiet_repeat(mut self) -> Self {
        self.quiet_repeat = true;
        self
    }

    /// The option's line in a `<verb> options:` section.
    #[must_use]
    pub fn help_line(&self) -> String {
        let mut line = format!("  {}", self.name);
        if let Some(value) = self.value {
            line.push(' ');
            line.push_str(value);
        }
        if self.repeatable {
            line.push_str(" (repeatable)");
        }
        line
    }
}

/// One verb's usage lines and options.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerbEntry {
    /// The verb as typed.
    pub verb: &'static str,
    /// Each usage line after `fsl `.
    pub usage: Vec<&'static str>,
    /// The verb's options, in help order.
    pub options: Vec<OptionSpec>,
}

impl VerbEntry {
    /// The entry's option named `name`.
    #[must_use]
    pub fn option(&self, name: &str) -> Option<&OptionSpec> {
        self.options.iter().find(|option| option.name == name)
    }

    /// A copy of the entry without the option named `name`.
    #[must_use]
    #[cfg(test)]
    pub fn without(&self, name: &str) -> Self {
        let mut copy = self.clone();
        copy.options.retain(|option| option.name != name);
        copy
    }

    /// The usage refusal text: each usage line, `fsl`-prefixed.
    #[must_use]
    pub fn usage_refusal(&self) -> String {
        format!("usage: fsl {}", self.usage.join(" | fsl "))
    }

    /// The refusal for an option the entry does not list: the nearest listed
    /// option when the typed one looks like a typo, and every option the verb
    /// accepts.
    #[must_use]
    pub fn unknown_option(&self, typed: &str) -> String {
        let mut message = format!("unknown {} option `{typed}`", self.verb);
        if let Some(near) = nearest(typed, self.options.iter().map(|option| option.name)) {
            message.push_str(&format!("; did you mean `{near}`"));
        }
        let names: Vec<&str> = self.options.iter().map(|option| option.name).collect();
        message.push_str(&format!("; {} accepts {}", self.verb, names.join(", ")));
        message
    }

    /// The `<verb> options:` section, each line ending in a newline.
    #[must_use]
    pub fn options_section(&self) -> String {
        let mut section = format!("{} options:\n", self.verb);
        for option in &self.options {
            section.push_str(&option.help_line());
            section.push('\n');
        }
        section
    }

    /// The text `fsl <verb> --help` writes: the usage lines, one blank line
    /// and the options section.
    #[must_use]
    pub fn help(&self) -> String {
        let mut help = String::new();
        for (index, line) in self.usage.iter().enumerate() {
            help.push_str(if index == 0 {
                "Usage: fsl "
            } else {
                "       fsl "
            });
            help.push_str(line);
            help.push('\n');
        }
        help.push('\n');
        help.push_str(&self.options_section());
        help
    }

    fn names(&self, argument: &OsStr) -> bool {
        argument
            .to_str()
            .is_some_and(|text| self.option(text).is_some() || HELP_OPTIONS.contains(&text))
    }
}

const PATH: &str = "a path";
const FORMAT_VALUE: &str = "text|json";

fn limit() -> OptionSpec {
    OptionSpec::valued(LIMIT, "name=value", "name=value").repeated()
}

fn format() -> OptionSpec {
    OptionSpec::valued(FORMAT, FORMAT_VALUE, "text, json, or dot")
}

fn provider_config() -> OptionSpec {
    OptionSpec::valued(PROVIDER_CONFIG, "<path>", PATH)
}

fn provider_limit() -> OptionSpec {
    OptionSpec::valued(PROVIDER_LIMIT, "name=value", "name=value").repeated()
}

fn build() -> Vec<VerbEntry> {
    let bytes = |name| OptionSpec::valued(name, "<bytes>", "a non-negative byte count");
    vec![
        VerbEntry {
            verb: "parse",
            usage: vec!["parse [options] <authority> <identity> <source-path>"],
            options: vec![
                limit(),
                OptionSpec::valued(EXTENSIONS, "<file>", PATH),
                format(),
            ],
        },
        VerbEntry {
            verb: "format",
            usage: vec!["format [options] <authority> <identity> <source-path>"],
            options: vec![limit(), format()],
        },
        VerbEntry {
            verb: "check",
            usage: vec![
                "check [options] <source-path>",
                "check [options] <authority> <identity> <source-path>",
            ],
            options: vec![
                OptionSpec::flag(MARKDOWN),
                OptionSpec::valued(MODEL, "<identity>=<file>", "<identity>=<file>").repeated(),
                OptionSpec::valued(LIBRARY, "<identity>=<file>", "<identity>=<file>").repeated(),
                bytes(DEPENDENT_BYTES),
                limit(),
                OptionSpec::valued(INTEGER_DIVISION, "euclidean|floor|truncating", "a law")
                    .quiet_repeat(),
                OptionSpec::valued(EXTENSIONS, "<file>", PATH),
                format(),
            ],
        },
        VerbEntry {
            verb: "compile",
            usage: vec!["compile [options] <request-file>"],
            options: vec![
                OptionSpec::valued(CLOCKS, "<json-file>", PATH),
                OptionSpec::valued(OUTPUT, "<file>", PATH),
                limit(),
                format(),
            ],
        },
        VerbEntry {
            verb: "run",
            usage: vec!["run [options] <request-file>"],
            options: vec![
                bytes(REQUEST_BYTES),
                bytes(DEPENDENT_BYTES),
                limit(),
                OptionSpec::valued(ENGINE, "interpreter|aot|jit", "interpreter, aot, or jit"),
                provider_config(),
                provider_limit(),
                OptionSpec::valued(CLOCKS, "<json-file>", PATH),
                format(),
            ],
        },
        VerbEntry {
            verb: "export",
            usage: vec![
                "export [options] <request-file>",
                "export --evidence <request-file>",
            ],
            options: vec![
                OptionSpec::valued(OUTPUT, "<file>", PATH),
                OptionSpec::valued(EVIDENCE, "<request-file>", "a request file"),
                limit(),
                format(),
            ],
        },
        VerbEntry {
            verb: "analyze",
            usage: vec!["analyze [options] <request-file>"],
            options: vec![limit(), provider_config(), provider_limit(), format()],
        },
        VerbEntry {
            verb: "monitor",
            usage: vec!["monitor [options] <request-file>"],
            options: vec![
                OptionSpec::valued(INPUT, "<file>", PATH),
                limit(),
                provider_config(),
                provider_limit(),
                format(),
            ],
        },
        VerbEntry {
            verb: "replay",
            usage: vec!["replay [options] <request-file>"],
            options: vec![limit(), provider_config(), provider_limit(), format()],
        },
        VerbEntry {
            verb: "inspect",
            usage: vec!["inspect [options] --handoff <directory>"],
            options: vec![
                OptionSpec::valued(HANDOFF, "<directory>", "a directory operand"),
                OptionSpec::valued(FORMAT, "text|json|dot", "text, json, or dot"),
            ],
        },
    ]
}

/// The entry of a recognized verb this build does not route. Its options
/// refuse as unknown like a routed verb's, so an unknown option exits 20
/// before the verb answers unsupported; it has no help section.
#[must_use]
pub fn unrouted_entry(verb: &str) -> Option<&'static VerbEntry> {
    static ENTRIES: OnceLock<Vec<VerbEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(|| {
            vec![VerbEntry {
                verb: "prove",
                usage: vec!["prove [options] <request-file>"],
                options: vec![format()],
            }]
        })
        .iter()
        .find(|entry| entry.verb == verb)
}

/// Every verb that executes operations, in `Commands:` order.
pub fn table() -> &'static [VerbEntry] {
    static TABLE: OnceLock<Vec<VerbEntry>> = OnceLock::new();
    TABLE.get_or_init(build)
}

/// The entry of `verb`, `None` for a verb this build does not route.
#[must_use]
pub fn entry(verb: &str) -> Option<&'static VerbEntry> {
    table().iter().find(|entry| entry.verb == verb)
}

/// The arguments of one invocation, scanned against a verb's entry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Scanned {
    /// The arguments that are not options or option values, in order.
    pub positionals: Vec<OsString>,
    /// Each option given, with its value, in the order given.
    pub options: Vec<(&'static str, Option<OsString>)>,
}

/// Why a scan refused the arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// A usage refusal: an unknown option, or a missing value.
    Usage(String),
    /// An option the table marks `quiet_repeat` given twice: FSL FR-767 writes
    /// this refusal to stderr with no stdout byte.
    Repeated(String),
}

impl Refusal {
    /// The refusal text.
    #[must_use]
    #[cfg(test)]
    pub fn message(&self) -> &str {
        match self {
            Self::Usage(message) | Self::Repeated(message) => message,
        }
    }
}

/// What a scan found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Scan {
    /// A help option was given in option position.
    Help,
    /// The arguments hold no help option.
    Options(Scanned),
}

/// Scan `arguments` against `entry`, deciding refusals in FR-002's order:
/// an option the entry does not list refuses first, a help option answers
/// next, and a value check that failed refuses last. A repeat of a
/// non-repeatable option is such a value check (`--format` is exempt: the root
/// parser validates it and the last one wins). A value option takes
/// the following argument unless that argument is an option of the entry or
/// a help option; an empty argument is no value.
pub fn scan(entry: &VerbEntry, arguments: &[OsString]) -> Result<Scan, Refusal> {
    let mut scanned = Scanned::default();
    let mut help = false;
    let mut deferred = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        index += 1;
        let Some(text) = argument.to_str().filter(|text| text.starts_with('-')) else {
            scanned.positionals.push(argument.clone());
            continue;
        };
        if HELP_OPTIONS.contains(&text) {
            help = true;
            continue;
        }
        let Some(spec) = entry.option(text) else {
            return Err(Refusal::Usage(entry.unknown_option(text)));
        };
        if !spec.repeatable
            && spec.name != FORMAT
            && scanned.options.iter().any(|(given, _)| *given == spec.name)
        {
            deferred.get_or_insert_with(|| {
                let message = format!(
                    "{} may be supplied only once; {}",
                    spec.name,
                    entry.usage_refusal()
                );
                if spec.quiet_repeat {
                    Refusal::Repeated(message)
                } else {
                    Refusal::Usage(message)
                }
            });
        }
        if spec.value.is_none() {
            scanned.options.push((spec.name, None));
            continue;
        }
        match arguments
            .get(index)
            .filter(|value| !value.is_empty() && !entry.names(value))
        {
            Some(value) => {
                scanned.options.push((spec.name, Some(value.clone())));
                index += 1;
            }
            None => {
                deferred.get_or_insert_with(|| {
                    Refusal::Usage(format!("{} requires {}", spec.name, spec.needs))
                });
            }
        }
    }
    if help {
        return Ok(Scan::Help);
    }
    match deferred {
        Some(refusal) => Err(refusal),
        None => Ok(Scan::Options(scanned)),
    }
}

impl Scanned {
    /// The values given for `name`, in order.
    pub fn values<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a OsString> {
        self.options
            .iter()
            .filter(move |(given, _)| *given == name)
            .filter_map(|(_, value)| value.as_ref())
    }

    /// Whether the flag `name` was given.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.options.iter().any(|(given, _)| *given == name)
    }

    /// The first value given for `name`. The scan already refused a repeat of
    /// an option the table does not mark repeatable.
    #[must_use]
    pub fn value<'a>(&'a self, name: &'a str) -> Option<&'a OsString> {
        self.values(name).next()
    }
}

/// The root help text, generated from the table: invocation syntax, the
/// verbs, the usage lines, the root options and each verb's options section.
#[must_use]
pub fn root_help(entries: &[VerbEntry], commands: &[(&str, &str)]) -> String {
    let mut help = String::from(concat!(
        "Usage: fsl <command> [arguments] [--format text|json|dot]\n",
        "       fsl --help|-h|help\n",
        "       fsl --version|-V\n\n",
        "Commands:\n",
    ));
    for (name, description) in commands {
        help.push_str(&format!("  {name:<10} {description}\n"));
    }
    help.push_str("\nSource operands:\n");
    for entry in commands
        .iter()
        .filter_map(|(name, _)| entries.iter().find(|entry| entry.verb == *name))
    {
        for line in &entry.usage {
            help.push_str(&format!("  {line}\n"));
        }
    }
    help.push_str(concat!(
        "\nOptions:\n",
        "  --format text|json|dot  Select output format (default: text).\n",
        "                         dot is accepted only for inspect.\n",
        "  --version, -V           Show the package version.\n",
        "  --help, -h, help        Show this help without running an operation.\n",
        "                         After a verb, --help or -h shows that verb's usage and options.\n",
    ));
    for entry in commands
        .iter()
        .filter_map(|(name, _)| entries.iter().find(|entry| entry.verb == *name))
    {
        help.push('\n');
        help.push_str(&entry.options_section());
    }
    help.push_str(ROOT_HELP_NOTES);
    help
}

/// The operand and exit-code explanations that close root help.
pub const ROOT_HELP_NOTES: &str = concat!(
    "\nOperands:\n",
    "  <authority> <identity>  Labels that name the source in diagnostics and outcome documents,\n",
    "                         for example `example example:shop/orders`. `check <source-path>`\n",
    "                         labels the source with authority `filament` and the path as identity.\n",
    "  --model <identity>=<file>  Supplies an imported model. <identity> is the string in the\n",
    "                         source's model line, such as `example/shop`; <file> is its source.\n",
    "  <request-file>          A JSON request; its `format` member names the request kind,\n",
    "                         such as native-run/1.\n",
    "\nExit codes:\n",
    "  0   success\n",
    "  10  the result is a violation or undefined\n",
    "  11  a finite observation window did not decide the result\n",
    "  20  refused: the command line, a file or the request was not accepted\n",
    "  21  unsupported: this build does not route that operation\n",
    "  22  incomplete or inconclusive: a limit, timeout or cancellation stopped it,\n",
    "      or it was neither proved nor refuted\n",
    "  30  internal failure\n",
);

/// The candidate nearest to `typed`, when one is close enough to be a likely
/// typo: within two edits, and fewer edits than the typed text has letters.
#[must_use]
pub fn nearest<'a>(typed: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let typed_letters = typed.trim_start_matches('-').chars().count();
    candidates
        .into_iter()
        .map(|candidate| (edit_distance(typed, candidate), candidate))
        .filter(|(distance, _)| *distance <= 2 && *distance < typed_letters)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

/// The Levenshtein distance between two texts, counted in characters.
fn edit_distance(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut row: Vec<usize> = (0..=right.len()).collect();
    for (row_index, left_char) in left.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = row_index + 1;
        for (column, right_char) in right.iter().enumerate() {
            let above = row[column + 1];
            row[column + 1] = if left_char == *right_char {
                diagonal
            } else {
                1 + diagonal.min(above).min(row[column])
            };
            diagonal = above;
        }
    }
    row[right.len()]
}

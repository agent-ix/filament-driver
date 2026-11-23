// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-002-AC-24, AC-25 and AC-26 (TC-049): root and per-verb help are
//! generated from the one option table the verb parsers read, `--version` is
//! a root option only, and `--color` is refused everywhere.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]

use std::process::{Command, Output, Stdio};

use ix_trace_rs::trace;

/// The table the binary reads, included by path so the tests read the same
/// definition and not a copy.
#[path = "../src/options.rs"]
#[allow(dead_code)]
mod options;

use options::{table, VerbEntry};

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const NOT_ROUTED: [&str; 3] = ["prove", "generate", "render"];
const EXTRA_REFUSED: [&str; 4] = ["--version", "-V", "--color", "--bogus"];
const ESCAPE: u8 = 0x1B;

fn run(arguments: &[&str], no_color: bool) -> Output {
    let directory = tempfile::tempdir().expect("empty working directory");
    let mut command = Command::new(FSL);
    command
        .args(arguments)
        .current_dir(directory.path())
        .stdin(Stdio::null());
    if no_color {
        command.env("NO_COLOR", "1");
    } else {
        command.env_remove("NO_COLOR");
    }
    let output = command.output().expect("run fsl");
    assert!(
        !output.stdout.contains(&ESCAPE) && !output.stderr.contains(&ESCAPE),
        "{arguments:?} wrote an escape byte (NO_COLOR={no_color})"
    );
    output
}

fn fsl(arguments: &[&str]) -> Output {
    run(arguments, false)
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// A well-formed value for `option`, as its value form shows.
fn sample(value: &str) -> &'static str {
    match value {
        "<bytes>" => "4096",
        "name=value" => "s1.input_bytes=1024",
        "euclidean|floor|truncating" => "floor",
        "interpreter|aot|jit" => "interpreter",
        "text|json" | "text|json|dot" => "json",
        _ => "operand.json",
    }
}

/// The operands a verb takes, naming files that do not exist, so no run
/// reaches an operation that reads one.
fn operands(entry: &VerbEntry) -> Vec<&'static str> {
    match entry.verb {
        "parse" | "format" => vec!["agent-ix", "unit", "nosuch.fsl"],
        "inspect" => Vec::new(),
        _ => vec!["nosuch.json"],
    }
}

/// Whether a refusal in `output` is the parser refusing `option`.
fn names_refusal(output: &Output, entry: &VerbEntry, option: &str) -> bool {
    let text = text(output);
    text.contains(&format!("unknown {} option `{option}`", entry.verb))
        || text.contains(&format!("{option} requires"))
}

fn probe(entry: &VerbEntry, option: &options::OptionSpec, no_color: bool) -> Output {
    let mut arguments = vec![entry.verb];
    arguments.extend(operands(entry));
    arguments.push(option.name);
    if let Some(value) = option.value {
        arguments.push(sample(value));
    }
    arguments.extend(["--format", "json"]);
    run(&arguments, no_color)
}

/// FR-002-AC-24, TC-049 steps 1, 2 and 6: every table option is on its own
/// line of its verb's section and the built parser accepts it.
#[test]
#[trace("TC-049", "FR-002-AC-24")]
fn help_lists_every_accepted_option() {
    let help = fsl(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(help.stderr.is_empty());
    let help = String::from_utf8(help.stdout).expect("UTF-8 help");
    for section in ["Usage: fsl", "Commands:", "Source operands:", "Options:"] {
        assert!(help.contains(section), "{section}\n{help}");
    }
    for verb in NOT_ROUTED {
        let line = help
            .lines()
            .find(|line| line.trim_start().starts_with(verb))
            .unwrap_or_else(|| panic!("{verb} in Commands:"));
        assert!(line.contains("Recognized;"), "{line}");
    }
    let mut position = help.find("Options:").expect("Options section");
    for entry in table() {
        let section = entry.options_section();
        let at = help[position..]
            .find(&section)
            .unwrap_or_else(|| panic!("{} section after the previous one\n{help}", entry.verb));
        position += at + section.len();
        for option in &entry.options {
            assert!(
                section.lines().any(|line| line == option.help_line()),
                "{} {}",
                entry.verb,
                option.name
            );
            let output = probe(entry, option, false);
            assert!(
                !names_refusal(&output, entry, option.name),
                "{} refused {}: {}",
                entry.verb,
                option.name,
                text(&output)
            );
        }
    }
    for (verb, option) in [
        ("check", options::INTEGER_DIVISION),
        ("check", options::MARKDOWN),
        ("check", options::EXTENSIONS),
        ("parse", options::EXTENSIONS),
        ("run", options::PROVIDER_LIMIT),
        ("run", options::LIMIT),
        ("check", options::LIMIT),
        ("analyze", options::LIMIT),
        ("monitor", options::LIMIT),
        ("replay", options::LIMIT),
        ("inspect", options::HANDOFF),
    ] {
        let entry = options::entry(verb).expect("verb entry");
        assert!(entry.option(option).is_some(), "{verb} {option}");
        if option == options::LIMIT {
            assert!(
                entry.option(option).is_some_and(|spec| spec.repeatable),
                "{verb} {option} is repeatable"
            );
        }
    }
    for verb in ["analyze", "monitor", "replay", "export"] {
        assert!(
            !options::entry(verb).expect("verb entry").options.is_empty(),
            "{verb}"
        );
    }
    let with_color = fsl(&["--help"]);
    assert_eq!(
        run(&["--help"], true).stdout,
        with_color.stdout,
        "NO_COLOR changes nothing"
    );
}

/// FR-002-AC-24, TC-049 step 3: a name listed only in another verb's entry,
/// or `--version`, `-V`, `--color`, `--bogus`, refuses naming it.
#[test]
#[trace("TC-049", "FR-002-AC-24")]
fn parser_refuses_options_outside_its_entry() {
    for entry in table() {
        let mut names: Vec<&str> = table()
            .iter()
            .flat_map(|other| other.options.iter().map(|option| option.name))
            .chain(EXTRA_REFUSED)
            .filter(|name| entry.option(name).is_none())
            .collect();
        names.sort_unstable();
        names.dedup();
        for name in names {
            let output = fsl(&[entry.verb, name]);
            assert_eq!(output.status.code(), Some(20), "{} {name}", entry.verb);
            assert!(
                text(&output).contains(name),
                "{} {name}: {}",
                entry.verb,
                text(&output)
            );
        }
    }
}

/// FR-002-AC-25, TC-049 step 7: `fsl check --help` and every other verb's
/// help, from an empty working directory.
#[test]
#[trace("TC-049", "FR-002-AC-25")]
fn verb_help_writes_its_section() {
    let expected = concat!(
        "Usage: fsl check [options] <source-path>\n",
        "       fsl check [options] <authority> <identity> <source-path>\n",
        "\n",
        "check options:\n",
        "  --markdown\n",
        "  --model <identity>=<file> (repeatable)\n",
        "  --library <identity>=<file> (repeatable)\n",
        "  --dependent-bytes <bytes>\n",
        "  --limit name=value (repeatable)\n",
        "  --integer-division euclidean|floor|truncating\n",
        "  --extensions <file>\n",
        "  --format text|json\n",
    );
    for arguments in [
        &["check", "--help"][..],
        &["check", "-h"],
        &["check", "nosuch.fsl", "--help"],
    ] {
        for no_color in [false, true] {
            let output = run(arguments, no_color);
            assert_eq!(output.status.code(), Some(0), "{arguments:?}");
            assert!(output.stderr.is_empty(), "{arguments:?}");
            assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
        }
    }
    let root = String::from_utf8(fsl(&["--help"]).stdout).expect("UTF-8 help");
    assert!(root.contains(&format!(
        "\n{}",
        &expected[expected.find("check options:").unwrap()..]
    )));
    for entry in table() {
        let output = fsl(&[entry.verb, "--help"]);
        assert_eq!(output.status.code(), Some(0), "{}", entry.verb);
        assert!(output.stderr.is_empty(), "{}", entry.verb);
        let help = String::from_utf8(output.stdout).expect("UTF-8 help");
        assert!(help.starts_with(&format!("Usage: fsl {}", entry.verb)));
        let (usage, section) = help.split_once("\n\n").expect("usage, blank line, section");
        assert!(usage.lines().all(|line| line.contains(entry.verb)));
        assert!(
            root.contains(section),
            "{} section is root help's",
            entry.verb
        );
        assert_eq!(section, entry.options_section());
    }
}

/// FR-002-AC-25, TC-049 steps 8 and 9: help precedence and root-only
/// `--version`.
#[test]
#[trace("TC-049", "FR-002-AC-25")]
fn help_precedence() {
    for verb in NOT_ROUTED {
        let output = fsl(&[verb, "--help"]);
        assert_eq!(output.status.code(), Some(21), "{verb}");
        assert!(text(&output).contains(verb), "{verb}");
        assert!(
            text(&output).contains("operation-not-routed")
                || text(&output).contains("does not route")
        );
    }
    let bad_format = fsl(&["run", "--format", "bogus", "--help"]);
    assert_eq!(bad_format.status.code(), Some(20));
    let message = text(&bad_format);
    assert!(
        message.contains("--format") && message.contains("bogus"),
        "{message}"
    );
    let bad_option = fsl(&["run", "--bogus", "--help"]);
    assert_eq!(bad_option.status.code(), Some(20));
    assert!(text(&bad_option).contains("--bogus"));
    for entry in table() {
        for option in options::VERSION_OPTIONS {
            let output = fsl(&[entry.verb, option, "nosuch.json"]);
            assert_eq!(output.status.code(), Some(20), "{} {option}", entry.verb);
            assert!(text(&output).contains(option), "{} {option}", entry.verb);
            assert!(!text(&output).contains("could not read"), "{}", entry.verb);
        }
    }
}

/// FR-002-AC-26, TC-049 steps 10 and 11: `--color` is refused on every verb
/// before any named file is read, no help section lists it, and no stream
/// carries an escape byte under either `NO_COLOR` setting.
#[test]
#[trace("TC-049", "FR-002-AC-26")]
fn color_is_refused_on_every_verb() {
    assert!(!String::from_utf8_lossy(&fsl(&["--help"]).stdout).contains("--color"));
    for no_color in [false, true] {
        for entry in table() {
            assert!(
                !entry.help().contains("--color"),
                "{} help lists --color",
                entry.verb
            );
            for value in ["always", "never"] {
                let mut arguments = vec![entry.verb];
                arguments.extend(operands(entry));
                arguments.extend(["--color", value, "--format", "json"]);
                let output = run(&arguments, no_color);
                assert_eq!(output.status.code(), Some(20), "{arguments:?}");
                let message = text(&output);
                assert!(message.contains("--color"), "{arguments:?}: {message}");
                assert!(
                    !message.contains("could not read"),
                    "{arguments:?}: {message}"
                );
            }
        }
    }
}

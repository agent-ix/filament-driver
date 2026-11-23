// SPDX-License-Identifier: AGPL-3.0-or-later
//! The exact text of the CLI's help and refusal messages, checked through the
//! real binary.

use fsl_cli_contracts::{Captured, CliCase};

const VERBS: &str =
    "parse, format, check, compile, run, export, analyze, monitor, prove, replay, generate, inspect, render";

fn stderr_only(output: &Captured, expected: &str) {
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    assert!(output.0.stdout.is_empty(), "{output:?}");
    assert_eq!(output.stderr_text().expect("text stderr"), expected);
}

fn message(output: &Captured) -> String {
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    let value = output.json().expect("one refusal document");
    let diagnostics = value["diagnostics"].as_array().expect("diagnostics");
    assert_eq!(diagnostics.len(), 1, "{value}");
    assert_eq!(diagnostics[0]["code"], "invalid-request", "{value}");
    diagnostics[0]["message"]
        .as_str()
        .expect("message")
        .to_owned()
}

/// Trace: FR-002-AC-31
/// Provenance: QSL-2932
#[test]
fn an_unknown_verb_lists_the_verbs_and_a_misspelling_names_the_nearest(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    stderr_only(
        &case.run(&["bogus"])?,
        &format!("unknown verb `bogus`; verbs are {VERBS}; run `fsl --help` for usage\n"),
    );
    stderr_only(
        &case.run(&["chek", "--format", "json"])?,
        &format!(
            "unknown verb `chek`; did you mean `check`; verbs are {VERBS}; run `fsl --help` for usage\n"
        ),
    );
    Ok(())
}

/// Trace: FR-002-AC-31
/// Provenance: QSL-2932
#[test]
fn an_unknown_option_names_the_nearest_option_and_lists_the_verbs_options(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    assert_eq!(
        message(&case.run(&["check", "--modle", "x", "a.fsl", "--format", "json"])?),
        "unknown check option `--modle`; did you mean `--model`; check accepts --markdown, --model, --library, --dependent-bytes, --limit, --integer-division, --extensions, --format"
    );
    assert_eq!(
        message(&case.run(&["run", "--engin", "jit", "r.json", "--format", "json"])?),
        "unknown run option `--engin`; did you mean `--engine`; run accepts --request-bytes, --dependent-bytes, --limit, --engine, --provider-config, --provider-limit, --clocks, --format"
    );
    assert_eq!(
        message(&case.run(&["compile", "--bogus", "r.json", "--format", "json"])?),
        "unknown compile option `--bogus`; compile accepts --clocks, --output, --limit, --format"
    );
    Ok(())
}

/// Trace: FR-002-AC-31
/// Provenance: QSL-2932
#[test]
fn an_unknown_engine_lists_the_engines() -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    assert_eq!(
        message(&case.run(&["run", "--engine", "fast", "r.json", "--format", "json"])?),
        "unknown engine `fast`; engines are interpreter, aot, jit"
    );
    Ok(())
}

/// Trace: FR-002-AC-31
/// Provenance: QSL-2932
#[test]
fn an_unknown_limit_setting_prints_one_stage_grouped_list() -> Result<(), Box<dyn std::error::Error>>
{
    let case = CliCase::from_environment()?;
    case.write("a.fsl", b"x")?;
    let text = message(&case.run(&["check", "--limit", "foo=1", "a.fsl", "--format", "json"])?);
    assert_eq!(
        text,
        "`--limit foo=1` names no setting; a setting is <stage>.<name>, and the settings are s1 (input_bytes, tokens, nodes, work_units); model (declaration_records, derivation_facts, effective_declarations, dispatch_candidates, work_units, ancestor_steps, family_steps, sequence_maximum); intake (input_bytes); s3 (nodes, input_bytes, work_units, rational_bits, decimal_scale, decimal_literal_scale, decimal_literal_scale_total); dependency (libraries, import_edges, source_bytes); environment (ancestor_steps, work_units); identity (input_bytes); i2 (input_bytes, nodes, edges, occurrences, diagnostics, work_units, native_join_storage); progress (covered_positions); emit (output_bytes)"
    );
    Ok(())
}

/// Trace: FR-002-AC-32
/// Provenance: QSL-2931
#[test]
fn help_with_a_verb_prints_that_verbs_help() -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    for verb in [
        "parse", "format", "check", "compile", "run", "export", "analyze", "monitor", "replay",
        "inspect",
    ] {
        let flagged = case.run(&[verb, "--help"])?;
        let worded = case.run(&["help", verb])?;
        assert_eq!(worded.0.status.code(), Some(0), "{worded:?}");
        assert!(worded.0.stderr.is_empty(), "{worded:?}");
        assert_eq!(worded.0.stdout, flagged.0.stdout, "{verb}");
        assert!(worded
            .stdout_text()?
            .starts_with(&format!("Usage: fsl {verb} ")));
    }
    stderr_only(
        &case.run(&["help", "bogus"])?,
        &format!("unknown verb `bogus`; verbs are {VERBS}; run `fsl --help` for usage\n"),
    );
    Ok(())
}

/// Trace: FR-002-AC-32
/// Provenance: QSL-2931
#[test]
fn root_help_describes_each_verb_the_operands_and_the_exit_codes(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    let output = case.run::<&str>(&["--help"])?;
    assert_eq!(output.0.status.code(), Some(0), "{output:?}");
    let text = output.stdout_text()?;
    for line in [
        "  parse      Parses a source file and reports its outcome; needs <authority> <identity> <source-path>. Output is text by default; --format json selects JSON.\n",
        "  format     Writes a source file reformatted; needs <authority> <identity> <source-path>.\n",
        "  check      Checks a source file and reports its diagnostics; needs <source-path>, and --model for each import. Output is text by default; --format json selects JSON.\n",
        "  compile    Checks a native-compile/1 request file and builds its package; needs <request-file>, --output writes the package. Output is text by default; --format json selects JSON.\n",
        "  run        Runs a native-run/1 request file; needs <request-file>. Output is text by default; --format json selects JSON.\n",
    ] {
        assert!(text.contains(line), "missing {line:?} in {text}");
    }
    assert!(
        text.ends_with(
            "\nOperands:
  <authority> <identity>  Labels that name the source in diagnostics and outcome documents,
                         for example `example example:shop/orders`. `check <source-path>`
                         labels the source with authority `filament` and the path as identity.
  --model <identity>=<file>  Supplies an imported model. <identity> is the string in the
                         source's model line, such as `example/shop`; <file> is its source.
  <request-file>          A JSON request; its `format` member names the request kind,
                         such as native-run/1.

Exit codes:
  0   success
  10  the result is a violation or undefined
  11  a finite observation window did not decide the result
  20  refused: the command line, a file or the request was not accepted
  21  unsupported: this build does not route that operation
  22  incomplete or inconclusive: a limit, timeout or cancellation stopped it,
      or it was neither proved nor refuted
  30  internal failure
"
        ),
        "{text}"
    );
    Ok(())
}

/// Trace: FR-002-AC-30
/// Provenance: QSL-2929
#[cfg(unix)]
#[test]
fn a_non_utf8_argument_after_a_verb_is_refused_on_stdout_a_non_utf8_verb_on_stderr(
) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let case = CliCase::from_environment()?;
    let bad = OsString::from_vec(vec![0xff]);

    let output = case.run(&[OsString::from("run"), bad.clone()])?;
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    assert_eq!(
        output.stdout_text()?,
        "run: refusal (exit 20)\n<no-source>: io-error: cannot access \u{fffd}: No such file or directory (os error 2)\n  hint: Check that the path exists and that the process can read or write it.\n"
    );

    stderr_only(&case.run(&[bad])?, "command must be valid UTF-8\n");
    Ok(())
}

/// Trace: FR-002-AC-23
/// Provenance: QSL-2929
#[test]
fn a_bare_inspect_is_not_routed_in_text_and_dot_and_dot_with_handoff_is_refused(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    for args in [
        vec!["inspect"],
        vec!["inspect", "--format", "text"],
        vec!["inspect", "--format", "dot"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(21), "{args:?} {output:?}");
        assert!(output.0.stderr.is_empty(), "{output:?}");
        assert_eq!(
            output.stdout_text()?,
            "inspect: unsupported (exit 21)\n<no-source>: unsupported_operation: fsl inspect: this build does not route the inspect operation (unsupported_operation)\n  cause: operation-not-routed\n",
            "{args:?}"
        );
    }
    let output = case.run(&["inspect", "--handoff", "missing", "--format", "dot"])?;
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    assert!(output.stdout_text()?.contains("--format"), "{output:?}");
    Ok(())
}

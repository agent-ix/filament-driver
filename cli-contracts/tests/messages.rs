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

const SPINE_SETTINGS: &str = "s1 (input_bytes, tokens, nodes, work_units); model (declaration_records, derivation_facts, effective_declarations, dispatch_candidates, work_units, ancestor_steps, family_steps, sequence_maximum); intake (input_bytes); s3 (nodes, input_bytes, work_units, rational_bits, decimal_scale, decimal_literal_scale, decimal_literal_scale_total); dependency (libraries, import_edges, source_bytes); environment (ancestor_steps, work_units); identity (input_bytes); i2 (input_bytes, nodes, edges, occurrences, diagnostics, work_units, native_join_storage); progress (covered_positions); emit (output_bytes)";
const RUN_EXTRA_SETTINGS: &str = "; observation (input_bytes, objects, values); accounting (integer_bits, decimal_digits, scale_expansion, text_input_bytes, text_scalars, normalized_scalars, unit_edges, value_occurrences, work_units, result_units)";
const EXPORT_EXTRA_SETTINGS: &str = "; export (output_bytes, nodes, work_units)";

fn unknown_setting(listing: &str) -> String {
    format!(
        "`--limit foo=1` names no setting; a setting is <stage>.<name>, and the settings are {listing}"
    )
}

/// Trace: FR-002-AC-31
/// Provenance: QSL-2932
#[test]
fn an_unknown_limit_setting_lists_only_what_the_verb_accepts(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    case.write("a.fsl", b"x")?;
    case.write("r.json", b"{}")?;
    let json = |verb: &str, operands: &[&str]| {
        let mut args = vec![verb];
        args.extend_from_slice(operands);
        args.extend(["--limit", "foo=1", "--format", "json"]);
        case.run(&args)
    };
    assert_eq!(
        message(&json("check", &["a.fsl"])?),
        unknown_setting(SPINE_SETTINGS)
    );
    assert_eq!(
        message(&json("run", &["r.json"])?),
        unknown_setting(&format!("{SPINE_SETTINGS}{RUN_EXTRA_SETTINGS}"))
    );
    assert_eq!(
        message(&json("export", &["r.json"])?),
        unknown_setting(&format!("{SPINE_SETTINGS}{EXPORT_EXTRA_SETTINGS}"))
    );
    assert_eq!(
        message(&json("analyze", &["r.json"])?),
        unknown_setting("replay (input_bytes)")
    );
    for verb in ["monitor", "replay"] {
        assert_eq!(
            message(&json(verb, &["r.json"])?),
            format!("`--limit foo=1` names no setting, and {verb} accepts no --limit settings")
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-31
/// Provenance: QSL-2932
#[test]
fn a_setting_the_verb_cannot_apply_is_named_with_the_settings_it_accepts(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    case.write("r.json", b"{}")?;
    assert_eq!(
        message(&case.run(&[
            "compile",
            "r.json",
            "--limit",
            "admission.work_units=5",
            "--format",
            "json"
        ])?),
        format!(
            "limit setting `admission.work_units` is unavailable for the `compile` operation; settings this command accepts, by stage: {SPINE_SETTINGS}"
        )
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
  --model <identity>=<file>
                         Supplies an imported model. <identity> is the string in the
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

const NOT_AVAILABLE: &str =
    "is not available in this build; running it exits 21 (unsupported_operation).\n\n";

/// Trace: FR-002-AC-25
/// Provenance: QSL-2931
#[test]
fn help_for_a_verb_this_build_does_not_route_says_so_first_and_exits_zero(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    for (verb, usage) in [
        ("prove", "prove [options] <request-file>"),
        ("generate", "generate [options]"),
        ("render", "render [options]"),
    ] {
        let expected = format!(
            "fsl {verb} {NOT_AVAILABLE}Usage: fsl {usage}\n\n{verb} options:\n  --format text|json\n"
        );
        for args in [vec![verb, "--help"], vec![verb, "-h"], vec!["help", verb]] {
            let output = case.run(&args)?;
            assert_eq!(output.0.status.code(), Some(0), "{args:?} {output:?}");
            assert!(output.0.stderr.is_empty(), "{args:?} {output:?}");
            assert_eq!(output.stdout_text()?, expected, "{args:?}");
        }
        let run = case.run(&[verb])?;
        assert_eq!(run.0.status.code(), Some(21), "{run:?}");
        assert_eq!(
            run.stdout_text()?,
            format!(
                "{verb}: unsupported (exit 21)\n<no-source>: unsupported_operation: fsl {verb}: this build does not route the {verb} operation (unsupported_operation)\n  cause: operation-not-routed\n"
            )
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-32
/// Provenance: QSL-2931
#[test]
fn help_with_a_non_verb_or_extra_words_refuses_on_stderr() -> Result<(), Box<dyn std::error::Error>>
{
    let case = CliCase::from_environment()?;
    stderr_only(
        &case.run(&["help", "--format", "json"])?,
        &format!("unknown verb `--format`; verbs are {VERBS}; run `fsl --help` for usage\n"),
    );
    stderr_only(
        &case.run(&["help", "run", "extra"])?,
        "help takes at most one verb\n",
    );
    Ok(())
}

/// Trace: FR-002-AC-23
/// Provenance: QSL-2933
#[test]
fn a_bare_inspect_is_not_routed_names_the_handoff_form_and_dot_with_handoff_is_refused(
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
            "inspect: unsupported (exit 21)\n<no-source>: unsupported_operation: fsl inspect: this build does not route the inspect operation (unsupported_operation); use `fsl inspect --handoff <directory>`\n  cause: operation-not-routed\n",
            "{args:?}"
        );
    }
    let output = case.run(&["inspect", "--handoff", "missing", "--format", "dot"])?;
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    assert_eq!(
        output.stdout_text()?,
        "inspect: refusal (exit 20)\n<no-source>: invalid-request: --format `dot` is not available with --handoff\n"
    );
    Ok(())
}

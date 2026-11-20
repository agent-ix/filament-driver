// SPDX-License-Identifier: AGPL-3.0-or-later
//! Text outcome writes retain their outcome status when the stdout reader closes.

#[cfg(unix)]
mod unix {
    use std::fs::File;
    use std::os::fd::OwnedFd;

    use fsl_cli_contracts::CliCase;
    use serde_json::json;

    /// Trace: FR-287-AC-8
    /// Provenance: QSL-2098
    #[test]
    fn closed_stdout_preserves_success_and_refusal_for_text_outcomes(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let case = CliCase::from_environment()?;
        case.write(
            "value.fsl",
            br#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function Double using v(n: Integer): Integer pure { n + n }
"#,
        )?;
        // An unfinished function body must refuse during syntax admission.
        case.write(
            "unfinished.fsl",
            br#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function Double using v(n: Integer): Integer pure {
"#,
        )?;
        for (file, source) in [("valid", "value.fsl"), ("refused", "unfinished.fsl")] {
            let program = json!({"source":{"file":source,"authority":"stream-contracts",
                "identity":"stream:double","document":"Double","formal_revision":1}});
            for (verb, wire) in [("compile", "native-compile/1"), ("run", "native-run/1")] {
                let mut request = json!({"format":wire,"request":{"models":[],"program":program}});
                if verb == "run" {
                    request["request"]["call"] = json!({"function":"Double","arguments":[
                        {"parameter":"n","value":{"integer":"6"}}]});
                }
                case.write(
                    format!("{verb}-{file}.json"),
                    &serde_json::to_vec(&request)?,
                )?;
            }
        }
        for (arguments, operation, category, exit) in [
            (
                vec!["parse", "stream-contracts", "stream:double", "value.fsl"],
                "parse",
                "success",
                0,
            ),
            (
                vec![
                    "parse",
                    "stream-contracts",
                    "stream:double",
                    "unfinished.fsl",
                ],
                "parse",
                "refusal",
                20,
            ),
            (
                vec!["compile", "compile-valid.json"],
                "package",
                "success",
                0,
            ),
            (
                vec!["compile", "compile-refused.json"],
                "package",
                "refusal",
                20,
            ),
            (vec!["run", "run-valid.json"], "execute", "success", 0),
            (vec!["run", "run-refused.json"], "execute", "refusal", 20),
        ] {
            let mut arguments = arguments;
            arguments.extend(["--format", "text"]);
            // Establish that this exact invocation produces a rendered outcome,
            // so the closed-reader call necessarily exercises an attempted write.
            let rendered = case.run(&arguments)?;
            assert_eq!(rendered.0.status.code(), Some(exit), "{rendered:?}");
            assert!(rendered.0.stderr.is_empty(), "{rendered:?}");
            assert!(
                rendered
                    .stdout_text()?
                    .starts_with(&format!("{operation}: {category} (exit {exit}) at ")),
                "{rendered:?}"
            );
            if category == "refusal" {
                assert!(
                    rendered.stdout_text()?.contains("invalid_syntax"),
                    "{rendered:?}"
                );
            }
            let (reader, writer) = std::io::pipe()?;
            drop(reader);
            let closed = case.run_stdout_to_file(&arguments, File::from(OwnedFd::from(writer)))?;
            assert_eq!(
                closed.0.status.code(),
                Some(exit),
                "{arguments:?}: {closed:?}"
            );
            assert!(closed.0.stderr.is_empty(), "{arguments:?}: {closed:?}");
        }
        Ok(())
    }
}

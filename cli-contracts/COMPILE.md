---
type: reference
---
# Compile contracts

The compile cases run an explicitly selected, prebuilt FSL through `CliCase`.
They create original sources and requests in private directories, then check
outcomes, written packages and side effects.

Run the compile group from the repository root:

```sh
FSL=/absolute/path/to/fsl CARGO_TARGET_DIR="$PWD/target" \
  cargo test --locked --manifest-path cli-contracts/Cargo.toml \
  --test compile compile_cases:: -- --test-threads=1
```

The cases cover single and multiple source units, library selection, clocks,
request and CLI output paths, handoff verification, text and JSON, argument
errors, source admission, closed request records and input/output limits. Package
assertions follow declaration and body references to verify that `Total` adds
Integer 2 to its `pieces` parameter. The public handoff reader independently
verifies the emitted package.

File input is tested from another working directory, at a non-UTF-8 request
filename on Unix, and through an explicit `/dev/stdin` operand. Stdin cannot
replace an omitted request-file argument or a named file.

The unknown-language diagnostic case is ignored under
[QSL-2658](https://linear.app/agent-ix/issue/QSL-2658): its desired
`unknown_language` oracle is preserved. It is excluded from active coverage.
Full native authoring (QSL-1980), source-model export (QSL-1979), and planned
Markdown extraction compilation retain their existing completion ownership and
are outside this runnable group.

One intake-option question remains outside the assertions: FSL FR-287-AC-4 and
FR-869 require compile intake overrides, while FR-027-AC-3 and driver FR-023-AC-7
say compile has fixed ceilings; the per-verb help lists no intake override flags.
The tests check the default request and dependent ceilings without claiming the
missing overrides or their diagnostic hints are complete.

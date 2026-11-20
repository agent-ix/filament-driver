---
type: reference
---
# Monitor contracts

The monitor cases exercise an explicitly selected public FSL binary with
original source programs, request files and JSON Lines observations. They check
ordered status changes, final summaries, trace semantics and process behavior.

Run the monitor group from the repository root with the matching authoritative
transport helper:

```sh
FSL=/absolute/path/to/fsl \
FSL_TEST_PROVIDER=/absolute/path/to/fsl-test-provider \
CARGO_TARGET_DIR="$PWD/target" \
  cargo test --locked --manifest-path cli-contracts/Cargo.toml \
  --test monitor monitor_cases:: -- --test-threads=1
```

The active cases cover finite, fixed-sample progressed and timestamped streams;
complete and incomplete progress; real success and violation; pending finite
prefixes; empty traces; malformed records; text and canonical JSON Lines;
request, source and filesystem refusals; argument forms; and provider options.
They distinguish physical record indices, observation positions, formula
origins and settlement horizons. A later safe observation cannot reverse a
settled violation.

Staged stdin holds the stream open until an actual deciding event appears.
The process must flush that event before receiving later observations or EOF.
The same bounded capture lifecycle checks interruption while stdin is open:
SIGTERM and SIGINT must each produce a final incomplete outcome and exit 22.

The transport helper supplies only provider-protocol responses. Its reported
proof is explicitly trusted, without a core certificate. Temporal verdicts come
from FSL's evaluation of the original observations. Provider routing checks the
actual item and deadline received by the helper; clauses alone send no item.

Five desired cases remain ignored, outside active coverage:

- QSL-2477: finite summaries must omit inapplicable optional members.
- QSL-2671: an explicit blank record must produce a refusal event.
- QSL-2673: resumed events must preserve global observation positions.
- QSL-1848: two timed-trace cases retain their intended witness and
  completeness verdicts while public timed execution is unavailable.

The machine checks validate complete records and their semantic fields. Full
published-schema validation remains under QSL-2477. The cases do not claim
byte equality against an in-process producer, which requires a different entry
point from this external-binary component.

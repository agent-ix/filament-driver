---
type: status
---
# Monitor contract validation

The final component gate passed on reviewed code with its accepted review
artifacts at `33b23292abbcbc0f40dc5352a6010914f5e29bc5`. The trailing validation
commit records these results without changing executable code.

## Inputs and command

The fixed L3 binary cohort is driver
`a260b3b26277bfa907ce77373c4f15dac6671083`, FSL
`8877b38b6397055fa11dc2e7882ad87d02ecb654`, codegen
`a11a66ab9c5eeee9163f5d4d084644296b565ba9`, and IR
`c7b62038ed95262e3e51477f0831978397c8c5f7`. No product build was run.

The command ran once from this ticket's worktree:

```sh
FSL=/home/peter/dev/.worktrees/luna-fsl-l3-main/target/debug/fsl \
FSL_TEST_PROVIDER=/home/peter/dev/.worktrees/luna-fsl-l3-main/target/debug/fsl-test-provider \
CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 make cli-contracts
```

The Make target selected the existing worktree `target` directory and one test
thread. Disk free before the gate was 215 GiB. Local log:
`/home/peter/dev/.agent/qsl-2473-final-component.log`. Actual process exit: **0**.

## Results

The table reports the files present on this branch, including accepted incoming
check, compile and run cases. It makes no claim about disjoint newer PRs.

| Target | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Lifecycle helpers | 5 | 0 | 0 |
| Check | 32 | 0 | 1 |
| Compile | 39 | 0 | 1 |
| Native contracts | 5 | 0 | 0 |
| Monitor | 28 | 0 | 6 |
| Run | 23 | 0 | 3 |
| Total | 132 | 0 | 11 |

The monitor exclusions preserve required oracles under QSL-2477, QSL-2671,
QSL-2673, QSL-2674 and two cases under QSL-1848. Ignored cases are not coverage.
The other five exclusions retain their existing check, compile and run owners.
QSL-2473 remains partial. The diagnostic naming ambiguity is recorded in
[Monitor contracts](MONITOR.md).

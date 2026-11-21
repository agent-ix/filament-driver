---
id: SR-202611210022442313144108509
title: "QSL-2378 driver process limits code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; full PR 221 diff including Unix launcher, host, adapter, Rust fixtures, installation and release packaging"
review_set: subset
relationships:
  - {target: "ix://agent-ix/filament-driver/FR-006", type: references}
  - {target: "ix://agent-ix/filament-driver/FR-013", type: references}
---

## Summary

Independent static Rust review of driver PR 221, including its working-tree trace corrections. The reviewer did not author the driver changes. The safe resource launcher and its forwarding path are present, but partial-frame process deaths are lost, resource fixtures duplicate behavior, and test evidence needs correction.

## Verdict

FAIL: consolidate the duplicated fixture workload and fix the failure and evidence issues below. No runtime pass is claimed by this reviewer. Focused execution is assigned separately; full gates were explicitly excluded.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The memory allocation workload and CPU spin are implemented twice in the new host fixture and existing CLI fixture. Searching both fixture trees found the same 256 MiB reservation, exit 71 and five-second spin; changes to this behavior require editing both owners. Consolidate the shared fixture behavior. | filament-plugin-host/tests/fixtures/resource_provider.rs:28; fsl-test-provider/src/main.rs:102 |
| FND-002 | medium | A resource death after a partial result frame becomes TruncatedLength or TruncatedFrame, then PluginError::Frame. The adapter accepts only Exited and returns Unknown, so the item is inconclusive rather than failed. Preserve the observed unsuccessful exit when EOF truncates a frame and test that Rust provider form. | filament-plugin-host/src/lib.rs:812; fsl-cli/src/analyze_provider.rs:110 |
| FND-003 | medium | Both CPU fixtures stop after five wall-clock seconds, whereas the OS enforces one CPU second. A descheduled provider can finish its loop before consuming its CPU limit and make a correct implementation fail the breach test. Drive the workload until the OS stops it and bound the test with an independent watchdog. | filament-plugin-host/tests/fixtures/resource_provider.rs:38; fsl-test-provider/src/main.rs:110 |
| FND-004 | medium | The five new tests use comment trace IDs, but this repository's engine accepts only rust-trace-attribute. Coverage reports seven unmatched mentions with trace-id-outside-marker; use the declared trace attribute on the tests. | filament-plugin-host/tests/resource_limits.rs:37; fsl-cli/tests/frontend_cli.rs:1532 |
| FND-005 | low | An installation directory literally named deps violates the documented sibling-launcher contract: /opt/deps/fsl with /opt/deps/fsl-plugin-launch instead searches /opt/fsl-plugin-launch. Prefer an actual sibling before any Cargo-test fallback, or pass the launcher path explicitly. | filament-plugin-host/src/lib.rs:46; README.md:43 |
| FND-006 | medium | The new enforcement-status test compares two values from the same getter but never asserts the platform's expected enum variant. Returning NotEnforcedOnPlatform from the public getter on Linux can pass while violating FR-013-AC-16. Assert Enforced on Linux and NotEnforcedOnPlatform where appropriate. | filament-plugin-host/tests/resource_limits.rs:90 |
| FND-007 | medium | A truncated frame during cancellation passes the full original grace to frame_or_exit_error after part of the grace has elapsed. That helper starts a fresh process-exit wait, so late EOF can extend cancellation past the configured grace. Bound it by the remaining grace. | filament-plugin-host/src/lib.rs:899 |
| FND-008 | medium | The request writer returns BrokenPipe as PluginError::Frame immediately. A resource-killed provider that closes stdin while the host is writing can therefore still settle Unknown instead of failed, because the adapter maps only an observed Exited status. Preserve the unsuccessful exit on this write path and cover it. | filament-plugin-host/src/lib.rs:923; fsl-cli/src/analyze_provider.rs:110 |

## Coverage

Reviewed all 15 changed files: release workflow, lockfile, README, host manifest, launch.rs, lib.rs, resource fixture and tests, analyze_provider.rs, cli.rs, process_provider.rs, signals.rs, frontend_cli.rs, fsl-test-provider/main.rs and driver process_provider.rs. Read repository AGENTS.md, CLAUDE.md, contributor rules, Rust idioms and lints, plus FR-006 and FR-013. No applicable AssuranceProfile was found.

The Unix launcher applies safe rlimit calls before exec, retains stricter inherited hard limits and disables core dumps. Command arguments remain separate argv entries. Windows keeps the existing direct provider path. Forwarding preserves the caller's memory and CPU values through both public limit carriers. Observed signal/nonzero failures use FSL's existing ToolFailure vocabulary through the proposed ProcessFailed API rather than inventing limit measurements. The release archive and source installation now include the launcher. No new unsafe operation, shell transport or weakened gate was found.

The parent reports the host all-target check passed and has requested focused host tests separately. This reviewer ran no Cargo command, following shared-lane admission instructions. Source-level checks do not establish runtime success. The normal consuming FSL pin remains pending, so the adapter cannot yet be claimed compiled against its new API.

The earlier incorrect FR-287 criterion mentions were removed in the working tree before this finding set was frozen. FND-004 records the separate marker-form issue demonstrated by the actual trace engine. Existing repository trace debt is reported in the separate gap-analysis artifact, not attributed to this PR.

## Reviewed units

```yaml
scope:
  - {id: resource-launcher, path: filament-plugin-host/src/launch.rs, role: examined, excerpt: "resource.set(hard, hard)?;"}
  - {id: launcher-discovery, path: filament-plugin-host/src/lib.rs, role: examined, excerpt: "directory.file_name() == Some(std::ffi::OsStr::new(\"deps\"))"}
  - {id: request-failure-mapping, path: fsl-cli/src/analyze_provider.rs, role: examined, excerpt: "PluginError::Exited(status) = error.primary_cause()"}
  - {id: limit-forwarding, path: fsl-cli/src/process_provider.rs, role: examined, excerpt: "max_memory_bytes: limits.max_memory_bytes"}
  - {id: host-fixture, path: filament-plugin-host/tests/fixtures/resource_provider.rs, role: examined, excerpt: "allocation.try_reserve_exact(256 * 1024 * 1024)"}
  - {id: cli-fixture, path: fsl-test-provider/src/main.rs, role: examined, excerpt: "allocation.try_reserve_exact(256 * 1024 * 1024)"}
  - {id: release-packaging, path: .github/workflows/release.yml, role: examined, excerpt: "tar -C \"target/${CARGO_PROFILE}\" -czf \"release/$NAME\" \"$BINARY\" fsl-plugin-launch"}
```

## Dispositions — round 1

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Both fixture binaries call the one ResourceWorkload implementation in fsl-test-provider/src/lib.rs; the host uses a default-off test-support dependency. |
| FND-002 | fixed | Truncated response EOF now waits for and preserves an observed unsuccessful process status; a partial-memory-response Rust fixture and host/CLI tests exercise that path. The distinct request-write path is recorded as FND-008. |
| FND-003 | fixed | The single CPU workload runs until OS termination; host and CLI tests use independent watchdogs rather than a five-second workload cutoff. |
| FND-004 | deferred: fix round 2 | Bare trace attributes on CLI integration tests bind, but the fully qualified ix_trace_rs::trace attributes on host and CLI unit tests still do not bind in the active engine. The fresh matrix leaves FR-013-AC-16 and AC-17 untagged. Import trace and use the declared bare attribute. |
| FND-005 | fixed | Launcher discovery first selects an existing sibling, including within a directory named deps, before using the Cargo test-layout fallback. |

## Additional review — c804a1f

Inspected the frozen platform-policy extension in the same review cycle. Linux retains pre-exec AS/DATA limits; Unix retains CPU enforcement. Non-Linux explicit memory requests return a typed UnsupportedPlatform before spawning. The explicit-presence bit is retained even for a numeric default and forwarded through run, requirement-item run, analyze, replay and monitor. Default macOS sessions expose the limitation through diagnostics and a separate CLI stderr notice, rather than changing FSL's document or silently claiming enforcement. This follows the architecture ruling supplied in the review brief.

The author reports the host lib/bins/resource-test check passed. Runtime runs and the CLI's normal consuming dependency pin are pending; no new execution pass is claimed by this reviewer, who ran no Cargo command. The newly added tests now have assertions on real process behavior, but FND-004 and FND-006 still limit their evidence. FND-007 and FND-008 are additional concrete error-path issues found while inspecting the fix, not a second general review pass.

Current code-review disposition is CONDITIONAL pending FND-004 and FND-006 through FND-008. The original high duplication finding is fixed. Separate base and integrity artifacts review the new FR-013 scope; neither artifact expands this PR into unrelated baseline work.

## Dispositions — round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | Host and CLI unit tests import ix_trace_rs::trace and use bare trace attributes. The fresh engine binds FR-013-AC-16, AC-17 and AC-18 and reports no unmatched resource-test tags. |
| FND-006 | fixed | The session result is explicitly compared with Enforced on Linux and NotEnforcedOnPlatform on other platforms before completed-session diagnostics are compared with it. |
| FND-007 | fixed | The partial-EOF wait during cancellation receives until.saturating_duration_since(Instant::now()), retaining the original grace endpoint. |
| FND-008 | fixed | BrokenPipe errors enter the same bounded status-observation helper as truncated EOF, with the caller's cancellation handle. The Rust CPU-before-request fixture and a one-MiB request exercise process death during a blocked write. |

## Final review state

PASS for the bounded code review after inspecting all consolidated fixes at 83cfddd46adedac5ad57f0851820d0831f0380e9. No additional functional finding was found in this final disposition pass. Every original PR code finding now has a fixed disposition.

The parent reports the focused host lib/bins/resource-test check passed and the previous three macOS host cases passed at c804a1f. The five-case Linux runtime run and updated four-case macOS run are still assigned separately; no result is presumed here. CLI consuming-crate compilation remains held for its normal coordinated dependency update. This source review does not claim those pending checks passed. The reviewer ran no Cargo or full gate.

## Final reviewed units

```yaml
scope:
  - id: enforcement-status-oracle
    path: filament-plugin-host/tests/resource_limits.rs
    role: examined
    excerpt: |-
      let enforcement = session.memory_enforcement();
          #[cfg(target_os = "linux")]
          assert_eq!(
              enforcement,
              filament_plugin_host::MemoryEnforcement::Enforced
          );
          #[cfg(not(target_os = "linux"))]
          assert_eq!(
              enforcement,
              filament_plugin_host::MemoryEnforcement::NotEnforcedOnPlatform
          );
          assert_eq!(
              session
                  .shutdown()
                  .expect("orderly shutdown")
                  .memory_enforcement,
              enforcement
          );
      }

      #[trace("FR-013-AC-4", "FR-
  - id: cancellation-remaining-grace
    path: filament-plugin-host/src/lib.rs
    role: examined
    excerpt: |-
      until.saturating_duration_since(Instant::now()),
                          None,
  - id: request-write-exit-observation
    path: filament-plugin-host/src/lib.rs
    role: examined
    excerpt: |-
      frame_or_exit_error(child, error, exit_grace, Some(cancel))?)
                  }
                  Err(RecvTime
```

## Tracker record

Ticket: QSL-2378. Linear has-review was already present. All posted comments passed read-back: the returned bodies matched, their YAML parsed with the installed yaml parser, finding IDs matched the retained artifact, and every fixed disposition retained a nonempty source excerpt.

- review: c73c250e-b3c0-40b7-987d-ecb3a0cb6ffb
- dispositions round 1: 87abca96-b0d8-447f-9e47-d404e13f41e5
- dispositions round 2: e189cf64-ce53-407e-bc73-72a9f16cb125


## Integration custody — 7eac980

Narrow integration disposition at driver `7eac980458ba93e0566b6dcf24793d3da3b3e2f0`, rebased from reviewed `e94b181e148a7edb8b1f08f6aab1b37d46af023f` onto the normal driver cohort `6b8f2a7`. This appends custody to the existing review cycle; it is not a fresh review, matrix run or baseline measurement.

The range-diff preserves the original launcher, typed process failures, dependency, shared Rust fixtures, partial-frame exit handling and request-write/remaining-grace fixes. The platform-policy commit changes only the placement of the unchanged memory-enforcement accessor into the existing implementation alongside hybrid exchange. Both methods remain present. The sole added integration commit adds `UnsupportedPlatform` to the hybrid transport error match. Explicit unsupported memory requests are still refused by `PluginSession::start` before process launch, so that defensive match arm does not turn an admitted platform refusal into an active exchange outcome. The normal manifest and lockfile agree on FSL `f5977125981288ea1bd36fb67f71780182055aac`; the adapter retains the observed Crash/NonzeroExit `ProcessFailed` mapping without inventing a resource measurement.

The author reports a passing focused locked CLI/frontend compilation at this head. This reviewer ran no Cargo or runtime tests. The earlier five Linux and four macOS host results remain evidence for their earlier source heads only. Current-head qualification remains conditional on the queued three CLI and two unit tests; no current-head runtime pass is claimed. No Linear operations were performed.

Static integration disposition: CLEAR, no new finding. Original FND-001 through FND-008 remain fixed: shared real Rust workloads, observed process-status precedence, external watchdog, bare imported trace attributes, sibling-first launcher discovery, explicit platform enum oracle, remaining grace and write-side status recovery are retained. `git diff --check 6b8f2a7..7eac980` exited 0.

Structural validation of these four retained artifacts with the explicit driver repository scope exited 0 with no diagnostics.

## Integration runtime disposition — 7eac980

Parent-reported frozen-head qualification at `7eac980458ba93e0566b6dcf24793d3da3b3e2f0`: corrected same-lease prerequisite build exited 0; frontend selected 3 passed, 0 failed, 0 ignored; the two retained same-head unit selections each passed 1, 0 failed, 0 ignored. All five targeted CLI tests therefore pass, clearing the current-head runtime condition above.

The bounded integration disposition is CLEAR and ready for the parent to copy and merge. Historical Linux five and macOS four host results remain bound to the earlier reviewed source, not this integration head. The historical whole-repository gap FAIL is unchanged. The remaining macOS watchdog work keeps QSL-2378 In Progress and is not claimed complete by this merge. This reviewer ran no Cargo, measurements or Linear operations.

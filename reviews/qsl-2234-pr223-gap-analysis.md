---
id: SR-202611210022442313144108515
title: "QSL-2234 PR223 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; PR223; all 18 changed files; whole repository traceability baseline"
review_set: subset
relationships:
  - {target: ix://agent-ix/filament-driver, type: references}
---

## Summary

FAIL: 175 untagged and five ignored-only criteria in the whole-repository baseline. No new missing criterion; two criteria improve to tagged. Ticket: QSL-2234. Reviewed frozen `f3711b7ee4408261ba29f0d69357fb791d31f1ee` against `260c35acc70783e4bbb3d580f582f2cd57c24454`.

## Verdict

FAIL: 175 untagged and five ignored-only criteria in the whole-repository baseline. No new missing criterion; two criteria improve to tagged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-001-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:322 |
| FND-002 | high | FR-001-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:323 |
| FND-003 | high | FR-001-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:324 |
| FND-004 | high | FR-001-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:326 |
| FND-005 | high | FR-001-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:327 |
| FND-006 | high | FR-001-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:328 |
| FND-007 | high | FR-001-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:329 |
| FND-008 | high | FR-001-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:330 |
| FND-009 | high | FR-001-AC-13 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:333 |
| FND-010 | high | FR-001-AC-14 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:334 |
| FND-011 | high | FR-001-AC-23 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:343 |
| FND-012 | high | FR-001-AC-25 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-001-orchestrate-the-driver-steps.md:345 |
| FND-013 | high | FR-002-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:330 |
| FND-014 | high | FR-002-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:331 |
| FND-015 | high | FR-002-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:332 |
| FND-016 | high | FR-002-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:333 |
| FND-017 | high | FR-002-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:334 |
| FND-018 | high | FR-002-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:335 |
| FND-019 | high | FR-002-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:336 |
| FND-020 | high | FR-002-AC-11 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:337 |
| FND-021 | high | FR-002-AC-12 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:338 |
| FND-022 | high | FR-002-AC-14 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:340 |
| FND-023 | high | FR-002-AC-15 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:341 |
| FND-024 | high | FR-002-AC-16 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:342 |
| FND-025 | high | FR-002-AC-18 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:344 |
| FND-026 | high | FR-002-AC-19 is tagged only by ignored tests at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:345 |
| FND-027 | high | FR-002-AC-20 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:346 |
| FND-028 | high | FR-002-AC-21 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:347 |
| FND-029 | high | FR-002-AC-28 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:354 |
| FND-030 | high | FR-003-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-003-apply-caller-selected-limits.md:87 |
| FND-031 | high | FR-003-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-003-apply-caller-selected-limits.md:88 |
| FND-032 | high | FR-003-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-003-apply-caller-selected-limits.md:91 |
| FND-033 | high | FR-003-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-003-apply-caller-selected-limits.md:92 |
| FND-034 | high | FR-003-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-003-apply-caller-selected-limits.md:93 |
| FND-035 | high | FR-003-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-003-apply-caller-selected-limits.md:94 |
| FND-036 | high | FR-003-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-003-apply-caller-selected-limits.md:95 |
| FND-037 | high | FR-004-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-004-select-the-execution-engine.md:100 |
| FND-038 | high | FR-004-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-004-select-the-execution-engine.md:101 |
| FND-039 | high | FR-004-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-004-select-the-execution-engine.md:102 |
| FND-040 | high | FR-004-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-004-select-the-execution-engine.md:103 |
| FND-041 | high | FR-004-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-004-select-the-execution-engine.md:104 |
| FND-042 | high | FR-004-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-004-select-the-execution-engine.md:105 |
| FND-043 | high | FR-004-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-004-select-the-execution-engine.md:106 |
| FND-044 | high | FR-004-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-004-select-the-execution-engine.md:107 |
| FND-045 | high | FR-005-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:151 |
| FND-046 | high | FR-005-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:152 |
| FND-047 | high | FR-005-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:153 |
| FND-048 | high | FR-005-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:154 |
| FND-049 | high | FR-005-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:155 |
| FND-050 | high | FR-005-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:156 |
| FND-051 | high | FR-005-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:157 |
| FND-052 | high | FR-005-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:158 |
| FND-053 | high | FR-005-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:159 |
| FND-054 | high | FR-005-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:160 |
| FND-055 | high | FR-005-AC-11 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:161 |
| FND-056 | high | FR-005-AC-12 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:162 |
| FND-057 | high | FR-005-AC-13 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:163 |
| FND-058 | high | FR-005-AC-14 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:164 |
| FND-059 | high | FR-005-AC-15 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:165 |
| FND-060 | high | FR-005-AC-16 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:166 |
| FND-061 | high | FR-005-AC-17 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:167 |
| FND-062 | high | FR-005-AC-18 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:168 |
| FND-063 | high | FR-005-AC-19 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:169 |
| FND-064 | high | FR-005-AC-20 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:170 |
| FND-065 | high | FR-005-AC-21 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-005-register-first-party-providers.md:171 |
| FND-066 | high | FR-006-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:229 |
| FND-067 | high | FR-006-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:230 |
| FND-068 | high | FR-006-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:231 |
| FND-069 | high | FR-006-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:232 |
| FND-070 | high | FR-006-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:233 |
| FND-071 | high | FR-006-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:234 |
| FND-072 | high | FR-006-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:235 |
| FND-073 | high | FR-006-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:236 |
| FND-074 | high | FR-006-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:237 |
| FND-075 | high | FR-006-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:238 |
| FND-076 | high | FR-006-AC-11 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:239 |
| FND-077 | high | FR-006-AC-12 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:240 |
| FND-078 | high | FR-006-AC-13 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:241 |
| FND-079 | high | FR-006-AC-14 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:242 |
| FND-080 | high | FR-006-AC-15 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:243 |
| FND-081 | high | FR-006-AC-16 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:244 |
| FND-082 | high | FR-006-AC-17 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:245 |
| FND-083 | high | FR-006-AC-18 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:246 |
| FND-084 | high | FR-006-AC-19 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:247 |
| FND-085 | high | FR-006-AC-20 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:248 |
| FND-086 | high | FR-006-AC-21 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:249 |
| FND-087 | high | FR-006-AC-22 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:250 |
| FND-088 | high | FR-006-AC-23 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:251 |
| FND-089 | high | FR-006-AC-25 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:253 |
| FND-090 | high | FR-006-AC-26 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:254 |
| FND-091 | high | FR-006-AC-27 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:255 |
| FND-092 | high | FR-006-AC-28 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:256 |
| FND-093 | high | FR-006-AC-29 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-006-host-provider-plugins.md:257 |
| FND-094 | high | FR-007-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-007-cache-results-by-content-identity.md:56 |
| FND-095 | high | FR-007-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-007-cache-results-by-content-identity.md:57 |
| FND-096 | high | FR-007-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-007-cache-results-by-content-identity.md:58 |
| FND-097 | high | FR-007-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-007-cache-results-by-content-identity.md:59 |
| FND-098 | high | FR-007-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-007-cache-results-by-content-identity.md:60 |
| FND-099 | high | FR-007-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-007-cache-results-by-content-identity.md:61 |
| FND-100 | high | FR-008-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-008-aot-execution-backend.md:44 |
| FND-101 | high | FR-008-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-008-aot-execution-backend.md:45 |
| FND-102 | high | FR-009-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:263 |
| FND-103 | high | FR-009-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:264 |
| FND-104 | high | FR-009-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:265 |
| FND-105 | high | FR-009-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:266 |
| FND-106 | high | FR-009-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:267 |
| FND-107 | high | FR-009-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:270 |
| FND-108 | high | FR-009-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:271 |
| FND-109 | high | FR-009-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:272 |
| FND-110 | high | FR-009-AC-11 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:273 |
| FND-111 | high | FR-009-AC-12 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:274 |
| FND-112 | high | FR-009-AC-13 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:275 |
| FND-113 | high | FR-009-AC-14 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:276 |
| FND-114 | high | FR-009-AC-15 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:277 |
| FND-115 | high | FR-009-AC-20 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:282 |
| FND-116 | high | FR-009-AC-21 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:283 |
| FND-117 | high | FR-009-AC-22 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:284 |
| FND-118 | high | FR-009-AC-23 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:285 |
| FND-119 | high | FR-009-AC-24 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-009-settle-execute-and-replay-results.md:286 |
| FND-120 | high | FR-011-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-011-cancel-a-driver-library-operation.md:87 |
| FND-121 | high | FR-011-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-011-cancel-a-driver-library-operation.md:88 |
| FND-122 | high | FR-011-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-011-cancel-a-driver-library-operation.md:89 |
| FND-123 | high | FR-011-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-011-cancel-a-driver-library-operation.md:90 |
| FND-124 | high | FR-012-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:83 |
| FND-125 | high | FR-012-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:84 |
| FND-126 | high | FR-012-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:85 |
| FND-127 | high | FR-012-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:86 |
| FND-128 | high | FR-012-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:87 |
| FND-129 | high | FR-012-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:88 |
| FND-130 | high | FR-012-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:89 |
| FND-131 | high | FR-012-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:90 |
| FND-132 | high | FR-012-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:91 |
| FND-133 | high | FR-012-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:92 |
| FND-134 | high | FR-012-AC-11 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:93 |
| FND-135 | high | FR-012-AC-12 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md:94 |
| FND-136 | high | FR-013-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:119 |
| FND-137 | high | FR-013-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:120 |
| FND-138 | high | FR-013-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:121 |
| FND-139 | high | FR-013-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:122 |
| FND-140 | high | FR-013-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:123 |
| FND-141 | high | FR-013-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:124 |
| FND-142 | high | FR-013-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:125 |
| FND-143 | high | FR-013-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:126 |
| FND-144 | high | FR-013-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:127 |
| FND-145 | high | FR-013-AC-11 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:128 |
| FND-146 | high | FR-013-AC-12 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:129 |
| FND-147 | high | FR-013-AC-13 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:130 |
| FND-148 | high | FR-013-AC-14 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:131 |
| FND-149 | high | FR-013-AC-15 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-013-host-process-plugins.md:132 |
| FND-150 | high | FR-014-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-014-separate-the-user-cli-package.md:86 |
| FND-151 | high | FR-014-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-014-separate-the-user-cli-package.md:91 |
| FND-152 | high | FR-015-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:398 |
| FND-153 | high | FR-015-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:399 |
| FND-154 | high | FR-015-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:400 |
| FND-155 | high | FR-015-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:401 |
| FND-156 | high | FR-015-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:402 |
| FND-157 | high | FR-015-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:403 |
| FND-158 | high | FR-015-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:404 |
| FND-159 | high | FR-015-AC-8 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:405 |
| FND-160 | high | FR-015-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:406 |
| FND-161 | high | FR-015-AC-10 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:407 |
| FND-162 | high | FR-015-AC-11 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:408 |
| FND-163 | high | FR-015-AC-12 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:409 |
| FND-164 | high | FR-015-AC-14 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:411 |
| FND-165 | high | FR-015-AC-15 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:412 |
| FND-166 | high | FR-017-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md:276 |
| FND-167 | high | FR-017-AC-2 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md:277 |
| FND-168 | high | FR-017-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md:278 |
| FND-169 | high | FR-017-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md:282 |
| FND-170 | high | FR-019-AC-1 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:212 |
| FND-171 | high | FR-019-AC-3 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:214 |
| FND-172 | high | FR-019-AC-4 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:215 |
| FND-173 | high | FR-019-AC-5 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:216 |
| FND-174 | high | FR-019-AC-6 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:217 |
| FND-175 | high | FR-019-AC-7 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:218 |
| FND-176 | high | FR-019-AC-9 is untagged in the computed matrix at both the exact base and reviewed head. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:220 |
| FND-177 | high | FR-026-AC-1 is tagged only by ignored tests at both the exact base and reviewed head. | spec/functional/FR-026-write-the-replay-results-with-fsl-replay.md:88 |
| FND-178 | high | FR-026-AC-2 is tagged only by ignored tests at both the exact base and reviewed head. | spec/functional/FR-026-write-the-replay-results-with-fsl-replay.md:89 |
| FND-179 | high | FR-026-AC-3 is tagged only by ignored tests at both the exact base and reviewed head. | spec/functional/FR-026-write-the-replay-results-with-fsl-replay.md:90 |
| FND-180 | high | FR-026-AC-4 is tagged only by ignored tests at both the exact base and reviewed head. | spec/functional/FR-026-write-the-replay-results-with-fsl-replay.md:91 |

## Review evidence

Frozen PR #223: `f3711b7ee4408261ba29f0d69357fb791d31f1ee` against `260c35acc70783e4bbb3d580f582f2cd57c24454`. Ticket: QSL-2234. All 18 changed files examined. No owned spec/plan edits, so spec-review and its sub-analyses do not apply. No applicable AssuranceProfile was found. Optional additional semantic gap review was not requested and was not run.

The Rust lane applied API, implementation, testing, safety/concurrency and wire/persistence references, repository AGENTS.md/CLAUDE.md, contribution/lint/formatter/source-policy instructions and CI diff. No CI lane changes. No Tokio path changes. No source, branch, push, build or runtime mutation was performed by this reviewer.

Ownership: the actual FSL Braking source and request files are consumed in place from the declared fsl-replay dependency using Cargo metadata. No copied native program or private upstream API. The new shared resolver replaces two existing metadata resolver implementations in monitor_refusal and source_record_admission. Composition independently overlays the existing driver Counter library; the process fixture scripts external wire answers rather than replacing the reference solver. The strong Reservoir oracle is unchanged apart from removing its ignore.

Lifecycle: Engine::hybrid_process receives the admitted descriptor and opaque exchange over the existing HostProcessSession/PluginSession. Owner request composition, strict response reading, proof trust, certificate checks and native replay remain in FSL. Unknown response members survive driver transport. Mixed value/hybrid registration is the explicitly authorized temporary refusal with all admitted sessions retained for cleanup; this review does not claim combined-engine implementation or QSL-2234 closure. Private per-exchange timer tokens preserve the global parent token. Post-wait checks reject late successful bytes and preserve parent cancellation precedence; typed failures map into FSL HybridExchangeError. Real child abort/grace/reap remains the host lifecycle.

Local cheap checks: git diff --check and cargo fmt --all --check both exit 0. Supplied ROOTTEST-RUN AA71 evidence reports two production builds, three focused Clippy invocations and 25/26 runtime tests passing, zero ignored; the sole failure was the composition fixture's duplicate subject alias, then fixed in f3711b7. The actual production source is unchanged in f3711b7. Exact-head composition execution and frontend Clippy remain pending in this review; no final-head passing runtime claim is made. Remote ROOT owns all build/test/Clippy work. Full-crate/all-targets/feature sweeps, Kani/proof and broad gates were not authorized for this reviewer. No runtime result is inferred from matrix trace tags.

Module/tool provenance: quoin 0.36.8 at /home/peter/.npm-global/bin/quoin; quire 0.52.0 (engine 0.62.2) at /home/peter/.npm-global/bin/quire; spec-artifacts-process@v0.36.0. Both matrix commands and coverage commands exited 0. Coverage emitted two cfg symbol-id-collision notices (signals::run and frontend invalid_os_string); both are present in the exact base as well.

## Coverage

Repository reverse inventory: compilation/emission/routing/generation (FR-001/003/004), CLI/outcomes/run (FR-002), scalar execution (FR-009), providers/process framing/lifecycle (FR-005/006/013/014), caching (FR-007), replay (FR-026), inspection/diagnostic/source intake and monitor preparation (FR-010/011/012/015/017/019/020/021/023/024/025/027), and analyze/composition/hybrid rendering (FR-022). These behavior groups were inventoried over src/, fsl-cli/src/, filament-plugin-host/src/ and filament-cache/src/ and linked through their existing requirement/module descriptions; no confirmed new unowned behavior, source stub, test stub or hollow coverage claim was found. Intentional library reexports and external test fixture answers are not treated as stubs. This is an inventory and focused changed-code inspection, not a claim of exhaustive semantic verification of every historical implementation.

Plan completion: not assessed

Reconciliation: quoin matrix (quoin 0.36.8, quire 0.52.0 / engine 0.62.2), whole repository. 317 criteria: 127 tagged, 175 untagged, 5 tagged-by-ignored-test, 10 method-without-symbol; qualified-record 0, retired 0. All 317 have no run evidence in the quoin store. Execution evidence above was supplied separately, not imported or invented as engine evidence.

Exact-base comparison: 125 tagged, 177 untagged, 5 ignored-only, 10 method-without-symbol. The ONLY criterion-status changes are FR-002-AC-22 and FR-022-AC-11, both untagged to tagged. No new criterion gap. Coverage: 201 untracked symbols on both snapshots, 1160 diagnostics on both snapshots; bound symbols 246 to 262, tagged 742 to 760, candidates 773 to 791. Unmatched tags 898 to 903: the five new unmatched bindings are FR-013-AC-6/TC-025 on request_timeout_rejects_a_reply_arriving_inside_the_receive_wait and FR-013-AC-5/FR-013-AC-6/TC-025 on request_timeout_preserves_parent_cancellation_over_a_late_ready_reply. They use the documented `/// Trace:` form, but this active module binds only rust-trace-attribute. They therefore are not engine-backed bindings. This extraction limitation is explicitly retained; supplied actual passing host execution is a different kind of evidence. Existing unmatched/upstream/provenance tags are not silently counted as covered local criteria. No Test Matrix was written.

The 180 high findings below report the unchanged whole-repository assurance baseline, not 180 newly introduced PR regressions. No historical repair tickets or unrelated scope expansion was requested. The dispatching ROOT must obtain one driver-specific ARCH baseline decision before merging; the codegen-specific earlier ruling cannot be reused. The audit verdict remains FAIL even if that separate merge exception is approved.

## Examined units

```yaml
scope:
- id: FR-001-AC-1
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'The driver''s normal and build dependency set matches FSL ADR-011 §6.1
    and the specified driver-layer exceptions: it contains exactly `fsl-analyze`,
    `fsl-foundation`, `fsl-package`, `fsl-replay`, `fsl-route`, `fsl-semantics`, `filament-exact`,
    `filament-contract-codegen`, `filament-contract-model`, `filament-contract-runtime`,
    `quire-canonical`, `filament-observation`, `filament-semantic-value`, `serde`,
    `serde_json` and `thiserror`; `serde_json` is required for production projection
    of FR-331''s retained optional `bounds` member and RT supplies FR-015/FR-017''s
    native retained program/arena type'
- id: FR-001-AC-2
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: A run whose registry holds a `BackendId` with no CG kind refuses before
    any negotiation, with `invalid_capability`/`unknown-backend` and the unconverted
    ids in bytewise order.
- id: FR-001-AC-3
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: '`route` is called only after negotiation, and only for `supported` items.'
- id: FR-001-AC-4
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'The steps run in the order of the Description: compile and emit, the E5
    read, the candidate step, conversion, negotiation, routing, generation.'
- id: FR-001-AC-5
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: Each command-level refusal carries its step, catalog code and outcome kind.
    A `Compile` refusal's code renders `code/cause` when the refusing stage names
    a cause (e.g. `missing_import`/`missing-selection`), not just the bare code, including
    FSL's native extension refusal cause.
- id: FR-001-AC-6
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: CG generation is called once, after routing, over the routed items only,
    each with its request index, its admitted `CheckedNodeId`, its routed backend
    and its kind; its output is keyed by request index, ascending; a missing kind
    context refuses the run, and a routed item naming a node the checked package does
    not hold is an internal fault.
- id: FR-001-AC-7
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: The E5 read receives every caller-supplied FR-056 domain package document
    under its supplied identity and bytes unchanged, then admits the emitted v2 bytes;
    an IR refusal is an internal fault at the hand-off before any backend conversion.
- id: FR-001-AC-8
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: CG generation's output carries the FR-014 claim map CG derived for the
    Kani group (IR-294), `Some` whenever one ran; a node whose operation has no derivable
    claim settles `Unsupported` with `no_derivable_claim` and does not affect its
    routed siblings' records or harnesses.
- id: FR-001-AC-9
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: The same caller-owned `Cancel` handle is passed to FSL `parse`, `select`,
    `check` and `package`; cancellation returns `DriverRefusal::Cancelled` with FSL's
    stage and cause, and the driver adds no category or exit mapping.
- id: FR-001-AC-10
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: E5 receives the E4 `EmittedUnit` source provision; the returned `Driven`
    contains no `EmittedUnit`, retains the `CheckedUnit` root source for diagnostic
    loci, retains no library source under the current empty `DependencyInput`, and
    retains no more root-source bytes than `SpineLimits.source.source_bytes`.
- id: FR-001-AC-11
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: The normal/build dependency closure rooted at `filament-driver` contains
    no `filament-cache` package.
- id: FR-001-AC-12
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: For each request item, the driver's CG negotiation input carries the typed
    IR tag/form projected from the admitted checked requirement and node named by
    that same item's FSL requirement record; generation preserves that item's exact
    application occurrence and direct conversion-context occurrence as ordered lowering
    roots, including when several items share one application node. No caller-computed
    lowerability/refusal or display/debug-derived form substitutes for the checked
    form.
- id: FR-001-AC-13
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'The driver''s internal scalar-generation composition receives the caller''s
    unchanged FSL `Call`, `ObjectEnvironment`, and `Cancel` with the request''s compile
    input. FSL binding and IR admission use the package produced by that same request:
    the exact emitted bytes are admitted once and are not reconstructed or re-admitted.
    The generated result remains associated with that call and admitted package in
    the driver library.'
- id: FR-001-AC-14
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: For each routed scalar item selected for native proof, the driver joins
    the FSL item and CG driver item by request index and exact `OccurrenceKey`, then
    uses FSL `bind_scalar_operands` on that call's package and exact function/application/occurrence.
    Bound operands remain in FSL order, matching the CG scalar identity's argument
    order, with their argument source and ordinal; `ScalarLimits` come from the same
    call's accounting. Two occurrences on one node remain distinct. No positional
    zip or node-only occurrence selection is permitted.
- id: FR-001-AC-15
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: CG receives the original E5-admitted `CheckedPackageV2` by reference alongside
    its existing admitted `ContractPackageV1`, complete FR-330 position vector, routed
    items, and generation contexts. The driver retains and shares the original admitted
    source owner without reconstruction or deep copying. The position vector retains
    one position per original request item, in driver-index order (a `DriverItem`
    for each bound item, an excluded position, keeping the item's `node_id` and counted
    in driver indices, for each export-refused item; excluded positions are absent
    from the FR-331 envelope), and a
- id: FR-001-AC-16
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'No selected scalar item means no scalar binding or native execution and
    no fallback selection. FSL''s typed missing/ambiguous call or occurrence refusal
    is preserved unchanged. A missing/ambiguous required conversion context preserves
    its owning FSL/IR/CG typed refusal; `None` is valid only when the exact FSL item
    has no conversion context. An admitted-package/request-position/routed-item mismatch
    preserves the owning IR/CG typed refusal, or is a driver `runtime_invariant` fault
    when the driver detects an inconsistent join. Each case stops before native execution
    or replay and invents no call, '
- id: FR-001-AC-17
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: For the FR-010-AC-4 vector, the run completes, `Driven` holds the settlements
    of A (driver index 0) and C (driver index 2) and B's refusal, unchanged, with
    its occurrence key and driver index 1 in the per-item refusal member, and no `DriverRefusal`
    is returned.
- id: FR-001-AC-18
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: When the export refuses every requested item, `Driven` holds no settled
    item and one refusal in the per-item refusal member for each requested item.
- id: FR-001-AC-19
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: An export that fails as a whole refuses the run with the export's typed
    refusal, and the refusal carries the bindings-export step.
- id: FR-001-AC-20
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: An occurrence key the driver built that the export reports as naming no
    requirement record is a `runtime_invariant` internal fault.
- id: FR-001-AC-21
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: When the package declares no abstraction relation, the driver does not
    call the export, every item goes to CG, and the per-item refusal member is empty.
- id: FR-001-AC-22
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver
    emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG
    projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`,
    `document_pointer`, `expected` and `actual`. The diagnostic carries the same code
    and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location,
    and carries `locus` as a `CheckedNodeId` payload when present; every returned
    code has an FR-271 row. When CG returns `ContractPackageReadIncomplete`, the driver
    emits `stage_limit_exc'
- id: FR-001-AC-23
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: Two generated function items that select the same checked node at different
    application occurrences retain two distinct contexts keyed by their original request
    indices. Each context contains the FSL-selected function declaration and exact
    occurrence, its producing source and dependency lock, the same E5-admitted package
    identity, and the matching CG `KaniObligationIdentity`; CG `ReplayInputs` use
    those retained source, lock, accounting, stage, domain and replay inputs; playback
    intake uses the smaller caller/proving-run byte and value decode limits. No identity
    or source is reconstructed from
- id: FR-001-AC-24
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: A FSL selection, IR admission or CG generation refusal is preserved as
    that owner’s typed refusal. After all three succeed, a missing, duplicate or mismatched
    request-index, occurrence, package, declaration or generated-identity association
    returns a driver `runtime_invariant` fault before provider execution or replay,
    without selecting another item or fallback context.
- id: FR-001-AC-25
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: The driver passes the caller's `SpineLimits.package.emit()` to FSL admission
    evidence at E5, preserving the exact `emit.output_bytes` value used for E4, including
    a caller-selected value.
- id: FR-002-AC-1
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: The driver binary exposes the command and output surface defined by FSL
    ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation.
- id: FR-002-AC-2
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: For each FSL category whose exit result is not reserved code `5`, the process
    status equals the value returned by FSL's `Category::exit_code`; no second category-to-exit
    mapping is applied. Reserved result `5` is governed by AC-6.
- id: FR-002-AC-3
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: In machine mode, each command writes one parseable `filament-outcome/1`
    value equal to the library outcome under FSL's schema; the CLI adds, removes or
    changes no outcome data. The mapping-invariant and signal exceptions are specified
    in AC-6 and AC-7.
- id: FR-002-AC-4
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: The CLI passes FSL's `Category::exit_code` result through `ix_cli_kit::exit::caller_exit_code`
    and preserves each accepted numeric value without mapping it through the kit's
    typed `Outcome`.
- id: FR-002-AC-5
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: The CLI reads only input paths named by its arguments and reads no unnamed
    configuration file.
- id: FR-002-AC-6
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: If FSL's category exit function returns reserved value `5`, the CLI writes
    a plain-text stderr diagnostic naming reserved code `5` and fallback code `30`,
    writes no text result or `filament-outcome/1` document to stdout, and returns
    `Category::InternalFailure.exit_code()` (`30`) without attributing a FSL cause.
- id: FR-002-AC-7
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: When a Linux or macOS invocation receives SIGINT or SIGTERM during an active
    plugin session, the CLI cleans up, restores the default disposition, and re-raises
    that signal without writing a result or outcome document or inventing a numeric
    exit value or FSL cause.
- id: FR-002-AC-8
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: When the CLI writes results and diagnostics, it uses ix-cli-kit's `streams`
    module for stream discipline.
- id: FR-002-AC-9
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: When the CLI writes machine-mode outcomes, it uses ix-cli-kit's `json`
    module for canonical JSON encoding.
- id: FR-002-AC-10
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: When the CLI resolves configuration values, it uses ix-cli-kit's `config`
    module.
- id: FR-002-AC-11
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: When invoked with `--version`, the CLI reports the semantic version declared
    by the `fsl-cli` Cargo package.
- id: FR-002-AC-12
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: The `fsl-cli` built-binary version-agreement check uses `ix_cli_kit::version::Agreement::command`
    to compare `CARGO_PKG_VERSION` with the output of the built `fsl --version` command
    and requires agreement.
- id: FR-002-AC-13
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl check <file>`, `fsl parse <authority> <identity> <file>`, `fsl compile
    <request>` and `fsl run <request>` without `--format`, and with `--format text`,
    each over a succeeding and a refused input, write to stdout the FSL FR-865 text
    form of the document `--format json` writes for the same invocation (header `<operation>:
    <category> (exit <code>)`, diagnostic lines with cause and locus, item lines,
    result line) and exit with the same status as under `--format json`; none returns
    `UnimplementedCapability` or exit 21.'
- id: FR-002-AC-14
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: For a `BrokenPipe` while writing or flushing primary stdout, the process
    status equals the completed operation's FSL category exit result and stderr is
    empty, including for a refused operation with nonempty diagnostics when stdout
    is writable. No output-failure category is selected and no `OutputFailure` diagnostic
    is emitted.
- id: FR-002-AC-15
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: If writing or flushing primary stdout returns an I/O error other than `BrokenPipe`,
    the process exits with FSL's `Category::InternalFailure` exit result. The criterion
    makes no assertion that stdout is empty or complete.
- id: FR-002-AC-16
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: If a non-`BrokenPipe` primary stdout write or flush fails and stderr is
    writable, stderr contains an `OutputFailure` diagnostic that identifies the failed
    output.
- id: FR-002-AC-17
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: With no arguments or with each of `--help`, `-h` and `help` as the sole
    argument, the built `fsl` binary writes root help to stdout containing invocation
    syntax, recognized lifecycle verb names, `--format text\
- id: FR-002-AC-18
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl run nosuch.json` and `fsl compile nosuch.json` without `--format`
    refuse with the I/O failure''s FSL code and exit 20, as under `--format json`,
    never `UnimplementedCapability` or exit 21, and stdout holds the text form of
    that refusal document; the text of a refused `check` names each diagnostic''s
    code, cause and source position; `--format bogus` on each of the four verbs refuses
    `invalid-request`, exit 20, naming `--format` and `bogus`, before any file is
    opened.'
- id: FR-002-AC-19
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl check P --format json` over a source at path `P` that does not parse
    writes the `check` refusal whose every diagnostic locus names `{"authority": "filament",
    "identity": P}`, with `P` exactly as given, and that contains neither `fsl-command`
    nor `check-source`; its stdout is byte-equal to the canonical encoding of the
    document FSL''s `parse` gives for the same bytes under that identity, plus a newline,
    with that document''s exit status, and `compile` and `run` over requests naming
    the same source refuse equally to their library operations.'
- id: FR-002-AC-20
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl run R --format json`, `R` FSL FR-931-AC-1''s request `{"clause":
    "Low"}` over the ConfigVersion model, writes the `tl_lowering` record with `disposition`
    `supported`, `correspondence` `mltl.closed-trace/v1` and `formula` `G[0,3] p0`,
    exit 0, and its stdout is byte-equal to the canonical encoding of the document
    FSL''s `fsl_command::command::run` returns for `R`, plus a newline; it never writes
    the "not run by the driver" input refusal. `fsl run R --limit s1.nodes=1 --format
    json` writes FSL''s typed limit outcome naming `s1.nodes` with configured value
    1, category `incomplete`, no lowering r'
- id: FR-002-AC-21
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl run P --format json`, `P` FSL FR-1039-AC-1''s request selecting the
    `half` unit''s checked package as `package` `{file}` with the `call` `half`, run
    with the source file removed, completes with `-4`, exit 0, byte-equal to FSL''s
    `run` for `P` plus a newline; the same request with `integer_division` gives FSL''s
    `invalid-request` refusal naming `integer_division`, exit 20.'
- id: FR-002-AC-22
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl run Q --format json`, `Q` FSL FR-960-AC-1''s request calling `sq`
    with `x = 3` and `call.requirements` `{}`, with no `--provider-config`, writes
    stdout byte-equal to the canonical encoding of the document `run_with_providers_and_limits_and_overrides`
    returns for `Q` with the providers `[Engine::hybrid_reference()]` and default
    limits, plus a newline: value `{"integer": "16"}` and three settled requirement
    entries, exit 0. With `--provider-config` starting a fixture provider whose admitted
    backend is `fsl-hybrid-reference`, the run refuses `invalid-request` naming `providers`,
    exit 20, beca'
- id: FR-002-AC-23
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: 'Over a handoff directory `H` written by `fsl compile` from a `native-compile/1`
    request carrying FSL FR-665-AC-1''s `handoff` member, `fsl inspect --handoff H
    --format json` writes stdout byte-equal to the canonical encoding of the document
    `fsl_command::command::inspect_handoff(H)` returns, plus a newline: operation
    `inspect`, category success, exit 0; without `--format` it writes that document''s
    FSL FR-865 text form, exit 0. Over a copy of `H` whose `selection.json` names
    one node count more, and over a copy whose `checked-package.json` has one changed
    byte, each run refuses with the `Selecti'
- id: FR-002-AC-24
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: For every verb that executes operations and every option `o` in its `fsl-cli`
    option table entry, `o` appears on its own line in the verb's `<verb> options:`
    section of root help and the built parser accepts `o` with a well-formed value
    (no `invalid-request` naming `o`). For every verb that executes operations and
    every option name `p` that is listed in another verb's entry or is one of `--version`,
    `-V`, `--color` and `--bogus`, but is not in this verb's entry, `fsl <verb> p`
    refuses `invalid-request`, exit 20, naming `p`. Removing one option from a copy
    of the table and running the parser an
- id: FR-002-AC-25
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl check --help`, `fsl check -h` and `fsl check nosuch.fsl --help`,
    from an empty working directory, each write exactly the following to stdout, exit
    0, write no stderr and read no file: `Usage: fsl check [options] <source-path>`
    / `       fsl check [options] <authority> <identity> <source-path>` / (blank)
    / `check options:` / `  --markdown` / `  --model <identity>=<file> (repeatable)`
    / `  --library <identity>=<file> (repeatable)` / `  --dependent-bytes <bytes>`
    / `  --limit name=value (repeatable)` / `  --integer-division euclidean\'
- id: FR-002-AC-26
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl run R --color always`, `fsl run R --color never` and `--color` on
    every other verb that executes operations refuse `invalid-request`, exit 20, naming
    `--color`, before `R` or any other named file is read, and no help section contains
    `--color`. Over inputs that contain no `0x1B` byte, no `fsl` invocation of TC-049
    writes the byte `0x1B` to stdout or stderr, with or without `NO_COLOR` set.'
- id: FR-002-AC-27
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: 'For FSL FR-960-AC-1''s native-run request `Q` whose call carries `requirements`,
    `fsl run Q --format json` writes FR-002-AC-22''s document (value `{"integer":
    "16"}`, three settled requirement entries), exit 0. `fsl run Q --limit s1.nodes=50000
    --format json` writes the same document. FSL''s provider run entry carries stage
    limits only, so `fsl run Q --limit accounting.work_units=1000000` and `fsl run
    Q --limit observation.input_bytes=1000000` each refuse `invalid-request`, exit
    20, naming the setting and `requirements`, and run nothing. `fsl run Q --limit
    s1.nodes=1 --format json` writes FSL''s t'
- id: FR-002-AC-28
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl prove R --integer-division floor --format json` and `fsl prove R
    --bogus --format json` refuse `invalid-request`, exit 20, as an `invalid-request`
    outcome document with operation `prove` naming the option, before the verb answers
    unsupported; `fsl prove R --format json`, `fsl prove --help` and `fsl prove --version`
    answer unsupported, exit 21 (AC-25 governs help and version on a verb the build
    does not route).'
- id: FR-003-AC-1
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: Setting one FSL-resolved `--limit` operand overrides only its exposed bound;
    omitted settings retain their published defaults, and effective limits passed
    to the operation equal those values.
- id: FR-003-AC-2
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: A setting name not resolved by FSL's operand parser is refused before any
    stage with code `invalid-request` and names the offending operand.
- id: FR-003-AC-3
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: An input that reaches a caller-lowered bound returns typed `LimitExceeded`
    with the configured value and field name and FSL's incomplete category; the process
    status equals FSL's category exit function. Raising the same bound to fit the
    input allows the operation to complete.
- id: FR-003-AC-4
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: The driver uses FSL `parse_operands`, checks each resolved stage setting
    against the selected operation's `SettingLimits::bounds()`, and applies an exposed
    stage setting with `CallerLimits::set_bound`; accounting settings bypass stage
    bounds and use `AccountingLimits::set_bound` on `Call.accounting`, and the driver
    maintains no duplicate grammar, alias table, or name-to-field mapping.
- id: FR-003-AC-5
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: A stage setting parsed by FSL but absent from the selected operation's
    `SettingLimits::bounds()` is refused before any stage with code `invalid-request`
    and a distinct diagnostic naming the setting as unavailable for that operation.
- id: FR-003-AC-6
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: Applying an accepted setting through FSL `CallerLimits::set_bound` changes
    only the matching bound; every other operation limit retains its prior value.
- id: FR-003-AC-7
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: A malformed `--limit` operand is refused before any stage with code `invalid-request`
    and identifies that operand.
- id: FR-003-AC-8
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: A duplicate `--limit` setting is refused before any stage with code `invalid-request`
    and identifies the duplicate setting.
- id: FR-003-AC-9
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: The driver accepts FSL FR-255's `accounting.<counter>` setting names through
    FSL's operand APIs and retains FSL's unknown-setting refusal for a bare counter
    such as `work_units`, without adding a driver-defined namespace or aliases.
- id: FR-003-AC-10
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: examined
  excerpt: When FSL resolves an `accounting.<counter>` setting name, the driver applies
    its value through FSL `AccountingLimits::set_bound` to `Call.accounting`; the
    stage `SpineLimits` remain unchanged and the accounting value reaches `spine::run`,
    except on a `fsl run` whose call carries `requirements`, which refuses accounting
    settings (FR-002-AC-27).
- id: FR-004-AC-1
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: An execution request without `--engine` runs the interpreter and returns
    its result.
- id: FR-004-AC-2
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: A value outside the closed engine set is refused before any stage with
    FSL's refusal category and names the supplied value.
- id: FR-004-AC-3
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: A recognized engine absent from the binary is refused before any stage
    with category unsupported; no other engine runs, and process status equals FSL's
    category exit function.
- id: FR-004-AC-4
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: When the selected engine returns unsupported or a typed preparation failure,
    that result is returned and no other engine is invoked.
- id: FR-004-AC-5
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: The public execution request carries the caller-owned typed FSL `ObjectEnvironment`,
    and the selected interpreter passes that same value unchanged to FSL `spine::run`.
- id: FR-004-AC-6
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: Given a function whose result depends on object context, a nonempty caller
    environment produces the FSL result for that context and differs from the result
    with an empty environment; the public library request derives no context or default.
- id: FR-004-AC-7
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: Given a FSL-defined absent object lookup whose FSL result is `Undefined`,
    the driver returns that `Undefined` outcome unchanged and does not replace it
    with a refusal or another outcome.
- id: FR-004-AC-8
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: Given a population argument whose identity has no binding in the caller-supplied
    environment, the driver returns FSL's existing typed refusal unchanged, as required
    by FSL FR-089-AC-4.
- id: FR-004-AC-9
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: The current CLI handler explicitly supplies an empty `ObjectEnvironment`
    as its caller policy because its invocation surface has no object-context input;
    public library callers still must supply the required environment themselves.
- id: FR-004-AC-10
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: The engine-invocation callback is private to the driver implementation;
    public `execute` supplies FSL `spine::run`, and the public API, CLI, configuration,
    environment, and production default expose no callback selector.
- id: FR-005-AC-1
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: A compile-time provider implementing `Provider` returns its FR-331 manifest;
    the descriptor-construction caller supplies origin `Linked`, and FSL registry
    registration preserves that origin and the manifest identity.
- id: FR-005-AC-2
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The provider receives the same emitted package, routed request, limits
    and cancellation handle supplied by the driver, and its typed results are returned
    without semantic changes. Kani generation uses caller-supplied `ProofCeilings`;
    execution requires those ceilings to match the harness identity and preserves
    CG's ceiling, memory and symbolic-argument evidence.
- id: FR-005-AC-3
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: Two providers with distinct manifest identities are registered as distinct
    backends; a run invokes only the provider selected for its routed items.
- id: FR-005-AC-4
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: 'The driver preserves FSL registry results: an equal descriptor repeat
    is idempotent; an unequal descriptor for an existing `BackendId`, including one
    with a different origin, returns the typed duplicate-backend refusal, withdraws
    that identity, and invokes neither provider.'
- id: FR-005-AC-5
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The driver preserves a caller-supplied `Process` origin, descriptor identity
    and advertised pairs when it registers the descriptor and projects it to CG unchanged.
- id: FR-005-AC-6
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The driver's shared provider-result carrier preserves each routed item's
    request index and backend identity.
- id: FR-005-AC-7
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: A plugin counterexample received before FSL replay is represented in the
    shared result carrier as a non-terminal candidate; it has no `Refuted` terminal
    until FSL S6a replay reproduces it.
- id: FR-005-AC-8
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: For Kani results, the driver passes CG's `KaniExecutionEvidence` and the
    optional replay settlement to CG's authoritative `run_terminal_value`; the driver
    implements no second mapping, and neither evidence diagnostics nor their text
    define or alter the terminal category.
- id: FR-005-AC-9
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The driver represents non-Kani provider data without Kani-specific evidence
    or failure types, while FSL remains the owner of terminal values and categories.
- id: FR-005-AC-10
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The driver does not map plugin `proved` results to a terminal until FSL
    supports its trusted proof basis, and it does not fabricate Kani `success_checks`.
- id: FR-005-AC-11
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: Once QSL-655 amends ADR-029 PL-7 to define plugin claims of `replay-parity`,
    `replay-refused`, or `kani_vacuous_proof` as invalid, the driver rejects those
    claims as tool failures.
- id: FR-005-AC-12
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: For each plugin hello, the `fsl-cli` host hello admitter calls FSL descriptor
    admission once with caller-supplied origin `Process` and returns the admitted
    descriptor to the driver.
- id: FR-005-AC-13
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The driver registers each admitted descriptor in its single FSL registry
    without re-admission, and `fsl-cli` builds no second registry.
- id: FR-005-AC-14
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The driver-owned `AdmittedRegistry` retains one lossless `ProviderManifest`
    carrier for each registered candidate.
- id: FR-005-AC-15
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The `fsl-cli` adapter converts each process hello DTO losslessly into the
    retained `ProviderManifest` carrier at admission.
- id: FR-005-AC-16
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: When building the registry for a linked provider, the driver populates
    its `ProviderManifest` carrier from that provider's FR-331 manifest; optional
    members absent from the manifest remain absent.
- id: FR-005-AC-17
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: Before negotiation, the driver projects each FSL registry descriptor through
    CG's `BackendKind::from_descriptor` and CG constructs `BackendKind::Process` from
    the supplied `Process` origin; the driver does not construct the kind or map backend
    identity to it.
- id: FR-005-AC-18
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: When a plugin hello manifest contains invalid domains, the driver passes
    the labels to FSL descriptor admission and preserves QSL-654's per-registration
    `invalid_capability` refusal with cause `invalid-domains`; the driver does not
    validate domains or construct the refusal.
- id: FR-005-AC-19
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: When a plugin hello manifest has an absent or unknown kind or mode, the
    driver preserves FSL's typed per-registration `invalid_capability` refusal.
- id: FR-005-AC-20
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: The driver forwards `KaniExecutionLimits.home_directory` unchanged to CG's
    `KaniExecutionRequest`; if it is `None`, the driver passes `None` even when the
    process environment has a `HOME` value. The driver does not infer the path from
    other directories or from the selected Kani installation.
- id: FR-005-AC-21
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: 'The typed Kani result retains CG''s `KaniExecutionEvidence` unchanged:
    a `NoVerdict` diagnostic record and a present `failure_cause` remain equal to
    CG''s values, while absent optional fields remain absent. The driver does not
    parse diagnostics or derive a failure cause from them.'
- id: FR-005-AC-22
  path: spec/functional/FR-005-register-first-party-providers.md
  role: examined
  excerpt: 'Before the linked Kani provider launches any item, it passes CG''s shipped
    general-route scalar tractability table (`TractabilityTable::shipped_general_route_scalar()`)
    to CG''s decided execution wrapper `execute_kani_obligation_decided`. A routed
    scalar harness the table refuses is not launched: the item''s result is `KaniProviderFailure::NotTractable`,
    carrying the harness path and CG''s `UnsupportedObligation::V2ClauseNotTractable`
    unchanged (node, operation identity, width, route and measured row with its versions).
    A harness the table does not refuse proceeds to CG''s own execution and its ref'
- id: FR-006-AC-1
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: When negotiation selects a process provider, the driver invokes its process-provider
    seam with the configuration entry associated with that provider's once-admitted
    hello identity, one `EmittedPackage` value, the run limit set's `deadline_ms`
    and transport limits, the registered-candidate manifest snapshot and exactly that
    provider's negotiated items; `fsl-cli` implements the seam with `filament-plugin-host`
    and passes the once-admitted hello descriptor to the driver's existing registry.
- id: FR-006-AC-2
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: Validated typed results returned through the process-provider seam enter
    the same driver proof settlement path as first-party provider results when FSL
    defines their terminal mapping; results without a current FSL mapping remain typed
    and unmapped, while unrelated routed items retain their own results.
- id: FR-006-AC-3
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: Before normal CLI command termination, the adapter requests graceful shutdown
    of every active plugin session; on Linux and macOS, if the run limit set's `grace_ms`
    expires first, the process tree terminates and cleanup completes within one second.
    The merged Windows host force-terminates the Job Object through `terminate_tree`/`terminate_job`;
    native Windows evidence remains planned. Kill-on-close is separately planned under
    IR-674.
- id: FR-006-AC-4
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is
    starting, running or shutting down requests cancellation through the caller-owned
    FSL handle, waits for settlement or the run limit set's `grace_ms` expiry, and
    cleans up the plugin process tree before the CLI restores the default disposition
    and re-raises the signal. No FSL cancellation cause is claimed; with no active
    session, the platform's default signal disposition is retained.
- id: FR-006-AC-5
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The driver composes the request manifest snapshot from the lossless FR-331
    DTO retained in each registered-candidate registry record, covering linked and
    process providers; the `fsl-cli` adapter serializes that snapshot without reconstructing
    or adding members.
- id: FR-006-AC-6
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The `fsl-cli` adapter serializes `package` as the unpadded base64url encoding
    of the emitted package's exact bytes.
- id: FR-006-AC-7
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: 'Retired: the request carries no `package_id`; the `fsl-cli` adapter serializes
    the package bytes from the `EmittedPackage` value.'
- id: FR-006-AC-8
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: 'The `fsl-cli` adapter serializes `deadline_ms` equal to the run limit
    set''s `deadline_ms`: the `--provider-limit deadline_ms=<value>` value unchanged,
    or 600000 when the option is not given; two configured providers of one run receive
    equal `deadline_ms`.'
- id: FR-006-AC-9
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The `fsl-cli` adapter serializes a request without top-level `domains`
    or `limits` members.
- id: FR-006-AC-10
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The `fsl-cli` adapter serializes each item's request index, claim locus
    and kind from the corresponding retained FSL `RequirementItem`/`CandidateOutcome`,
    using the same request data used to build typed CG negotiation.
- id: FR-006-AC-11
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The `fsl-cli` adapter serializes each item's selected backend from CG's
    routed output and its candidate set from the corresponding retained FSL `RequirementItem`/`CandidateOutcome`,
    using the same request data used to build typed CG negotiation.
- id: FR-006-AC-12
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The `fsl-cli` adapter serializes each item's extent and bounds from the
    corresponding retained FSL `RequirementItem`/`CandidateOutcome`, using the same
    request data used to build typed CG negotiation.
- id: FR-006-AC-13
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The `fsl-cli` adapter serializes each item's temporal proof bounds under
    `items[].temporal.subject.proof_bounds`.
- id: FR-006-AC-14
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The driver's shared settlement path preserves a plugin `declined` result's
    typed code in FSL `TerminalValue::Declined { code }`.
- id: FR-006-AC-15
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: A validated result without a specified FSL terminal mapping remains typed
    and unmapped in the driver's shared settlement path.
- id: FR-006-AC-16
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The driver's shared settlement path carries the FR-272 cause, diagnostic
    code and available locus and message in a FSL `OutcomeDiagnostic`; free-form message
    or `detail` text does not select a terminal.
- id: FR-006-AC-17
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: 'Retired: the package-id mismatch case no longer exists (the request carries
    no `package_id`); the id is not reused.'
- id: FR-006-AC-18
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: Given an FR-331 response for non-canonical package spelling, the `fsl-cli`
    adapter preserves each routed item identity and the FR-272 diagnostic with code
    `invalid_runtime_input` and cause `invalid-value`.
- id: FR-006-AC-19
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The driver synthesizes failed provider results only for host/protocol failures
    defined by ADR-029 PL-7; it does not attribute a process request-body refusal
    to FSL.
- id: FR-006-AC-20
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: For a preserved package-mismatch response, the driver's shared settlement
    path maps each failed routed item to FSL `TerminalValue::Failed`.
- id: FR-006-AC-21
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: For a preserved non-canonical-package response, the driver's shared settlement
    path maps each failed routed item to FSL `TerminalValue::Failed`.
- id: FR-006-AC-22
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: For package-mismatch and non-canonical-package responses, the driver's
    shared settlement path carries each typed FR-272 diagnostic separately in FSL
    `OutcomeDiagnostic`.
- id: FR-006-AC-23
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: Settling a failed process-provider response leaves unrelated provider results
    unchanged.
- id: FR-006-AC-24
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The `--provider-config` file accepts only FSpec FR-561's `filament.provider-config/v1`
    shape (`format`, `providers`, entries with `path` and optional `arguments`); a
    missing or unreadable file refuses naming the path; a non-JSON document, a missing
    `format` or `providers`, an unknown member at any object level (including an entry
    member `deadline_ms` or `limits`), an entry with no or a blank `path`, and `arguments`
    that are not strings refuse `invalid-request` naming the file and the member;
    `format` `filament.provider-config/v2` refuses `unknown_wire`/`unsupported-wire`;
    each exits 20 with em
- id: FR-006-AC-25
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: The `fsl-cli` composition starts and admits every configured provider before
    negotiation, and uses each admitted hello descriptor as the sole provider identity
    for subsequent selection and dispatch.
- id: FR-006-AC-26
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: If distinct configured sessions return the same admitted identity, the
    composition begins no negotiation and closes all sessions; it preserves a FSL
    registration refusal, or returns the existing CLI `InvalidRequest` input refusal
    when FSL treats repeated identical admission as idempotent.
- id: FR-006-AC-27
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: If a configured provider fails to start or admit its hello, the composition
    begins no negotiation, closes all earlier active sessions, and preserves the original
    typed host failure.
- id: FR-006-AC-28
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: '`fsl run --provider-config providers.json` with no `--provider-limit`
    sends the fixture plugin a request whose `deadline_ms` is 600000 and holds it
    to a 67108864-byte frame limit; with `--provider-limit deadline_ms=250 --provider-limit
    max_frame_bytes=1048576` the request carries `deadline_ms` 250 and a 1048577-byte
    result frame settles the item `failed` with the frame-length tool failure naming
    limit 1048576; `--provider-limit` is accepted without `--provider-config`.'
- id: FR-006-AC-29
  path: spec/functional/FR-006-host-provider-plugins.md
  role: examined
  excerpt: '`fsl run` with `--provider-limit max_frames=4`, `deadline_ms`, `deadline_ms=0`,
    `deadline_ms=10s`, `deadline_ms=010`, `max_total_bytes=9007199254740992`, or `deadline_ms=5`
    given twice each refuses `invalid-request` naming `--provider-limit` and the argument,
    exits 20 with empty stdout under both `--format text` and `--format json`, and
    starts no plugin; `fsl parse --provider-limit deadline_ms=5` and `fsl parse --provider-config
    providers.json` refuse the option as unknown, exit 20.'
- id: FR-007-AC-1
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: examined
  excerpt: Each lookup carries operation, package identity, canonical request, selected
    `BackendId`, and (for `generate`) target and output-affecting options.
- id: FR-007-AC-2
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: examined
  excerpt: A valid `CacheLookup::Hit` is returned unchanged and does not invoke the
    provider.
- id: FR-007-AC-3
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: examined
  excerpt: A `CacheLookup::Miss` invokes the selected provider exactly once and records
    the same typed result that the driver returns.
- id: FR-007-AC-4
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: examined
  excerpt: A cache I/O fault remains a typed error and is not treated as a miss.
- id: FR-007-AC-5
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: examined
  excerpt: '`NeverStore` and `NotStored` preserve the provider result and do not fail
    the operation.'
- id: FR-007-AC-6
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: examined
  excerpt: A registry identity refusal produces no `CacheRequest` or lookup.
- id: FR-008-AC-1
  path: spec/functional/FR-008-aot-execution-backend.md
  role: examined
  excerpt: When a request selects AOT, the driver calls the backend with the checked
    package and selected entries from that request, its limits and cancellation input,
    and the caller-named toolchain.
- id: FR-008-AC-2
  path: spec/functional/FR-008-aot-execution-backend.md
  role: examined
  excerpt: The driver returns the backend's typed results and preparation failures
    unchanged and does not invoke another engine after an AOT failure.
- id: FR-009-AC-1
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: The proof provider receives the generated request and the exact checked-package
    bytes emitted by the same request.
- id: FR-009-AC-2
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: Predicate/function replay that reproduces the provider counterexample settles
    as `refuted`.
- id: FR-009-AC-3
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: A replay refusal retains its typed cause and a replay fault is an internal
    failure; neither is converted to `refuted`.
- id: FR-009-AC-4
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: The terminal result is composed by CG's FR-331 mapping from both the provider
    result and replay result; the driver introduces no second mapping.
- id: FR-009-AC-5
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: When a proof request reaches per-item settlement, the outcome contains
    exactly one FR-331 terminal record for every item in the CG request, including
    unsupported and incomplete items; a request-level typed refusal before per-item
    settlement is governed by its refusal contract and has no per-item terminal-record
    guarantee.
- id: FR-009-AC-6
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For routed scalar replay, the driver retains one context binding the exact
    proved generated artifact and emitted package to the scalar identity, occurrence,
    ordered operands, original `ScalarLimits` and `ReplayLimits`, and exact FSL claim
    and replay context; it sends the retained `ReplayLimits` in that FSL request,
    executes the artifact with the retained scalar inputs and `ScalarLimits`, and
    retains the actual typed native outcome.
- id: FR-009-AC-7
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: The actual typed native outcome is carried separately as the claim. Before
    FSL evaluates it, the driver relies on the following checks. The opaque CG receipt
    binds the exposed entries, including CG-generated runner files, to the immutable
    snapshot CG executed; the driver checks its driver-owned entries against retained
    originals. The driver also compares the receipt's attested request/meter/returned
    `ScalarLimits` with its privately retained original limits, using the CG meter-limits
    attestation contract owned by IR-857. Missing or mismatched context, artifact
    identity, or observation prevents
- id: FR-009-AC-8
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For scalar parity, the CG-owned binding converter compares `report.claim()`
    with the exact claim sent for every report outcome; an absent report or mismatch
    yields CG FR-032's typed binding refusal with no settlement, which the driver
    surfaces unchanged and never turns into a terminal. This path remains planned
    until CG exports the public scalar binding converter; the driver adds no refusal
    mapping.
- id: FR-009-AC-9
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: Each scalar operator-parity report outcome is converted only through CG
    FR-032's typed settlement conversion of the FSL FR-357 report; `Agrees`, `Diverged`,
    `GeneratedFault`, `RefusedInput`, `Incomplete`, and other typed refusal outcomes
    retain the meanings those contracts define.
- id: FR-009-AC-10
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: Predicate/function replay that disagrees or completes without reproducing
    a provider counterexample settles as `inconclusive` with its replay-parity cause.
- id: FR-009-AC-11
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: No scalar operator-parity report settles as `refuted`.
- id: FR-009-AC-12
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: If CG returns a successful native-execution receipt that consistently attests
    request, metered, and returned `ScalarLimits` `L1` while the private retained
    context contains different original limits `L0`, the driver returns `DriverRefusal::Fault(DriverFault::NativeExecutionLimitsMismatch)`
    at the new `DriverStage::Replay` stage, with code `runtime_invariant` and outcome
    kind `Fault`, before FSL replay evaluation; no replay evaluation, settlement,
    or terminal record is created. An upstream typed completion refusal, when returned,
    is preserved unchanged.
- id: FR-009-AC-13
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For routed scalar native execution, the driver obtains concrete operands
    from a typed mapping of one selected `ExecutionRequest.call` to one exact FSL
    requirement occurrence and checked scalar application, with values ordered by
    the matching CG `ScalarObligationIdentity.arguments`; the original `ScalarLimits`
    come from that same call's accounting. Positional zipping, expression-node-only
    occurrence matching, and limits from another call are not valid mappings. If the
    mapping or exact binding is absent or ambiguous, it is missing retained context
    under FR-009-AC-7 and cannot produce an accepted
- id: FR-009-AC-14
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For a Kani result, the driver passes the original CG `KaniExecutionEvidence`
    and optional replay settlement to CG `run_terminal_value`; it preserves the resulting
    typed terminal without adding a driver mapping. The map receives the evidence's
    actual optional `failure_cause`; the driver invents or infers no cause.
- id: FR-009-AC-15
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: Changing or omitting optional CG diagnostics while holding the outcome,
    success-check count, failure cause, and replay settlement fixed does not change
    the terminal produced by CG `run_terminal_value`; the driver does not parse diagnostic
    content to classify or map a result.
- id: FR-009-AC-16
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For the FR-010-AC-4 vector and a proof request that reaches per-item settlement,
    the outcome holds one terminal record for each of A and C and none for B, whose
    refusal appears only in `Driven`'s per-item refusal member.
- id: FR-009-AC-17
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For an operation-frame replay selected from an admitted package, the driver
    projects FSL's exact typed `OperationSite.frame_operation` value to the CG selected
    item associated with the same admitted package identity and request index, without
    changing the pair or deriving it from CG selection or caller spelling. FSL FR-121
    owns pair shape and declaring-object resolution; FSpec FR-323 owns replay-target
    and packet/request shape. This driver criterion covers only object-type operations
    exposed by FSL's current call-site API; process/state-machine operations and state-clause
    projection are outsid
- id: FR-009-AC-18
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: When FSL call-site selection returns a typed refusal for a frame operation,
    the driver preserves that refusal unchanged and stops before CG frame-target construction,
    replay, settlement, or terminal creation; it does not recast the refusal as a
    driver join fault.
- id: FR-009-AC-19
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: When FSL call-site selection succeeds but the driver detects that its retained
    association of the FSL result to the CG selected item is missing or has a package
    identity, request index or complete occurrence that differs from the original
    run or the CG item, the driver returns FR-001's typed `runtime_invariant` fault
    and stops before CG frame-target construction, replay, settlement, or terminal
    creation. A typed refusal actually returned by CG is preserved under CG's contract
    and is not this driver-detected join fault.
- id: FR-009-AC-20
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For a falsified function item with a complete valid transcript at the byte
    and value ceilings, the driver calls CG `parse_assertion_playback` once on exactly
    the retained `KaniRunOutcome::Falsified.counterexample`, with the selected item's
    retained source identity and context and the smaller caller/proving-run ceilings
    in each dimension. It passes the resulting typed `Playback`, exact proving-run
    identity and same admitted package to CG function replay. A reproducing replay
    remains subject to FR-009-AC-2 and CG terminal mapping.
- id: FR-009-AC-21
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: A full transcript that exceeds the byte ceiling, has more values than the
    value ceiling, or has no unique valid assertion yields CG's corresponding typed
    `PlaybackRefusal` from that single parse, even if a shortened assertion block
    would fit. The driver neither uses optional diagnostics nor invokes function replay,
    creates settlement, or creates a terminal for the refused item.
- id: FR-009-AC-22
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: A typed upstream refusal before playback intake is preserved before any
    parse, settlement, or terminal creation. A parser refusal is preserved without
    replay settlement or terminal creation and is not recast as replay parity or a
    driver join fault. After a successful parse, a CG typed identity, decode, or replay
    refusal or evidence failure follows CG's `ReplaySettlement` conversion and `run_terminal_value`
    mapping with the original execution evidence; the driver preserves the resulting
    typed terminal and never settles that result as `refuted`.
- id: FR-009-AC-23
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For two function proof items sharing a checked node but naming distinct
    exact application occurrences, the driver invokes the selected provider once per
    matching generated item, retains each typed result with its own request index
    and CG `KaniObligationIdentity`, and supplies the corresponding retained source,
    dependency lock, admitted package, accounting/stage/domain/replay inputs and smaller
    caller/proving-run decode limits to function playback and replay. Neither result
    or context is selected by node alone.
- id: FR-009-AC-24
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: The driver invokes the selected provider with exactly one generated function
    item and privately retains that invocation’s request index, backend, occurrence,
    admitted package and full CG proving-run identity. A typed provider refusal is
    preserved before playback intake. A returned item count other than one, a returned
    request index or backend different from the invocation, or successful `KaniExecutionEvidence`
    whose harness path, proof ceilings, contract kind, symbolic argument bounds, solver
    or unwind differ from the selected generated harness and identity returns a driver
    `runtime_invariant`
- id: FR-009-AC-25
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: A `ToolFailed` terminal value becomes FSL's tool-failed terminal record
    with the same typed failure, byte-limit counts when present, request index, and
    backend identity.
- id: FR-009-AC-26
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: 'Before the driver calls CG''s paired proof/native runner (`execute_kani_scalar_native`)
    for a routed scalar harness, it passes the harness to CG''s `refuse_if_not_tractable`
    with `TractabilityTable::shipped_general_route_scalar()`. A harness the table
    refuses is not launched: `drive` returns `DriverRefusal::ScalarNotTractable` carrying
    CG''s `UnsupportedObligation::V2ClauseNotTractable` unchanged, with stage Replay
    and kind Refused. A harness the table does not refuse reaches the runner. A failed
    table load is the existing `Generation(Kani(TractabilityTable))` runtime-invariant
    fault.'
- id: FR-010-AC-1
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: Given a package whose abstraction relation binds a subset of requested
    items, CG negotiation receives exactly that subset and receives no unbound item.
- id: FR-010-AC-2
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: CG generation receives only items that the same admitted relation bound
    and that negotiation settled for generation.
- id: FR-010-AC-3
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: When a binding required for emission is absent, the item's FSL-defined
    typed unbound-element refusal appears unchanged in the per-item refusal member
    of the completed result, and no unbound item is sent to CG.
- id: FR-010-AC-4
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: Given an export that returns a bound result for a clause item A, a refusal
    for an extent-domain item B over an unbound population, and a bound result with
    an empty binding set for an item C that references no model element, the CG request
    carries A and C and not B, and B holds an excluded position that keeps its driver
    index (the CG change named in FR-001's Status). This is the vector FR-001-AC-17
    and FR-009-AC-16 cite.
- id: FR-010-AC-5
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: When the export refuses every requested item, the driver sends CG a request
    with zero items.
- id: FR-011-AC-1
  path: spec/functional/FR-011-cancel-a-driver-library-operation.md
  role: examined
  excerpt: The public driver library accepts one caller-owned `Cancel` token and passes
    that same token to `parse`, `select`, `check` and `package` in order.
- id: FR-011-AC-2
  path: spec/functional/FR-011-cancel-a-driver-library-operation.md
  role: examined
  excerpt: When the caller cancels the token during a FSL lifecycle stage, the public
    driver failure identifies that `SpineStage` and preserves the `CancelCause`; no
    `Driven` result or partial E4 package is returned.
- id: FR-011-AC-3
  path: spec/functional/FR-011-cancel-a-driver-library-operation.md
  role: examined
  excerpt: The cancellation result has FSL's `incomplete` category, and the binary
    obtains the process code only through FSL `Category::exit_code`; the driver adds
    no category or exit conversion.
- id: FR-011-AC-4
  path: spec/functional/FR-011-cancel-a-driver-library-operation.md
  role: examined
  excerpt: The driver does not route cancellation through the refusal-or-fault conversion
    for an unshared `Cancel` token.
- id: FR-011-AC-5
  path: spec/functional/FR-011-cancel-a-driver-library-operation.md
  role: examined
  excerpt: CLI signal and deadline triggers are outside this specification slice;
    a later frontend adapter uses the caller's `Cancel` token to request cancellation.
- id: FR-012-AC-1
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: The key preimage is JCS with operation, package identity, canonical request
    and backend identity; `generate` also has a target member.
- id: FR-012-AC-2
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: 'Retired: the manifest digest is no longer key material; the id is not
    reused.'
- id: FR-012-AC-3
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: Changing a meaning input changes the derived key.
- id: FR-012-AC-4
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: Changing top-level excluded path, time, deadline, pin, revision, application
    version or execution-backend label leaves the key unchanged; nested operation-request
    fields retain their meaning.
- id: FR-012-AC-5
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: The `target` member occurs for generate and is absent for prove and analyze.
- id: FR-012-AC-6
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: A valid present entry accepted by the requested typed reader is returned
    as a hit; absent or invalid entries are misses.
- id: FR-012-AC-7
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: ADR-029 CA-5 excluded values return `NeverStore`.
- id: FR-012-AC-8
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: A deterministic-budget incomplete result within size limits is stored and
    readable.
- id: FR-012-AC-9
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: An entry above the per-entry limit returns `NotStored`.
- id: FR-012-AC-10
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: Atomic rename from a temporary file publishes each entry; direct final-name
    writes are absent, temporary files count toward capacity, and cleanup removes
    abandoned temporary files without following symlinks.
- id: FR-012-AC-11
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: Concurrent same-key writers leave a valid typed entry, and cross-process
    eviction treats `NotFound` for an already-removed entry as success.
- id: FR-012-AC-12
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: The cache counts every regular file in its caller-named directory, evicts
    only validated owned entries and owned temporary files, preserves unrelated files,
    keeps bytes within configured limits, and uses no index or ledger.
- id: FR-012-AC-13
  path: spec/functional/FR-012-implement-the-downstream-content-addressed-cache.md
  role: examined
  excerpt: '`filament-cache` is separate from `filament-driver`, depends on its interface,
    and is absent from the driver''s transitive normal/build dependency closure.'
- id: FR-013-AC-1
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: Given a bare executable name, the host refuses to spawn it; given an explicit
    path, it launches only that executable with the supplied arguments and no shell.
- id: FR-013-AC-2
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: Before any request, the session returns a valid canonical `hello`; a rejected
    protocol identity or manifest ends the session and writes no request frame.
- id: FR-013-AC-3
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: After successful admission, a session accepts zero, one, or multiple requests
    over the same process, validates each response, and preserves each typed result
    and request identity.
- id: FR-013-AC-4
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: An invalid typed result, wrong request identity, plugin fault, or non-zero
    exit is returned as its specific failure and is not converted to cancellation
    or timeout.
- id: FR-013-AC-5
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: A plugin's valid typed cancellation settlement received within grace is
    returned unchanged; no settlement by grace expiry terminates the child and descendants
    and returns cancellation within the one-second cleanup bound.
- id: FR-013-AC-6
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: A deadline expiring while a plugin does not read a request frame returns
    `TimedOut` within the 20-millisecond polling interval plus the one-second cleanup
    bound.
- id: FR-013-AC-7
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: Truncated, malformed, non-canonical, oversized, unexpected, and wrongly
    correlated frames produce their specific typed failures without allocation above
    configured limits.
- id: FR-013-AC-9
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: Stderr beyond the session retention limit is drained but not retained;
    stderr is available on failures, and reader/writer workers are joined within one
    second after termination begins.
- id: FR-013-AC-10
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: After stdout closes while the child remains alive, process-exit waiting
    does not busy-wait; EOF before a response preserves an available child exit status.
- id: FR-013-AC-11
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: The host uses FSL ADR-029 PL-1 through PL-7 without defining a local wire
    or manifest-digest format; `filament-driver` excludes the host crate, and `fsl-cli`
    has a direct host dependency and composes it with the driver's seam.
- id: FR-013-AC-12
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: On a clean Cargo cache, the repository `make test` target fetches every
    locked platform dependency before running the offline TC-005 dependency metadata
    check, and the check passes.
- id: FR-013-AC-13
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: The host test suite reports an actionable prerequisite failure naming the
    missing fixture runtime and its installation instructions.
- id: FR-013-AC-14
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: When the IR-674 follow-up selects and implements a supported Windows process-tree
    cleanup mechanism, the host assigns the plugin process tree to a kill-on-close
    Job Object before plugin code executes; closing the owning job handle terminates
    remaining job members.
- id: FR-013-AC-15
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: When Windows process-tree setup fails before plugin execution, the host
    returns a typed host-start failure, dispatches no provider request, and terminates
    any child already created.
- id: FR-014-AC-1
  path: spec/functional/FR-014-separate-the-user-cli-package.md
  role: examined
  excerpt: Exactly one workspace package, `fsl-cli`, provides the `fsl` binary target.
- id: FR-014-AC-2
  path: spec/functional/FR-014-separate-the-user-cli-package.md
  role: examined
  excerpt: '`fsl-cli` has direct normal dependency edges to `filament-driver`, `filament-cache`,
    `filament-plugin-host` and `ix-cli-kit`; the `filament-driver` closure has no
    normal or build edge to `filament-plugin-host` or `ix-cli-kit`.'
- id: FR-014-AC-3
  path: spec/functional/FR-014-separate-the-user-cli-package.md
  role: examined
  excerpt: The workspace root lists `fsl-cli` in `default-members`.
- id: FR-014-AC-4
  path: spec/functional/FR-014-separate-the-user-cli-package.md
  role: examined
  excerpt: CLI contract and binary process-boundary test sources and their non-executable
    fixtures reside in the `fsl-cli` package; a test-only executable they launch may
    live in a separate `publish = false` workspace package.
- id: FR-014-AC-5
  path: spec/functional/FR-014-separate-the-user-cli-package.md
  role: examined
  excerpt: The CLI parser definition resides in `fsl-cli/src/cli.rs`, and the binary
    entry point is in `fsl-cli/src/main.rs`; the `filament-driver` package's `src/lib.rs`
    does not export a `cli` module.
- id: FR-014-AC-6
  path: spec/functional/FR-014-separate-the-user-cli-package.md
  role: examined
  excerpt: The `fsl run` command accepts `--provider-config <path>` as its sole process-plugin
    source, a plugins-only document whose entries carry no deadline or limit, and
    `--provider-limit <name>=<value>` as the sole source of the run's limits, including
    `deadline_ms`; omitting `--provider-config` registers no process providers, omitting
    a limit applies FSL FR-287's default (`deadline_ms` 600000), and no default, ambient
    or file configuration is read.
- id: FR-014-AC-7
  path: spec/functional/FR-014-separate-the-user-cli-package.md
  role: examined
  excerpt: Installing into an empty root gives only `bin/fsl`; the installed `fsl
    --version` and a model-free check succeed; the TC-044 provider tests still start
    the fixture.
- id: FR-015-AC-1
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. A fresh FSL plan-root package is strictly admitted
    and its independently selected root, clause, authentic occurrence, subject and
    target prepare one RT owner. Changing each selection independently, including
    another package instance with equal labels/indices, gives typed selection refusal
    before init and no owner.
- id: FR-015-AC-2
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. Layout retains the admitted root/four-entry
    content keys, lock, clock/order/progress associations, exact tick premises and
    capacities. Independent edits are either retained exactly after valid admission
    or refused at the owning selection boundary; labels, metadata PlanId, defaults
    and truncated keys never supply a match.
- id: FR-015-AC-3
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. A genuine generated closure containing shared
    expressions, lexical bindings, lazy branches, recursive calls, records, tuples,
    sequences/folds, union payloads and native comparisons lowers with original identities/order
    and one address per expression/function. Its actual admitted bodies execute; no
    handwritten temporal body substitutes.
- id: FR-015-AC-4
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. Generated source records, tuples, collections
    and unions reach the canonical TypeEnvironment with complete shapes, presence
    and original member/variant identities. Missing/foreign identity or projection
    gives the native typed refusal and no invented type or partial owner.
- id: FR-015-AC-5
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. Reachable equalities join same-acquisition
    producer witnesses against final admitted identities and adopt CheckedEquality
    without rechecking. Missing, conflicting, foreign and stale-after-rewrite rows
    refuse with exact native cause/locus; no seal/mint or fallback schedule path exists
    in the driver.
- id: FR-015-AC-6
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. A seventeen-binding plan retains the ordered
    State fields, buffers, exact event_position and admitted opaque activation-status
    type. Zero and above-host-width capacities retain their values without a local
    cap, bound interval table or status-schema restriction.
- id: FR-015-AC-7
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. A reachable admitted operation absent from
    RT gives typed unsupported lowering at that operation before init; an unrelated
    unsupported node outside the closure does not block the plan. Neither invokes
    a callback, evaluator, parser, compiler or replacement temporal algorithm.
- id: FR-015-AC-8
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. Work/storage exhaustion, cancellation, upstream
    nominal/equality refusal and RT ProgramRefusal preserve distinct typed causes/loci,
    return no partial program and leave every input unchanged except the caller budget,
    whose cumulative consumption retains every charge admitted before the stop. A
    broken lowering invariant is a driver fault rather than input refusal; expected
    operating failure is not a fault. No preparation failure becomes a temporal verdict,
    StepResourceStop or ExecutorInvariantFailure.
- id: FR-015-AC-9
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. After producer/reader preparation is dropped,
    two builds of one prepared owner have independent fresh state. Actual admitted
    entries execute with explicit StepBudget; baseline event, watermark, poll and
    gap controls preserve RT's event-position/baseline rules and exact outcomes. Preparation
    itself invokes no entry or build.
- id: FR-015-AC-10
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. Real producer/read/lower/RT execution is compared
    with FSL's independent tick reference for RT FR-276's boundary, uncertainty, buffer21/22,
    wrap/gap, changed identity/order, activation and typed-stop controls. FSpec vectors
    are read by reference. Construction diagnostics and a single pilot plan receive
    no complete FR-276 execution credit.
- id: FR-015-AC-11
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. A large selected closure amid an unrelated
    long graph visits only relevant reachable nodes/edges and identity/value material
    under the caller allowance, without a whole-package sort or repeated full indexing.
    Equal budgets give equal typed work stops without a semantic depth/size ceiling.
- id: FR-015-AC-12
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned until IR-721 lands. Public consumer/API and dependency controls
    establish one retained RT Program, no source input/callback, no driver seal/mint
    export or copied plan/expression/value family, and no IR/FSL/driver backedge from
    RT or RT edge from the cycle-free IR model.
- id: FR-015-AC-13
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: The separate clock-admission entry preserves native `WrongClause`, `WrongRole`,
    `UnknownOccurrence`, `MissingClock` and `WorkLimit` refusal payloads, including
    exact locus, occurrence, consumed work and attempted work. Semantic clock refusals
    map to outer `selection`; native work refusal maps to `work`. A missing clock
    returns no clock-admitted token even when the optional clock expectation is absent.
    Pre-cancellation returns the original cancellation cause, no clock-admitted token
    and unchanged inputs. Existing partial selection remains explicitly unclocked.
- id: FR-015-AC-14
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: A healthy clock-admission entry exposes the native `clock_for_claim` receipt
    for its independently selected final clause/Claim, charges successful receipt
    work exactly once to the remaining cumulative allowance, and compares every field
    of any optional independent clock row. A Timed model-subject receipt identifies
    the admitted timed-model root, model, source and exact unit with clock name `model-time`
    and no temporal clock row; an extra independent configured row refuses. The exposed
    inner selection retains that work prefix for later projection. Authentic Timed
    model-time receipt, extra-row r
- id: FR-015-AC-15
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: A genuine `AdmittedPlanAuthority` admits one final-subject token over its
    own `admitted_package()`, with no driver strict read, only after lock and root/entry
    key equality with the independent request, owner declaration lookup, exact root/clause/Claim,
    full source-map, native name and existing independent selection checks. Two-clause
    controls select their own final declarations; absent lookup, foreign subject/package,
    role/node/ordinal and independent request discrepancies give typed refusal. Cancellation
    returns the original cause and no token. Budget consumption is retained across
    the token'
- id: FR-016-AC-1
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: For an actual strictly admitted node, the locus retains the supplied selector
    and exactly its package-resolved structural key as distinct values, plus the original
    path and optional occurrence; its owned fields remain readable after package/input
    destruction.
- id: FR-016-AC-2
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Aggregate/application source-order children and binding child zero resolve
    their actual typed subterms. An unavailable child, nonzero binding child or continuation
    past a reference returns InvalidBodyPath; selecting the referenced original node
    starts a separate path.
- id: FR-016-AC-3
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: On a genuinely admitted monitor root, Every TargetPremise resolves the
    value of its own actual named binding, including the final Binding.value child
    zero, without comparing or changing that value. Uncertainty resolves its actual
    uncertainty value path; BufferCapacity resolves the chosen capacity value path,
    including its final child zero and original row owner.
- id: FR-016-AC-4
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: BufferCapacity without an index, an out-of-range buffer index, and an index
    supplied for another premise each return InvalidPremiseIndex, with no target payload.
    A known ordinary node used as a root returns NotMonitorRoot; an unresolved node
    returns UnknownNode.
- id: FR-016-AC-5
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Each selected actual root application at an empty path yields its entire
    original CheckedApplicationOperation and caller-resolved containing-node kind.
    Selecting a different root operation retains its different original record. Admitted
    nonempty paths select actual nonapplication terms and yield None; reference continuations
    fail rather than fabricate nested application coverage. Nested Applications are
    forbidden by the current owner admission contract. No support-policy verdict is
    inferred.
- id: FR-016-AC-6
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: An actual admitted role/ordinal is retained exactly; a nonmember occurrence
    returns OccurrenceMismatch. A genuine generated root succeeds with caller-supplied
    None even when it retains a Generated occurrence, without selecting a source-map
    row or inventing a selected occurrence.
- id: FR-016-AC-7
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Construction retains typed error distinctions and returns no partial diagnostic
    on failure. Neither constructor changes the admitted package, target values, operation
    records or source correspondence.
- id: FR-016-AC-8
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: The diagnostic module uses native IR identities/terms and native Vec child
    paths, introduces no plan/target/expression DTO or evaluator, and performs no
    parse, initialization, runtime build, support-policy decision, whole-graph index
    collection or identity recomputation.
- id: FR-016-AC-9
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: The bounded copy entry returns the existing canonical locus with both original
    node axes, native path and exact optional occurrence; after source destruction,
    shared nested Arc keys and copied text/path fields remain usable. It performs
    no resolution or validity claim.
- id: FR-016-AC-10
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: A budget whose work and storage limits equal the consumption before the
    call plus the call's measured cumulative work and storage (scratch included) succeeds,
    and the receipt's `work` and `storage` equal those measured amounts; one unit
    below either returns `Work` or `Storage` whose payload equals the budget's first
    unpaid kind, amount and `PreparationOutput` locus, with no locus copied; pre-cancellation
    returns the original cancellation cause. Nested/shared keys, UTF-8 text and long
    native paths are included; arithmetic overflow refuses before allocation.
- id: FR-016-AC-11
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Every allocating copy/scratch reservation is fallible and precharged; reservation
    failure preserves TryReserveError, excess text capacity returns Capacity before
    conversion, and cleanup drops partial output without allocating or refunding a
    charge. No allocating native clone or infallible boxed-text shrink hides behind
    a successful receipt.
- id: FR-016-AC-12
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Source inspection confirms opaque operation/native failure clones, existing
    diagnostic derivation and error boxing are outside bounded-copy coverage; no bounded
    whole-preparation claim or replacement native operation representation is introduced.
- id: FR-017-AC-1
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Full lowering consumes the actual opaque admitted selection and produces
    the existing RT Program from the complete four-entry native closure and authentic
    type/association inputs. No replacement request, callback, fake environment or
    constant-refusal preparation stub supplies the handoff.
- id: FR-017-AC-2
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: After complete success, dropping all source, package, witness, request
    and preparation objects leaves the RT-owned program usable. Two actual runtime
    builds retain independent initialized state without rebinding or a borrowed compiler
    owner.
- id: FR-017-AC-3
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Full lowering preserves admission's already consumed work under the same
    total limit, native read success work, charged storage and original cancellation.
    An exact/one-under operating stop returns its original typed cause and no partial
    owner; no guessed reader receipt or semantic-capacity change supplies success.
- id: FR-017-AC-4
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Native IR refusal, unsupported source operation, operating stop, cancellation,
    runtime ProgramRefusal and driver invariant failure remain distinct, with original
    source node/path and native payload. No failure becomes a temporal outcome or
    a partial successful program.
- id: FR-017-AC-5
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: The public lower_scalar_functions helper accepts the closed initial scalar
    set and returns owned native RT root addresses/functions/expressions charging
    the caller's one `NativeJoinBudget` from its current consumption. Requested order
    and repeated-root FunctionId sharing, exact supported operation metadata/arity/types,
    parameter ownership and unsupported/native-reader dispositions match the declared
    contract. No source parse or evaluator supplies meaning; this is component evidence
    only.
- id: FR-017-AC-6
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Within one genuine checked package, two functions sharing actual parameter
    handles retain distinct native Argument FunctionId owners and positions. Context-qualified
    memoization retains the identical original source keys without merging the owning
    invocations or rewriting keys. Native scalar outputs remain inspectable after
    setup destruction; separately lowered packages also retain their own qualified
    keys and scoped addresses. This supplies no temporal subformula-association credit.
- id: FR-017-AC-7
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: The complete monitor program retains original clause/subject/occurrence/root/buffer
    ownership. Equal numeric subformula indices under different genuine subjects do
    not alias in selection or retained Layout; no scalar-only sample substitutes for
    this control.
- id: FR-017-AC-8
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Lowering consumes native typed owner facts and emits only RT-native executable
    types, with no copied evaluator, transition implementation, second IR reader/plan
    vocabulary, hidden identity mint, whole-package index, repeated unrelated callable
    scan or semantic depth cap.
- id: FR-018-AC-1
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: Genuine ordinary source records emitted by public FSL and strictly admitted
    by IR retain all complete joined native rows and opaque record keys through driver
    admission/projection, including required, optional and Option fields and recursive/shared
    types where the owners admit them.
- id: FR-018-AC-2
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: Native structural/source/owner/member-order/presence/type mismatch and
    duplicate controls return the original native error payload and work; independent
    join exhaustion returns no admitted owner. Counterfactual native-row controls
    establish structural checks only and never producer authority.
- id: FR-018-AC-3
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: The public API accepts genuine immutable Emission authority and binds its
    rows to its own strict reader output; caller-created row structs cannot enter
    production admission. Native reader refusal/incompleteness remain distinct.
- id: FR-018-AC-4
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: Projection uses the retained package and native identities with its independent
    limit, retaining original native type and complete source_record_view without
    schema reconstruction or invented member identities.
- id: FR-018-AC-5
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: An authentic emission joins successfully at its measured native work and
    storage charge. One unit below either dimension returns no owner and preserves
    the original native exhausted-work or exhausted-storage payload. The outer preparation
    classifier maps native exhausted storage to `storage` and incomplete identity
    to `projection` without losing the native payload. This does not claim cumulative
    preparation accounting.
- id: FR-019-AC-1
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: From a genuine FSL plan root for the 20-bit 1 MHz deadline claim with a
    genuine progress view, `prepare_monitor` returns a Program whose `Layout` fields
    equal the IR layout context, the receipt bytes, FSL's order value, the observation
    owner's association value and the converted admitted premises, with the four entry
    addresses; no driver strict read occurs, and the program builds after the authority
    and request are dropped.
- id: FR-019-AC-2
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: 'Let `C(p, r)` be the cumulative consumption of resource `r` before phase
    `p` plus `p`''s first charge of `r`. For every phase and every resource it charges,
    a budget with `r` limited to `C(p, r) - 1` and the other resource at its measured
    total refuses in that phase with code `work` or `storage` and the budget''s first
    unpaid kind, amount and phase locus, no program and earlier consumption retained;
    the measured totals succeed exactly. Status: D2 is deferred; the code keeps the
    private ledger and `MonitorPreparationLimits` for selection, lowering and the
    recursion bound, and does not yet meet th'
- id: FR-019-AC-3
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: A FSL `admit_authority`, `identity_rows` or adapter budget denial, and
    a non-budget `PlanAuthorityRefusal` (missing progress, claim, selection, final
    emission), are returned unchanged inside `MonitorPreparationError` with the codes
    of the classification table.
- id: FR-019-AC-4
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: Each IR authority refusal code reaches the caller unchanged as `selection`;
    a row built from one authority but read against the package of a second admission
    of the same root bytes returns `ForeignEmission`; a `SelectedMonitorAuthorityRowError`
    returns `selection`.
- id: FR-019-AC-5
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: With defects in two phases at once, preparation returns the earlier phase's
    refusal; cancellation set before each phase returns `cancelled`, no program and
    the consumption admitted so far.
- id: FR-019-AC-6
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: A reached map collection lowers into the canonical `TypeEnvironment` with
    its key and value types; changing only FSL's `PlanId` metadata leaves `Layout`
    unchanged, while changing an entry body changes `Layout.root`.
- id: FR-019-AC-7
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: The budget's consumption is unchanged by `Program::try_new`; an RT construction
    refusal is returned as `program` with its original `ProgramRefusal`.
- id: FR-019-AC-8
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: 'Source and API inspection finds one `&mut NativeJoinBudget` parameter
    on `prepare_monitor`, `admit_monitor_final_subject`, `lower_monitor`, `lower_scalar_functions`
    and `copy_monitor_locus`; no `MonitorPreparationLimits`, private ledger, storage
    refund, driver strict read, driver-side FSL-to-IR row conversion, `PlanId` read,
    digest, compatibility path or second meter. Status: D2 is deferred; the code keeps
    the private ledger and `MonitorPreparationLimits` for selection, lowering and
    the recursion bound, and does not yet meet this criterion; the type-environment
    and layout key copies also build'
- id: FR-019-AC-9
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: Each Layout conversion succeeds on genuine admitted values, with `Layout.time_dimension`
    equal to `tick_unit_dimension()`; a request time dimension unequal to it returns
    `target_mismatch`; an unconvertible key returns `projection` with the native `InvalidNodeKey`.
- id: FR-019-AC-10
  path: spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md
  role: examined
  excerpt: Source inspection shows that disagreeing `on_event`, `on_watermark` and
    `on_gap` result types map to `internal`, with no fault-injection seam or public
    closure constructor added to produce them.
- id: FR-020-AC-1
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For a routed item whose FSL record has a conversion context and a two-guard
    path condition (one guard requiring `holds = true`, one `holds = false`), the
    `RoutedRequirement` passed to CG carries that item's occurrence (node, role and
    ordinal), the V2 node of its result bound, the conversion occurrence, and the
    two guards in FSL's order with the same `holds` values. For an item with no conversion
    context it carries `None`.
- id: FR-020-AC-2
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '`RequirementRecords` on the Kani context holds one entry per `value-validity`
    record of the package''s `requirements()`, including a record whose item the export
    refused, each with its own result bound, conversion occurrence and ordered path
    condition, and holds no entry for a record of another capability.'
- id: FR-020-AC-3
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: 'For a built unit in which one `x + 1` node''s then and else occurrences
    are guarded by `p` and `not p` and converted to `Int[0, 10]` and `Int[0, 20]`
    (`function f using v(p: Boolean, x: Int[0, 9]): Boolean pure { if p then g(x +
    1) else h(x + 1) }` with `g` over `Int[0, 10]` and `h` over `Int[0, 20]`, built
    on the RR16_UNIT shape), the two occurrences differ in ordinal, result bound and
    path condition. The two routed items carry different `RoutedRequirement` values,
    `RequirementRecords` holds two entries with different result bounds and path conditions,
    and each item''s requirement equals the en'
- id: FR-020-AC-4
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: With a `RequirementMap` that holds no `value-validity` record for the occurrence
    of an item negotiation routes to Kani, the run completes the lowering and negotiation
    steps and then refuses, when the generation items are built, with `DriverFault::RequirementRecordAbsent`,
    naming the request index and occurrence, with stage Generation, wire code `runtime_invariant`
    and outcome kind internal fault; CG generation is not called and the result carries
    no settlement built from a substitute record.
- id: FR-020-AC-5
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: Given a `RequirementMap` built with `FromIterator` that holds, at one occurrence,
    both a `value-validity` record and an `operation-contract` record with a different
    result bound, `RequirementRecords` holds exactly the `value-validity` record's
    fields at that key, the `value-validity` item's requirement equals them, and no
    fault is raised. A lookup by occurrence alone finds the key ambiguous and returns
    no record, so that lookup raises `RequirementRecordAbsent` here.
- id: FR-020-AC-6
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For the FR-010-AC-4 vector, the projection returns a requirement for A
    (index 0) and C (index 2) and `None` at B's index 1, B's position is excluded,
    `RequirementRecords` still holds B's record, the run completes without a `DriverRefusal`,
    and the same holds with B's record removed from the map. FR-001-AC-17 keeps its
    own evidence in TC-017; this criterion asserts only the projection and the absence
    of a fault.
- id: FR-020-AC-7
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For an item with two path guards and a conversion context, the request
    positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard
    in outermost-first order, all inside the item's context span, and the next item's
    driver index is unchanged by them.
- id: FR-020-AC-8
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: Given a record whose FSL result-bound node differs from the type the IR
    result context selects, the driver passes the FSL node unchanged; CG refuses with
    `RequirementRecord` and `ResultBoundMismatch`, and the driver returns it unchanged
    in `DriverRefusal::Generation` with stage Generation, wire code `runtime_invariant`
    and kind internal fault.
- id: FR-020-AC-9
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For each of the eight `RequirementRecordMismatch` variants CG defines (`RequirementMissing`,
    `OccurrenceNotSelected`, `RecordMissing`, `ApplicationMismatch`, `ResultBoundMismatch`,
    `ConversionContextMismatch`, `PathConditionMismatch`, `GuardContextMissing`),
    a CG refusal carrying it is returned as `DriverRefusal::Generation` with the request
    index, occurrence and mismatch unchanged, stage Generation, wire code `runtime_invariant`
    and outcome kind internal fault.
- id: FR-020-AC-10
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '~~Retired by IR-304 slice (b): the interim `guard_lowering_not_implemented`
    settlement of slice (a) no longer exists. Its replacement outcomes are FR-020-AC-11.~~'
- id: FR-020-AC-11
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: 'Over the IR-304 guarded package, each item with a lowerable guard (`if
    y > 5 then y + 1 else y + 1`: a then item and an else item) generates a harness
    with one guard `kani::assume` that follows the symbolic inputs, precedes the oracle
    call, and is negated for the else item, and its identity''s `requirement_occurrence`
    equals the item''s occurrence; an item whose guard CG cannot lower (`if z + 1
    > 5 then z + 1 else z + 1`) settles `unsupported` with `path_guard_not_lowered`
    and reason `unsupported_guard_form`, and no harness omits a guard.'
- id: FR-020-AC-12
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For `if p then x + 1 else x + 1`, whose then and else items are both guarded
    by the node of `p`, the positions hold a `ContextRoot` for `p` in the then item's
    span and another in the else item's span, and the root is not emitted once for
    both.
- id: FR-020-AC-13
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: When CG finds a path guard of a routed item without a `ContextRoot` in
    that item's context span, its refusal `RequirementRecord` with `GuardContextMissing`
    is returned as `DriverRefusal::Generation` with its request index, occurrence
    and guard unchanged, stage Generation, wire code `runtime_invariant` and outcome
    kind internal fault; end to end, real CG given an item whose requirement and record
    name a guard rooted only in another item's span refuses the whole call with it.
- id: FR-020-AC-14
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: When a `value-validity` record of the package has a guard occurrence whose
    node the admitted package does not hold, the run refuses with `DriverFault::RequirementRecordUnresolved`
    naming the record's FSL occurrence key with cause `UnknownNode`; when a record's
    key has a role outside IR's occurrence vocabulary, the cause is `UnknownOccurrenceRole`.
    Each has stage Generation, wire code `runtime_invariant` and outcome kind internal
    fault, is raised even when no item routes to that record, and CG generation is
    not called. The `AmbiguousNode` cause is produced by the same node lookup FR-001
    uses an
- id: FR-020-AC-15
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For a bound item of another capability (a model-reading `operation-contract`
    item) whose occurrence has no `value-validity` record in the map, negotiation
    settles it `InvalidRequest` as before, it is absent from the generation items,
    the projection returns `None` at its index and adds no guard root for it, no `RequirementRecordAbsent`
    fault is raised at any step, and a `value-validity` item in the same request still
    receives its requirement.
- id: FR-021-AC-1
  path: spec/functional/FR-021-gate-the-driver-on-the-examples-suite.md
  role: examined
  excerpt: With the workspace-built `fsl`, the runner's `check` over the fetched suite
    exits 0 and reports a non-zero number of running cases passed and 0 failed.
- id: FR-021-AC-2
  path: spec/functional/FR-021-gate-the-driver-on-the-examples-suite.md
  role: examined
  excerpt: With the same binary, the runner's `staged` over the fetched suite exits
    0 and reports a non-zero number of staged cases passed and 0 failed.
- id: FR-021-AC-3
  path: spec/functional/FR-021-gate-the-driver-on-the-examples-suite.md
  role: examined
  excerpt: In a copy of one system whose running case has one edited recorded outcome
    field, `check` exits 1 and its failure line names the case as `<system>/<case>`.
- id: FR-021-AC-4
  path: spec/functional/FR-021-gate-the-driver-on-the-examples-suite.md
  role: examined
  excerpt: In a copy of one system whose staged case now gives its ready exit, `staged`
    exits 1 and prints `promote <system>/<case>` for that case.
- id: FR-021-AC-5
  path: spec/functional/FR-021-gate-the-driver-on-the-examples-suite.md
  role: examined
  excerpt: An unset `FSL_EXAMPLES_DIR`, or one naming a folder without `examples/`
    and `tools/fsl-examples`, is an error naming the cause; the runner given a root
    with no `examples/` exits 2 rather than passing zero cases.
- id: FR-021-AC-6
  path: spec/functional/FR-021-gate-the-driver-on-the-examples-suite.md
  role: examined
  excerpt: The binary the gate runs prints, for `fsl --version`, the version of this
    workspace's `fsl-cli` package and lies in this workspace's target directory; `make
    ci` runs `examples-check`.
- id: FR-021-AC-7
  path: spec/functional/FR-021-gate-the-driver-on-the-examples-suite.md
  role: examined
  excerpt: Without `FSL_LANGUAGE_DIR`, the `fsl` build reports language packages from
    the filament-spec-language git source and no path package outside this workspace;
    with it, the build reports no package from that git source and at least one from
    the named checkout, the workspace copy's `Cargo.lock` differs from the checkout's,
    the gate prints `fsl built from language checkout <path>`, and in both modes the
    checkout's `Cargo.lock` is byte for byte what it was before the build.
- id: FR-022-AC-1
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: '`fsl analyze` over the Counter request with the item `{"claim": "Bounded"}`
    and `--format json` writes one document with `operation` `analyze`, category `success`,
    one item whose record is `proved`, and `aggregate.status` `complete`, and exits
    0.'
- id: FR-022-AC-2
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'The item `{"claim": "BelowThree"}` gives an item whose record is `refuted`,
    with a `counterexample` entry and `replay` `{"target": {"state_clause": Q}, "reproduced":
    true}`, `Q` ending in `BelowThree`, category `violation`, and exit 10.'
- id: FR-022-AC-3
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'A request whose item sets `temporal.limits.max_states` to 1 gives an `incomplete`
    record whose cause names `max_states`, with no `counterexample`, and exit 22;
    a request with `deadline_ms` 0 gives an `incomplete` record whose cause is `{"kind":
    "cancelled", "source": "deadline"}`, and exit 22.'
- id: FR-022-AC-4
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'An item naming a backend the registry does not hold settles with category
    `refusal` and the record diagnostic `invalid_capability`/`unknown-backend` naming
    the backend, exit 20; an item over the Counter fixture naming backend `en-6`,
    which lists no temporal-satisfaction form, settles `unsupported` with `unsupported_projection`/`unsupported-requested-capability`,
    exit 21 (the item kind is incidental: EN-1 serves derived deadlock-freedom since
    QSL-2122, and a state-graph claim over a timed subject stays unsupported until
    QSL-2305); neither writes empty output.'
- id: FR-022-AC-5
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'A malformed `native-analyze/1` document gives, in machine mode, a document
    with `operation` `analyze`, category `refusal`, empty `items` and one `invalid-request`
    diagnostic, and in text mode one stderr line starting `fsl analyze: invalid-request:`;
    both exit 20. A program the checker refuses gives a document whose `last_stage`
    is the checker stage FSL names (`S3`), empty `items` and that refusal''s code,
    and the text line `fsl analyze: <code>: `; a snapshot naming a population the
    model does not declare gives the binding refusal''s code with empty `items`; each
    exits 20. A missing operand, an u'
- id: FR-022-AC-6
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: The machine-mode stdout for the AC-2 request is byte-equal to the canonical
    encoding of the document `fsl_command::command::analyze` returns in process for
    the same request, followed by one newline.
- id: FR-022-AC-7
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: In text mode the AC-2 request prints `0 BelowThree refuted` at the start
    of its first line, `violation` as its last line, and exits 10; an item naming
    an unregistered backend prints `0 Bounded invalid-request invalid_capability/unknown-backend`.
- id: FR-022-AC-8
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: Each settlement row of the table holds for the driver's negotiation over
    a candidate outcome of that shape, except the several-candidates-with-a-named-backend
    row, which FSL's candidate computation does not produce; the `registry` member
    of an outcome lists the engines FSL registers (`en-1` among them), and the `Bounded`
    item's candidates are `["en-1"]`.
- id: FR-022-AC-9
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: With `--provider-config` starting a process provider whose `hello` advertises
    (`value-validity`, `unbounded`), an item naming its backend is sent to it as one
    FR-331 request at the item's request index and settles `proved` with certification
    `trusted` when the provider answers `proved`, and `inconclusive` when it answers
    anything else; the document's `registry` lists the provider and `en-1`; another
    item is unchanged by it. A provider advertising only `bounded`, an unreadable
    configuration and a malformed one each refuse with exit 20.
- id: FR-022-AC-10
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'SIGTERM or SIGINT while an item runs gives a document whose open and later
    items are `incomplete` with cause `{"kind": "cancelled", "source": "requested"}`,
    and exit 22.'
- id: FR-022-AC-11
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'With no `--provider-config`, `fsl analyze` over FSL''s `examples/hybrid-braking/slow-enough.analyze.json`
    with `--format json` writes one item settled `proved` with certification `certified`
    and backend `fsl-hybrid-reference`, category `success`, exit 0, and over `too-slow.analyze.json`
    one item settled `refuted` with backend `fsl-hybrid-reference`, category `violation`,
    exit 10; neither writes `HybridProviderAbsent`, and the document''s `registry`
    lists `fsl-hybrid-reference` and `en-1`. The process-provider half depends on
    FSL''s `Engine::hybrid_process` (FSL FR-750): with `--provider-config` s'
- id: FR-022-AC-12
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: With no `--provider-config`, `fsl analyze` over a timed Counter subject
    with one item named `TimedLiveness`, whose clause is the unbounded `always (holds(c.value
    = 0) implies eventually holds(c.value = 0))` on `model-time`, writes one item
    settled `proved` by `en-6`; the record includes FR-632's `non_zeno` member, the
    document category is `success`, and the process exits 0.
- id: FR-023-AC-1
  path: spec/functional/FR-023-supply-clock-configurations-with-clocks.md
  role: examined
  excerpt: '`fsl run <request> --clocks <file>` on a unit with a timestamped-event
    clause on clock `plant-ms` exits with the outcome and writes the document the
    same request carrying the file''s object as its `clocks` member gives.'
- id: FR-023-AC-2
  path: spec/functional/FR-023-supply-clock-configurations-with-clocks.md
  role: examined
  excerpt: '`fsl compile <request> --clocks <file>` on that unit writes the document
    the request carrying the member gives, and the same request with no clocks refuses
    `missing-clock-selection`.'
- id: FR-023-AC-3
  path: spec/functional/FR-023-supply-clock-configurations-with-clocks.md
  role: examined
  excerpt: An unreadable `--clocks` file refuses `io-error`, cause `clocks-file-unreadable`,
    stage null, exit 20 on `run` and `compile`.
- id: FR-023-AC-4
  path: spec/functional/FR-023-supply-clock-configurations-with-clocks.md
  role: examined
  excerpt: A `--clocks` file that is an array, not JSON, or has trailing content refuses
    `invalid-request`, cause `clocks-file-not-an-object`, naming `--clocks`, stage
    null.
- id: FR-023-AC-5
  path: spec/functional/FR-023-supply-clock-configurations-with-clocks.md
  role: examined
  excerpt: A request carrying `clocks` with `--clocks` refuses `invalid-request` (FSL's
    refusal, FR-640), naming `clocks` and `--clocks`, exit 20, with the outcome's
    `last_stage` null (FR-286, FR-287-AC-11), on `run` and `compile`.
- id: FR-023-AC-6
  path: spec/functional/FR-023-supply-clock-configurations-with-clocks.md
  role: examined
  excerpt: A member FSL refuses (unknown `kind`) in the file refuses with the code,
    cause, stage and message the same object gives as the request's `clocks` member,
    and a repeated member name in the file reaches FSL and is refused naming the clock.
- id: FR-023-AC-7
  path: spec/functional/FR-023-supply-clock-configurations-with-clocks.md
  role: examined
  excerpt: 'A `--clocks` file over the request byte limit refuses `resource_exhausted`,
    cause `clocks-file-too-large`, stage null, exit 22, naming `--clocks` and the
    limit: `--request-bytes` on `run`, 1,048,576 bytes on `compile`.'
- id: FR-024-AC-1
  path: spec/functional/FR-024-write-the-checked-package-with-output.md
  role: examined
  excerpt: '`fsl compile <request> --output <file>` over a request without `output`
    exits 0, writes a file whose bytes equal those `fsl_command::command::compile`
    returns and those the request carrying the same path as its `output` member writes,
    resolves a relative path against the working directory, and gives an outcome with
    `artifacts` `[]` equal to the member''s.'
- id: FR-024-AC-2
  path: spec/functional/FR-024-write-the-checked-package-with-output.md
  role: examined
  excerpt: '`--output` naming an existing file refuses `invalid-request`, exit 20,
    `last_stage` null, naming the path, and the file''s bytes are unchanged.'
- id: FR-024-AC-3
  path: spec/functional/FR-024-write-the-checked-package-with-output.md
  role: examined
  excerpt: A request carrying `output` with `--output` refuses `invalid-request` (FSL's
    refusal, FR-810), exit 20, naming `output` and `--output`, `last_stage` null,
    and neither file is written.
- id: FR-024-AC-4
  path: spec/functional/FR-024-write-the-checked-package-with-output.md
  role: examined
  excerpt: '`--output` on `run`, `check` and `analyze` is refused as an unknown option,
    exit 20, with nothing on stdout and no file written.'
- id: FR-024-AC-5
  path: spec/functional/FR-024-write-the-checked-package-with-output.md
  role: examined
  excerpt: '`--output` given twice, or without a path, refuses the command line naming
    `--output`, exit 20, and writes no file.'
- id: FR-024-AC-6
  path: spec/functional/FR-024-write-the-checked-package-with-output.md
  role: examined
  excerpt: A compile that the checker refuses with `--output` exits 20 with the checker's
    code and writes no file.
- id: FR-025-AC-1
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: '`fsl monitor` over `Reaches` and `Never` with levels 0, 1, 2, 1 then `{"end":
    true}` writes event lines then one summary line and exits 10, with a `settled`
    `violation` event for `Never` at record 2; over `Never` alone with levels 0, 1,
    0 it exits 0.'
- id: FR-025-AC-2
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: The lines the binary writes equal, byte for byte, the canonical encodings
    of the events and summary `command::monitor` hands back in process for the same
    request and records, and the exit code equals the summary's.
- id: FR-025-AC-3
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: A request naming a clause the program lacks, a request that is not JSON,
    and a missing operand each exit 20 before any record is read; the first writes
    only the `monitor` command-error document.
- id: FR-025-AC-4
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: '`fsl monitor monitor.json --provider-config providers.json --input records.jsonl
    --format json`, over FSL FR-644-AC-8''s request (the fixture unit adding `function
    Even using v(n: Integer): Boolean pure { n * 2 - n = n }`, `clauses` `Reaches`
    and `Never`, `items` `[{"claim": "Even", "kind": "value-validity", "backend":
    "fsl.test-recording-provider"}]`) with levels 0, 1, 2, 1, 0 and a configuration
    that starts `fsl-test-provider` (advertising `value-validity` `unbounded`, answering
    `proved`), starts the plugin, which receives a request; writes the event lines
    of the same command over the request'
- id: FR-025-AC-5
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: A record with an unknown member between good records gives one `refusal`
    event naming that record, the summary counts it, the stream continues and the
    output equals `command::monitor`'s.
- id: FR-025-AC-6
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: With standard input held open, the event that settles `Never` is on standard
    output before the next record is sent or the input is closed.
- id: FR-025-AC-7
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: '`--input <file>` gives the same bytes and exit code as the same records
    on standard input; an unopenable `--input` file refuses with exit 20, and in machine
    mode stdout holds the `monitor` command-error document, in text mode stdout is
    empty.'
- id: FR-025-AC-8
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: In text mode, over AC-1's fixture and records, stdout holds one event line
    `<record> <clause> <category> settled\
- id: FR-025-AC-9
  path: spec/functional/FR-025-stream-observations-with-fsl-monitor.md
  role: examined
  excerpt: 'SIGTERM or SIGINT while standard input is still open ends the run with
    no further record read: the last line written is an outcome document with category
    `incomplete`, and the process exits 22.'
- id: FR-026-AC-1
  path: spec/functional/FR-026-write-the-replay-results-with-fsl-replay.md
  role: examined
  excerpt: '`fsl replay <request> --format json` over the fixture writes stdout byte-equal
    to the canonical encoding of the outcome document `fsl_command::command::run`
    returns in process for the same request, followed by one newline: operation `replay`,
    category `success`, and `result` `{"kind": "replayed", "results": [R]}` with `R`''s
    `arm` `input` and `settlement` `reproduced-without-witness`; exit 0.'
- id: FR-026-AC-2
  path: spec/functional/FR-026-write-the-replay-results-with-fsl-replay.md
  role: examined
  excerpt: 'The same request without `--format` writes stdout byte-equal to `render_outcome`''s
    text of AC-1''s document, whose lines are `replay: success (exit 0) at S8` and
    `result 0: input reproduced-without-witness`; exit 0.'
- id: FR-026-AC-3
  path: spec/functional/FR-026-write-the-replay-results-with-fsl-replay.md
  role: examined
  excerpt: The fixture request with `clocks` added writes in machine mode an outcome
    document with operation `replay`, category `refusal`, `result` `null` and an `invalid-request`
    diagnostic, and in text mode `render_outcome`'s text of that document, which holds
    no `result` line; both exit 20.
- id: FR-026-AC-4
  path: spec/functional/FR-026-write-the-replay-results-with-fsl-replay.md
  role: examined
  excerpt: 'A `native-run/1` request handed to `fsl replay` writes in text mode `render_outcome`''s
    text of the refusal document, a `replay: refusal (exit 20)` header and no `result`
    line, and exits 20, with no replay run.'
- id: FR-027-AC-1
  path: spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md
  role: examined
  excerpt: '`fsl export <request> --output m.tla --format json` over FR-756-AC-1''s
    counter request (`inc`, `value <= 5`) exits 0, writes `m.tla` whose bytes equal
    the document''s `result.export.target_text`, and writes FSL''s outcome document
    with operation `export`, category `success` and `result.kind` `exported`; the
    request carrying `export.output` naming `m.tla`, run with no option in a fresh
    directory, writes a byte-identical file.'
- id: FR-027-AC-2
  path: spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md
  role: examined
  excerpt: '`--output` naming an existing file refuses `invalid-request`, exit 20,
    `last_stage` null, naming the path, and the file''s bytes are unchanged; a request
    carrying `export.output` with `--output` refuses naming `output` and `--output`,
    exit 20, and writes neither file; `--output` given twice refuses naming `--output`,
    exit 20.'
- id: FR-027-AC-3
  path: spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md
  role: examined
  excerpt: '`fsl export <request> --output m.tla --limit export.output_bytes=1` exits
    22, names `export.output_bytes` and writes no file; with `--limit export.output_bytes=16777216`
    it exits 0.'
- id: FR-027-AC-4
  path: spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md
  role: examined
  excerpt: '`fsl export <request> --output m.tla` with no `--format` writes exactly
    the four lines `export: success (exit 0) at S4`, `target: tla filament.output.tla2/v1`,
    `bytes written: <n>` and `output: m.tla`, and exits as in machine mode.'
- id: FR-027-AC-5
  path: spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md
  role: examined
  excerpt: '`fsl export --evidence e.json --format json` writes FSL''s evidence outcome
    document with operation `export`; `fsl export --evidence e.json --output m.tla`
    and `fsl export <request> --evidence e.json` each refuse `invalid-request` naming
    `--evidence`, exit 20, with nothing written to the output path; `fsl compile <request>
    --evidence e.json` refuses as an unknown option, exit 20.'
bindings:
- test_id: fsl-cli/tests/cli_contract.rs::run_routes_requirement_items_with_caller_spine_limits
  ac_id: FR-002-AC-22
  trace: correct
- test_id: fsl-cli/tests/cli_contract.rs::run_refuses_a_configured_provider_with_the_reference_backend_name
  ac_id: FR-002-AC-22
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::analyze_text_mode_prints_item_lines
  ac_id: FR-022-AC-7
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::composition_text_preserves_derived_outcomes_trust_basis_and_extents_from_json
  ac_id: FR-022-AC-7
  trace: correct
- test_id: fsl-cli/src/backend_provider.rs::tests::hybrid_hello_retains_dynamics_and_solver_bounds_through_fsl_admission
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/src/backend_provider.rs::tests::hybrid_hello_refuses_malformed_or_unrecognized_dynamics
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/src/backend_provider.rs::tests::hybrid_hello_preserves_missing_fairness_and_solver_bound_refusals
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/src/backend_provider.rs::tests::hybrid_hello_invalid_fairness_labels_retain_owner_refusals
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_braking_default_proof_and_refutation_have_real_evidence
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_selection_transports_fsl_request_and_keeps_trusted_proof
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_preserves_malformed_results_for_fsl_refusal
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_provider_unsuitable_hello_is_refused_and_started_sessions_are_closed
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_deadline_settles_incomplete_instead_of_accepting_a_late_proof
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_signal_cancels_the_open_item
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_refutation_requires_exact_native_replay
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_refuses_nontrusted_plugin_certification
  ac_id: FR-022-AC-11
  trace: correct
```

## Final narrow disposition (second and final reviewer pass)

Reviewed exact latest head `9d57266af4f24c79f6f447e977a1854cf7cfe043` against initial reviewed head `f3711b7ee4408261ba29f0d69357fb791d31f1ee`. The only changed file is fsl-cli/tests/frontend_cli.rs: 187 insertions and 14 deletions. No production source, manifest, pin, original Reservoir oracle or CI change occurred. This section supersedes the initial review's pending-check statements; initial findings and evidence are preserved as history.

The composition fixture now supplies two distinct selected model subjects with two components, the existing driver-owned Counter schema/clauses instantiated for the meter subject, and one complete joint initialization world containing both populations. Both library scopes map both populations. This preserves the real native model route and required minimum-two-component composition grammar; no upstream native program is copied. The transformations consume existing driver fixture data, with no second checked-in semantic fixture definition. All previous three-item JSON/text outcomes, full proof basis, certification, per-premise extent, capability, backend and exit-21 assertions remain.

The former string rfind assertion is replaced by structural native counterexample inspection. It checks the declared population identity, package/node identity, reference type, object identity, original object type, record identity and integer value field. It ties the deciding meter reference to over_binding, asserts actual native replay reproduced=true, checks the full deciding trajectory [1,2,3], independently checks the peer Counter trajectory [0,0,0], and verifies both final values. No assertion is hidden behind a possibly unexecuted guard. This strengthens the CEX oracle and prevents a peer object's value from being mistaken for the deciding subject.

Actual remote evidence supplied by ROOT for this exact head: Titan simple223-9d57-c1 rc=0, selected=1, passed=1, failed=0, ignored=0; simple223-9d57-c2 focused frontend Clippy rc=0. ROOT reports the complete native JSON/text/certification/basis/extents/exit-21 and typed identity trajectory assertions executed successfully. The prior AA71 batch supplies the other 25 passing runtime checks, two passing production builds and three passing focused lint checks; all intervening edits are confined to this frontend test fixture. This is exact-head focused evidence plus preserved unchanged-production evidence, not a claim that the whole runtime batch reran at the latest head. No Cargo, runtime, repeat matrix, baseline or additional review method was run in this final pass.

ARCH explicitly accepts the inherited driver 317-criterion baseline conditionally for this bounded PR223/PR224 change with no new gap. The whole-repository audit remains FAIL: 175 untagged and five ignored-only criteria. The initial exact base/head comparison found no newly missing criterion and two improvements to tagged. No spec or test trace declarations changed in the final fixture diff, so there is no new traceability delta to recompute. The five documented-comment host trace tags remain an explicit active-module extraction limitation; no engine-backed PASS or historical gap repair is claimed.

Merge eligibility: source review and required focused actual evidence permit merge at `9d57266af4f24c79f6f447e977a1854cf7cfe043` under that bounded ARCH decision. QSL-2234's broader combined-provider acceptance is not declared complete. The lead/coder must retain the latest two SR artifacts; PR223 must use a merge commit to preserve stacked child PR224.

Tracker hold: Peter/ARCH forbids Terra Linear queries, comments and state calls until approximately 11:39 PT. This final pass made none. The initial private comments were previously posted and read back; the final disposition update is prepared for ROOT to post once after the hold, with read-back then. No final tracker write/read-back is claimed here.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-002 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-003 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-004 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-005 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-006 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-007 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-008 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-009 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-010 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-011 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-012 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-013 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-014 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-015 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-016 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-017 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-018 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-019 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-020 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-021 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-022 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-023 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-024 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-025 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-026 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-027 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-028 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-029 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-030 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-031 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-032 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-033 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-034 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-035 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-036 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-037 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-038 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-039 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-040 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-041 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-042 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-043 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-044 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-045 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-046 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-047 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-048 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-049 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-050 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-051 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-052 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-053 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-054 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-055 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-056 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-057 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-058 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-059 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-060 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-061 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-062 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-063 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-064 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-065 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-066 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-067 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-068 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-069 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-070 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-071 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-072 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-073 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-074 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-075 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-076 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-077 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-078 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-079 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-080 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-081 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-082 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-083 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-084 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-085 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-086 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-087 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-088 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-089 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-090 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-091 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-092 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-093 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-094 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-095 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-096 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-097 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-098 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-099 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-100 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-101 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-102 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-103 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-104 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-105 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-106 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-107 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-108 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-109 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-110 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-111 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-112 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-113 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-114 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-115 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-116 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-117 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-118 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-119 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-120 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-121 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-122 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-123 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-124 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-125 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-126 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-127 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-128 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-129 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-130 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-131 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-132 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-133 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-134 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-135 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-136 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-137 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-138 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-139 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-140 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-141 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-142 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-143 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-144 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-145 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-146 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-147 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-148 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-149 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-150 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-151 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-152 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-153 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-154 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-155 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-156 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-157 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-158 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-159 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-160 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-161 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-162 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-163 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-164 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-165 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-166 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-167 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-168 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-169 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-170 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-171 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-172 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-173 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-174 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-175 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-176 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-177 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-178 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-179 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |
| FND-180 | accepted-no-change | ARCH explicitly accepts the inherited driver baseline for bounded PR223/PR224 with no new gap; exact initial base/head comparison confirms no newly missing criterion, and final fixture-only diff changes no trace declarations. Whole audit remains FAIL. |

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Timed analysis through the installed command-line boundary.

use ix_trace_rs::trace;
use serde_json::{json, Value};

use super::AnalyzeRequest;

/// Arm at global time one, resetting only `x`. Completion uses elapsed
/// time `x`, while the unreset `y` enforces the global deadline. A completed
/// timer can keep stepping with divergent time. Clock values come from the
/// timed model, rather than data fields in the initial snapshot.
fn timer_request(name: &str, delay: u32, zeno: bool) -> AnalyzeRequest {
    let mut model: Value =
        serde_json::from_str(include_str!("../fixtures/analyze-counter.semantic-ir.json"))
            .expect("the Counter model is JSON");
    let object = &mut model["types"][1];
    for clock in ["x", "y"] {
        let identity = format!("ix://test/counter/Counter/{clock}");
        object["fields"]
            .as_array_mut()
            .expect("fields")
            .push(json!({
                "identity": identity, "name": clock,
                "typeRef": "ix://filament/native/Clock",
                "presence": "required", "nullable": false, "defaultKind": "none",
                "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": false},
                "origin": {"generated": {"generatorIdentity": identity,
                    "generatorVersion": "1.0.0", "inputIdentities": [identity]}}
            }));
    }
    let body = if zeno {
        object["operations"][0]["frame"]["modifies"] = json!(["ix://test/counter/Counter/x"]);
        "post Reset using v on M::Counter::inc { self.x = 0 ms }\n\
         time invariant Frozen when { true } { self.x <= 0 ms }\n"
            .to_owned()
    } else {
        object["operations"][0]["frame"]["modifies"] = json!([
            "ix://test/counter/Counter/value",
            "ix://test/counter/Counter/x"
        ]);
        for name in ["finish", "idle"] {
            let identity = format!("ix://test/counter/Counter/{name}");
            object["operations"]
                .as_array_mut()
                .expect("operations")
                .push(json!({
                    "identity": identity, "name": name, "params": [], "pre": [], "post": [],
                    "origin": {"generated": {"generatorIdentity": identity,
                        "generatorVersion": "1.0.0", "inputIdentities": [identity]}},
                    "frame": {"modifies": ["ix://test/counter/Counter/value"],
                        "creates": [], "deletes": []}
                }));
        }
        let remaining = delay.checked_sub(1).expect("a positive completion delay");
        format!(
            "pre Arm using v on M::Counter::inc {{ self.value = 0 and self.y >= 1 ms }}\n\
             post Armed using v on M::Counter::inc {{ self.value = 1 and self.x = 0 ms }}\n\
             pre Ready using v on M::Counter::finish {{ \
                 self.value = 1 and self.x >= {remaining} ms }}\n\
             post Finished using v on M::Counter::finish {{ self.value = 2 }}\n\
             pre Idle using v on M::Counter::idle {{ self.value = 2 }}\n\
             post Stable using v on M::Counter::idle {{ self.value = 2 }}\n\
             time invariant BeforeArm when {{ self.value = 0 }} {{ self.y <= 1 ms }}\n\
             time invariant Pending when {{ self.value = 1 }} {{ self.y <= {delay} ms }}\n\
             temporal Deadline using tt over (c: M::Counter) clock \"model-time\" on origin {{\n\
                 eventually[0 ms, 3 ms] holds(c.value = 2)\n\
             }}\n"
        )
    };
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"filament.model.complete/v1\";\n\
         profile t = \"filament.temporal.complete/v1\";\n\
         profile tt = \"filament.temporal.timed/v1\";\n\
         model M = \"test/counter\";\n\
         dimension Time;\nunit s : Time = rational(1, 1);\n\
         unit ms : Time = rational(1, 1000) * s;\n\
         time M dense unit ms;\n{body}"
    );
    let items = if zeno {
        json!([{"claim": "M", "temporal": {"item_kind": "time-lock-freedom"}}])
    } else {
        json!([{"claim": "Deadline"}])
    };
    AnalyzeRequest::new(name, items, json!({}))
        .with_file(
            "/request/program/source/file",
            "timer.fsl",
            source.as_bytes(),
        )
        .with_file(
            "/request/models/0/source/file",
            "timer.semantic-ir.json",
            model.to_string().as_bytes(),
        )
}

fn timed_liveness_request() -> AnalyzeRequest {
    let source = "\
        language \"ix:native\" edition \"1-draft\";\n\
        profile v = \"filament.model.complete/v1\";\n\
        profile t = \"filament.temporal.complete/v1\";\n\
        profile tt = \"filament.temporal.timed/v1\";\n\
        model M = \"test/counter\";\n\
        pre CanInc using v on M::Counter::inc { self.value < 3 }\n\
        post Inc using v on M::Counter::inc { self.value - pre(self.value) = 1 }\n\
        dimension Time;\n\
        unit s : Time = rational(1, 1);\n\
        unit ms : Time = rational(1, 1000) * s;\n\
        time M dense unit ms;\n\
        temporal TimedLiveness using tt over (c: M::Counter) clock \"model-time\" on origin {\n\
            always (holds(c.value = 0) implies eventually holds(c.value = 0))\n\
        }\n";
    AnalyzeRequest::new(
        "timed-liveness",
        json!([{"claim": "TimedLiveness"}]),
        json!({}),
    )
    .with_file(
        "/request/program/source/file",
        "timed-liveness.fsl",
        source.as_bytes(),
    )
}

/// An unbounded liveness item over a dense-time subject is decided by EN-6
/// through the real `fsl analyze` process and records the non-Zeno premise.
#[trace("TC-036", "FR-022-AC-12")]
#[test]
fn analyze_timed_liveness_reports_non_zeno() {
    let (document, exit) = timed_liveness_request().json();
    assert_eq!(exit, Some(0), "{document}");
    assert_eq!(document["operation"], "analyze");
    assert_eq!(document["category"], "success");
    assert_eq!(document["items"].as_array().map(Vec::len), Some(1));
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "proved", "{item}");
    assert_eq!(item["record"]["backend"], "en-6", "{item}");
    assert!(
        ["not-required", "assumed", "checked", "refuted"]
            .contains(&item["record"]["non_zeno"].as_str().unwrap_or("")),
        "a timed-liveness record carries its `non_zeno` premise: {item}"
    );
}

/// A two-clock timer is proved to finish within its three-millisecond bound.
#[trace("TC-036", "FR-022-AC-1", "TC-1592", "FR-632-AC-1")]
#[test]
fn analyze_zone_deadline_holds() {
    let (document, exit) = timer_request("zone-deadline-holds", 3, false).json();
    assert_eq!(exit, Some(0), "{document}");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "proved", "{item}");
    assert_eq!(item["record"]["backend"], "en-6", "{item}");
    assert_eq!(item["record"]["basis"]["type"], "zone-certified", "{item}");
    assert_eq!(item["record"]["certification"], "certified", "{item}");
    assert!(item.get("counterexample").is_none(), "{item}");
}

/// Delaying the same timer to four milliseconds produces an exact timed
/// counterexample that the native evaluator reproduces.
#[trace("TC-036", "FR-022-AC-2", "TC-1592", "FR-632-AC-1")]
#[test]
fn analyze_zone_deadline_refuted() {
    let (document, exit) = timer_request("zone-deadline-refuted", 4, false).json();
    assert_eq!(exit, Some(10), "{document}");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "refuted", "{item}");
    assert_eq!(item["record"]["backend"], "en-6", "{item}");
    assert_eq!(item["replay"]["reproduced"], true, "{item}");
    let steps = item["counterexample"]["prefix"]
        .as_array()
        .expect("a concretised timed prefix");
    assert!(
        steps.len() >= 2,
        "arming and late completion must be witnessed: {item}"
    );
    assert_eq!(
        steps[0]["delay"],
        json!({"numerator": "1", "denominator": "1"}),
        "{item}"
    );
    assert_eq!(
        steps[1]["delay"],
        json!({"numerator": "3", "denominator": "1"}),
        "{item}"
    );
}

/// Collapsing the global clock onto the reset clock admits a completion at
/// time four, so the two-clock holding oracle must change its verdict.
#[trace("TC-036", "FR-022-AC-2", "TC-1592", "FR-632-AC-1")]
#[test]
fn analyze_zone_deadline_distinguishes_reset_and_global_clocks() {
    let request = timer_request("zone-clock-distinction", 3, false);
    let source = std::fs::read_to_string(request.directory.0.join("timer.fsl"))
        .expect("the generated timer source");
    let collapsed = source.replace("self.y", "self.x");
    let (document, exit) = request
        .with_file(
            "/request/program/source/file",
            "timer.fsl",
            collapsed.as_bytes(),
        )
        .json();
    assert_eq!(exit, Some(10), "{document}");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "refuted", "{item}");
    assert_eq!(item["replay"]["reproduced"], true, "{item}");
}

/// A reset loop under a zero clock invariant cannot advance time and is
/// reported as a replayed time lock, rather than as a liveness witness.
#[trace("TC-036", "FR-022-AC-2", "TC-1586", "FR-626-AC-3")]
#[test]
fn analyze_zeno_refuted() {
    let (document, exit) = timer_request("zone-zeno", 0, true).json();
    assert_eq!(exit, Some(10), "{document}");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "refuted", "{item}");
    assert_eq!(item["record"]["backend"], "en-6", "{item}");
    assert_eq!(item["counterexample"]["kind"], "time-lock", "{item}");
    assert_eq!(item["replay"]["reproduced"], true, "{item}");
    assert_eq!(
        item["counterexample"]["final_delay"],
        json!({"numerator": "0", "denominator": "1"}),
        "{item}"
    );
}

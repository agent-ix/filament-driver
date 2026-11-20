use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::{json, Value};

mod semantics;

// Original two-step workflow over the authoritative same-repository Counter
// input model. The model is referenced in place, never copied into this crate.
const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile values = "filament.value.complete/v1";
profile paths = "filament.temporal.infinite-trace/v1";
model Queue = "test/counter";
pre SpaceAvailable using values on Queue::Counter::inc { self.value < 2 }
post AppendOne using values on Queue::Counter::inc { self.value - pre(self.value) = 1 }
temporal WithinCapacity using paths over (q: Queue::Counter) clock "model-steps" on origin {
  always holds(q.value <= 2)
}
temporal NeverFull using paths over (q: Queue::Counter) clock "model-steps" on origin {
  always holds(q.value < 2)
}
"#;

fn setup() -> Result<(CliCase, Value), ContractError> {
    let case = CliCase::from_environment()?;
    case.write("data/workflow.fsl", SOURCE.as_bytes())?;
    case.write("data/start.json", &serde_json::to_vec(&json!({
        "format":"filament.state.snapshot/v1", "identity":{"authority":"contracts","identity":"queue-start"},
        "observation":"current", "model":{"identity":"test/counter"},
        "anchor":{"kind":"initialization","name":"queue-start"},
        "populations":[{"population":"ix://test/counter/counters","complete":true,
            "objects":[{"key":"queue-east","type":"ix://test/counter/Counter","fields":{"value":{"integer":"0"}}}]}]
    }))?)?;
    let model = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fsl-cli/tests/fixtures/analyze-counter.semantic-ir.json");
    let request = json!({"format":"native-analyze/1","request":{
        "program":{"source":{"file":"workflow.fsl","authority":"contracts","identity":"queue-workflow","document":"QueueWorkflow","formal_revision":1}},
        "models":[{"format":"semantic-ir/2.0.0","source":{"file":model,"authority":"contracts","identity":"counter-reference","document":"CounterInput","formal_revision":1}}],
        "documents":[{"file":"start.json"}],
        "items":[{"claim":"WithinCapacity"},{"claim":"NeverFull"}]
    }});
    write_request(&case, &request)?;
    Ok((case, request))
}

fn write_request(case: &CliCase, request: &Value) -> Result<(), ContractError> {
    case.write("data/request.json", &serde_json::to_vec(request)?)
}

fn document(output: &Captured, exit: i32, category: &str) -> Value {
    assert_eq!(output.0.status.code(), Some(exit), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    let value = output.json().expect("one JSON analyze document");
    assert_eq!(value["format"], "filament-outcome/1");
    assert_eq!(value["operation"], "analyze");
    assert_eq!(value["category"], category, "{value}");
    value
}

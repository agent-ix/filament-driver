// SPDX-License-Identifier: AGPL-3.0-or-later
//! Defensive clock-row rejection over an original admitted source identity.

use super::*;
use filament_contract_model::CheckedPackageReadLimits;
use fsl_foundation::SourceIdentity;
use fsl_replay::spine::{check, parse, select, DependencyInput, ParseRequest, SpineLimits};

/// FR-017-AC-4 component: a clock-shaped member refuses as Unsupported with
/// its original node and path before a runtime field is constructed. This is
/// a defensive input control, with no clock witness admission or full failure
/// classification claim.
#[ix_trace_rs::trace("FR-017-AC-4")]
#[test]
fn clock_record_member_refuses_without_a_substitute_runtime_type() {
    let source = SourceIdentity::new("agent-ix", "clock-row-refusal");
    let text = "language \"ix:native\" edition \"1-draft\";\n\
        profile v = \"filament.value.complete/v1\";\n\
        record Reading { count: Integer; }\n\
        function read using v(r: Reading): Integer pure { r.count }\n";
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let packages = std::collections::BTreeMap::new();
    let parsed = parse(
        &ParseRequest {
            source: &source,
            path: "clock-row.native",
            bytes: text.as_bytes(),
        },
        limits.source,
        &cancel,
    )
    .unwrap()
    .into_value();
    let models = select(&parsed, &packages, limits.model, &cancel)
        .unwrap()
        .into_value();
    let checked = check(
        &parsed,
        "clock-row-refusal".to_owned(),
        &models,
        &DependencyInput::default(),
        limits,
        &cancel,
    )
    .unwrap()
    .into_value();
    let emission = fsl_package::emit_checked(checked.package(), limits.package.emit()).unwrap();
    let admitted = crate::monitor::source_record::admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        MonitorPreparationLimits {
            work: 100_000,
            storage: 1_000_000,
        },
    )
    .unwrap();
    let row = admitted
        .rows()
        .find(|row| row.qualified_declaration == [Box::<str>::from("Reading")])
        .unwrap();
    assert_eq!(row.source_authority.as_ref(), "agent-ix");
    assert_eq!(row.source_identity.as_ref(), "clock-row-refusal");
    let fields = source_record_fields(admitted.package(), row).unwrap();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].value_type(), &ValueType::Integer);
    let original = row.structural_node.clone();
    let mut clock_row = row.clone();
    clock_row.members[0].type_node = None;
    clock_row.members[0].value_type = None;
    clock_row.members[0].member_type = Some("ix://filament/native/Clock".into());
    let error = match source_record_fields(admitted.package(), &clock_row) {
        Ok(_) => panic!("a clock entry has no runtime value representation"),
        Err(error) => error,
    };
    let Error::Unsupported(refusal) = error else {
        panic!("clock entries require the typed unsupported payload");
    };
    assert_eq!(refusal.locus.node, original);
    assert_eq!(
        &refusal.locus.structural_key,
        admitted.package().structural_key(&original).unwrap()
    );
    assert!(refusal.locus.body_path.is_empty());
    assert!(clock_row.members[0].value_type.is_none());
    assert_eq!(
        clock_row.members[0].member_type.as_deref(),
        Some("ix://filament/native/Clock")
    );
}

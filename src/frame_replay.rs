// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retained request to FSL-selected frame replay preparation (FR-009).

use std::collections::BTreeMap;

use filament_contract_codegen::{
    FrameReplay, FrameReplayError, FrameReplayInputs, SelectedFrameTarget,
};
use filament_contract_model::{ClockConfiguration, ContractSourceV1, TemporalBinding};
use fsl_foundation::SourceIdentity;
use fsl_replay::spine::{DependencyInput, DependencyInputRefusal, RequestClocks, SuppliedLibrary};

use crate::{Driven, DriverFault, DriverRefusal};

/// The original owner-specific failure of frame replay preparation.
#[derive(Debug, thiserror::Error)]
pub enum FrameReplayFailure {
    /// FSL refused the supplied dependency selection before call-site selection.
    #[error(transparent)]
    Dependencies(DependencyInputRefusal),
    /// FSL refused call-site selection; the complete typed payload is unchanged.
    #[error(transparent)]
    CallSite(Box<fsl_replay::CallSiteRefusal>),
    /// The selected frame is not associated with the retained driver run.
    #[error(transparent)]
    Driver(Box<DriverRefusal>),
    /// CG refused preparation; the complete typed payload is unchanged.
    #[error(transparent)]
    Codegen(FrameReplayError),
}

impl Driven {
    /// Prepares one operation-frame replay using FSL's exact declaring-owner target.
    ///
    /// `inputs.selected_target` must be `None`: the driver supplies that value from
    /// FSL. The selected item must name this run's admitted package and an original
    /// request's complete occurrence. Public request inventory edits do not change
    /// that retained association. The source, domains and dependencies must reproduce
    /// this run's emitted package. No request item, settlement or terminal is created.
    ///
    /// The returned CG preparation exposes its packet/request and may be replayed
    /// through `FrameReplay::replay` under CG's refusal and settlement contracts.
    ///
    /// # Errors
    /// Returns the unchanged FSL or CG failure, or a driver `runtime_invariant`
    /// fault for missing, prepopulated or mismatched selection associations.
    pub fn prepare_frame_replay(
        &self,
        mut inputs: FrameReplayInputs<'_>,
    ) -> Result<FrameReplay, FrameReplayFailure> {
        let dependencies = DependencyInput::new(
            inputs
                .run
                .dependencies
                .iter()
                .map(|dependency| SuppliedLibrary {
                    identity: dependency.identity.clone(),
                    source: SourceIdentity::new(
                        &dependency.source.authority,
                        &dependency.source.identity,
                    ),
                    path: dependency.source.identity.clone(),
                    bytes: dependency.source.bytes.clone(),
                })
                .collect::<Vec<_>>(),
        )
        .map_err(FrameReplayFailure::Dependencies)?;
        let clocks = request_clocks(self.checked().temporal_bindings())?;
        let located = fsl_replay::call_site(
            SourceIdentity::new(&inputs.run.source.authority, &inputs.run.source.identity),
            inputs.run.package_name.clone(),
            &inputs.run.source.identity,
            &inputs.run.source.bytes,
            inputs
                .packages
                .iter()
                .map(|document| document.bytes.as_slice()),
            &dependencies,
            &clocks,
            &inputs.operation,
        )
        .map_err(FrameReplayFailure::CallSite)?;
        inputs.run.clocks.program = replay_program_clocks(&clocks);
        let mismatch = || {
            FrameReplayFailure::Driver(Box::new(DriverRefusal::Fault(
                DriverFault::FrameReplayAssociation,
            )))
        };
        let selected = &inputs.selected_item;
        let original = self
            .replay_item(selected.request_index)
            .ok_or_else(mismatch)?;
        if inputs.selected_target.is_some()
            || located.package != self.emitted().bytes()
            || selected.package != self.checked()
            || original.index().get() != selected.request_index
            || original.occurrence() != selected.occurrence
            || located.site.frame_occurrence != *selected.occurrence
        {
            return Err(mismatch());
        }
        inputs.selected_target = Some(SelectedFrameTarget {
            operation: located.site.frame_operation,
            package_id: ContractSourceV1 {
                name: self.checked().name().into(),
                lock: self.checked().lock().clone(),
            },
            request_index: selected.request_index,
            occurrence: located.site.frame_occurrence,
        });
        FrameReplay::new(inputs).map_err(FrameReplayFailure::Codegen)
    }
}

/// Stable ownership classification for a frame preparation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameReplayFailureCode {
    /// FSL dependency admission refused.
    Dependencies,
    /// FSL operation selection refused.
    CallSite,
    /// The driver detected a broken retained association.
    RuntimeInvariant,
    /// CG frame preparation refused.
    Codegen,
}

impl FrameReplayFailureCode {
    /// The stable boundary code; native details remain on `FrameReplayFailure`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dependencies => "frame_dependencies",
            Self::CallSite => "frame_call_site",
            Self::RuntimeInvariant => "runtime_invariant",
            Self::Codegen => "frame_codegen",
        }
    }

    /// Every stable frame preparation boundary code.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Dependencies,
            Self::CallSite,
            Self::RuntimeInvariant,
            Self::Codegen,
        ]
    }

    /// Parses a stable boundary code.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

impl FrameReplayFailure {
    /// The stable ownership code, without interpreting any native message.
    pub fn code(&self) -> FrameReplayFailureCode {
        match self {
            Self::Dependencies(_) => FrameReplayFailureCode::Dependencies,
            Self::CallSite(_) => FrameReplayFailureCode::CallSite,
            Self::Driver(_) => FrameReplayFailureCode::RuntimeInvariant,
            Self::Codegen(_) => FrameReplayFailureCode::Codegen,
        }
    }
}

/// The FR-640 clock selections the run compiled under: the admitted package's own
/// `temporal_bindings` rows, read as FSL's replay reads them (model binding names
/// are derived, not members; the first row of a clock name wins).
///
/// `rows` is IR's typed accessor over a package its strict reader already
/// admitted, so `None` is an absent member (a unit without clock selections), not
/// an unreadable one; an unreadable array never reaches this point. A row that is
/// not a well formed selection (empty name or unit, non-positive period) is the
/// `FrameReplayAssociation` fault.
///
/// Known limit (QSL-2999): the package lock's `dependency_clock_selections` are
/// not carried. FSL's `RequestClocks::for_replay`, which keeps program and
/// dependency clocks apart, is `pub(crate)`; `RequestClocks::new` serves both
/// namespaces from one map. The driver adopts `for_replay` once FSL exposes it.
fn request_clocks(rows: Option<&[TemporalBinding]>) -> Result<RequestClocks, FrameReplayFailure> {
    let mut members = BTreeMap::new();
    for row in rows.unwrap_or_default() {
        if fsl_package::is_model_binding_name(&row.clock_name) {
            continue;
        }
        members
            .entry(row.clock_name.to_string())
            .or_insert_with(|| row.clock.clone());
    }
    RequestClocks::new(members).map_err(|_| {
        FrameReplayFailure::Driver(Box::new(DriverRefusal::Fault(
            DriverFault::FrameReplayAssociation,
        )))
    })
}

fn replay_program_clocks(clocks: &RequestClocks) -> BTreeMap<String, ClockConfiguration> {
    clocks
        .iter()
        .map(|(name, configuration)| (name.to_owned(), configuration.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use filament_contract_model::CheckedNodeId;
    use filament_exact::{NodeKey, PathSegment};
    use ix_trace_rs::trace;

    use super::*;

    fn row(name: &str, unit: &str) -> TemporalBinding {
        let key = NodeKey::new(
            "pkg".to_owned(),
            vec![PathSegment::Text("claim".to_owned())],
            0,
        )
        .expect("a fixture key is admissible");
        let declaration: CheckedNodeId = serde_json::from_value(
            fsl_foundation::node_wire::node_key_to_json(&key).expect("the key encodes"),
        )
        .expect("a checked node identity");
        serde_json::from_value(serde_json::json!({
            "declaration": serde_json::to_value(&declaration).expect("serializes"),
            "occurrence": 0,
            "clock_name": name,
            "clock": {"kind": "timestamped_event", "timestamp_unit": unit},
        }))
        .expect("a temporal binding row")
    }

    fn configured(unit: &str) -> ClockConfiguration {
        ClockConfiguration::TimestampedEvent {
            timestamp_unit: unit.into(),
        }
    }

    #[test]
    #[trace("FR-009-AC-17", "TC-016")]
    fn replay_inputs_preserve_the_selected_program_clocks() {
        let clocks = RequestClocks::new(BTreeMap::from([("events".to_owned(), configured("ms"))]))
            .expect("well-formed clock selection");

        assert_eq!(
            replay_program_clocks(&clocks),
            BTreeMap::from([("events".to_owned(), configured("ms"))])
        );
    }

    /// A real clock row becomes a named member, not the default selection.
    #[test]
    fn a_real_clock_row_becomes_a_request_clock_member() {
        let rows = [row("events", "ms")];
        let clocks = request_clocks(Some(&rows)).expect("well formed rows");
        assert_eq!(clocks.get("events"), Some(&configured("ms")));
        assert_ne!(clocks, RequestClocks::default());
    }

    /// A model binding row is derived, not a member; the first row of a name wins.
    #[test]
    fn model_binding_rows_are_skipped_and_the_first_row_of_a_name_wins() {
        let rows = [
            row("model-time", "ms"),
            row("events", "ms"),
            row("events", "s"),
        ];
        let clocks = request_clocks(Some(&rows)).expect("well formed rows");
        assert!(clocks.get("model-time").is_none());
        assert_eq!(clocks.get("events"), Some(&configured("ms")));
        assert_eq!(clocks.iter().count(), 1);
    }

    /// A package without the member has no clock selections.
    #[test]
    fn an_absent_member_selects_no_clocks() {
        assert!(request_clocks(None).expect("no rows").is_empty());
    }

    /// A malformed row is the association fault, never silently dropped.
    #[test]
    fn a_malformed_row_is_the_frame_replay_association_fault() {
        for rows in [[row("", "ms")], [row("events", "")]] {
            let Err(FrameReplayFailure::Driver(refusal)) = request_clocks(Some(&rows)) else {
                panic!("a malformed row must refuse");
            };
            assert!(matches!(
                *refusal,
                DriverRefusal::Fault(DriverFault::FrameReplayAssociation)
            ));
        }
    }
}

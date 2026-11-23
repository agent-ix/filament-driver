// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retained request to FSL-selected frame replay preparation (FR-009).

use std::collections::BTreeMap;

use filament_contract_codegen::{
    FrameReplay, FrameReplayError, FrameReplayInputs, SelectedFrameTarget,
};
use filament_contract_model::ContractSourceV1;
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
    /// The FR-640 clock selections this run compiled under: the admitted package's
    /// own `temporal_bindings` rows, read as FSL's replay reads them (model binding
    /// names are derived, not members; the first row of a clock name wins). `None`
    /// when a row is not a well formed selection.
    fn request_clocks(&self) -> Option<RequestClocks> {
        let mut members = BTreeMap::new();
        for row in self.checked().temporal_bindings().unwrap_or_default() {
            if fsl_package::is_model_binding_name(&row.clock_name) {
                continue;
            }
            members
                .entry(row.clock_name.to_string())
                .or_insert_with(|| row.clock.clone());
        }
        RequestClocks::new(members).ok()
    }

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
        let clocks = self.request_clocks().ok_or_else(|| {
            FrameReplayFailure::Driver(Box::new(DriverRefusal::Fault(
                DriverFault::FrameReplayAssociation,
            )))
        })?;
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

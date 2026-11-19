// SPDX-License-Identifier: AGPL-3.0-or-later
//! Resolve and apply FR-003's caller-selected FSL limit settings through FSL.

use fsl_foundation::{Setting, SettingLimits, SettingName, UsageRefusal};
use fsl_replay::{
    AccountingLimits, CallerLimits, ObservationLimits,
    spine::{Call, ClauseRunRequest, SpineLimits},
};

/// A recognized caller-selected setting unavailable to the selected operation.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum LimitError {
    /// FSL recognizes the setting, but this operation does not expose it.
    #[error("limit setting `{}` is unavailable for the `{operation}` operation", setting.name())]
    OperationUnavailable {
        /// The recognized setting absent from the operation's bounds.
        setting: SettingName,
        /// The operation that refused the setting.
        operation: &'static str,
    },
}

/// Named, non-negative settings supplied with repeatable `--limit` options.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LimitOverrides(Vec<(SettingName, u64)>);

impl LimitOverrides {
    /// Parse operands with FSL's canonical setting grammar.
    pub fn parse(operands: &[String]) -> Result<Self, UsageRefusal> {
        fsl_foundation::setting::parse_operands(operands.iter().map(String::as_str)).map(Self)
    }

    /// Apply settings exposed by the selected `run` operation.
    pub fn apply_to(&self, limits: &mut SpineLimits, call: &mut Call) -> Result<(), LimitError> {
        self.apply_parts(limits, None, &mut call.accounting, "run")
    }

    /// Apply stage settings exposed by a source operation.
    pub fn apply_to_spine(
        &self,
        limits: &mut SpineLimits,
        operation: &'static str,
    ) -> Result<(), LimitError> {
        let mut caller_limits = CallerLimits {
            spine: *limits,
            ..CallerLimits::default()
        };
        for (name, value) in &self.0 {
            let SettingName::Stage(setting) = name else {
                return Err(LimitError::OperationUnavailable {
                    setting: *name,
                    operation,
                });
            };
            if !caller_limits.set_bound(*setting, *value) {
                return Err(LimitError::OperationUnavailable {
                    setting: *name,
                    operation,
                });
            }
        }
        *limits = caller_limits.spine;
        Ok(())
    }

    /// Apply each supplied setting to the first matching value in `limits`.
    pub fn apply_to_setting_group(
        &self,
        limits: &mut [&mut dyn SettingLimits],
        operation: &'static str,
    ) -> Result<(), LimitError> {
        let mut updates = Vec::with_capacity(self.0.len());
        for (name, value) in &self.0 {
            let SettingName::Stage(setting) = name else {
                return Err(LimitError::OperationUnavailable {
                    setting: *name,
                    operation,
                });
            };
            let Some((index, previous)) = limits.iter().enumerate().find_map(|(index, target)| {
                target
                    .bounds()
                    .into_iter()
                    .find(|(candidate, _)| candidate == setting)
                    .map(|(_, previous)| (index, previous))
            }) else {
                return Err(LimitError::OperationUnavailable {
                    setting: *name,
                    operation,
                });
            };
            updates.push((index, *setting, previous, *value));
        }

        let mut applied: Vec<(usize, Setting, u64)> = Vec::with_capacity(updates.len());
        for (index, setting, previous, value) in updates {
            if !limits[index].set_bound(setting, value) {
                for (index, setting, previous) in applied.into_iter().rev() {
                    let restored = limits[index].set_bound(setting, previous);
                    debug_assert!(restored, "a previously applied bound must be restorable");
                }
                return Err(LimitError::OperationUnavailable {
                    setting: SettingName::Stage(setting),
                    operation,
                });
            }
            applied.push((index, setting, previous));
        }
        Ok(())
    }

    /// Apply stage settings to the bounds an operation accepts.
    ///
    /// Accounting settings require a call and are unavailable to front-end
    /// operations such as parse and check.
    pub fn apply_to_limits<T: SettingLimits>(
        &self,
        limits: &mut T,
        operation: &'static str,
    ) -> Result<(), LimitError> {
        let available = limits.bounds();
        let mut updates = Vec::with_capacity(self.0.len());
        for (name, value) in &self.0 {
            match name {
                SettingName::Stage(setting)
                    if available.iter().any(|(available, _)| available == setting) =>
                {
                    updates.push((*setting, *value));
                }
                SettingName::Stage(setting) => {
                    return Err(LimitError::OperationUnavailable {
                        setting: SettingName::Stage(*setting),
                        operation,
                    });
                }
                SettingName::Accounting(_) => {
                    return Err(LimitError::OperationUnavailable {
                        setting: *name,
                        operation,
                    });
                }
            }
        }
        for (setting, value) in updates {
            // `bounds()` and `set_bound()` are the paired FSL API for an
            // operation's exposed settings.
            let updated = limits.set_bound(setting, value);
            debug_assert!(updated, "FSL bound listed by bounds() must be settable");
            if !updated {
                return Err(LimitError::OperationUnavailable {
                    setting: SettingName::Stage(setting),
                    operation,
                });
            }
        }
        Ok(())
    }

    /// Apply compile-stage and caller-supplied export ceilings for `fsl export`.
    /// Export limits are optional in FSL's bounds view, so this applies them
    /// directly while routing the remaining settings through the shared API.
    pub fn apply_to_export(&self, limits: &mut SpineLimits) -> Result<(), LimitError> {
        let mut export = limits.export;
        let mut compile = Vec::new();
        for (name, value) in &self.0 {
            match name {
                SettingName::Stage(Setting::ExportOutputBytes) => {
                    export.output_bytes = Some(*value);
                }
                SettingName::Stage(Setting::ExportNodes) => {
                    export.nodes = Some(*value);
                }
                SettingName::Stage(Setting::ExportWorkUnits) => {
                    export.work_units = Some(*value);
                }
                _ => compile.push((*name, *value)),
            }
        }
        Self(compile).apply_to_limits(limits, "export")?;
        limits.export = export;
        Ok(())
    }

    /// Apply settings to the actual stage, observation and accounting limits of a clause run.
    pub fn apply_to_clause(&self, request: &mut ClauseRunRequest) -> Result<(), LimitError> {
        self.apply_parts(
            &mut request.limits,
            Some(&mut request.observation_limits),
            &mut request.accounting,
            "run",
        )
    }

    fn apply_parts(
        &self,
        limits: &mut SpineLimits,
        mut observation: Option<&mut ObservationLimits>,
        accounting: &mut filament_exact::ScalarLimits,
        operation: &'static str,
    ) -> Result<(), LimitError> {
        let mut available: Vec<Setting> = limits
            .bounds()
            .into_iter()
            .map(|(setting, _)| setting)
            .collect();
        if let Some(observation) = observation.as_deref() {
            available.extend(observation.bounds().into_iter().map(|(setting, _)| setting));
        }
        let mut updated_accounting = AccountingLimits(*accounting);
        let mut caller_limits = CallerLimits {
            spine: *limits,
            ..CallerLimits::default()
        };
        if let Some(observation) = observation.as_deref() {
            caller_limits.observation = *observation;
        }
        for (setting, value) in &self.0 {
            match setting {
                SettingName::Stage(setting) => {
                    if !available.contains(setting) {
                        return Err(LimitError::OperationUnavailable {
                            setting: SettingName::Stage(*setting),
                            operation,
                        });
                    }
                    // `bounds()` and `set_bound()` are the paired FSL API for
                    // the selected operation; an exposed setting must be writable.
                    let updated = caller_limits.set_bound(*setting, *value);
                    debug_assert!(updated, "FSL bound listed by bounds() must be settable");
                    if !updated {
                        return Err(LimitError::OperationUnavailable {
                            setting: SettingName::Stage(*setting),
                            operation,
                        });
                    }
                }
                SettingName::Accounting(setting) => {
                    if operation != "run" {
                        return Err(LimitError::OperationUnavailable {
                            setting: SettingName::Accounting(*setting),
                            operation,
                        });
                    }
                    updated_accounting.set_bound(*setting, *value);
                }
            }
        }
        *limits = caller_limits.spine;
        if let Some(observation) = observation.as_mut() {
            **observation = caller_limits.observation;
        }
        *accounting = updated_accounting.0;
        Ok(())
    }

    /// The parsed FSL settings, for caller inspection.
    pub fn values(&self) -> &[(SettingName, u64)] {
        &self.0
    }
}

impl LimitError {
    /// Format the refusal for the selected operation.
    pub fn message_for(&self, operation: &str) -> String {
        match self {
            Self::OperationUnavailable { setting, .. } => format!(
                "limit setting `{}` is unavailable for the `{operation}` operation",
                setting.name()
            ),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use fsl_foundation::{
        AccountingSetting, Setting, SettingLimits, SettingName, UsageCause, UsageRefusal,
    };
    use fsl_replay::AccountingLimits;
    use fsl_replay::spine::{Call, SpineLimits, default_accounting};

    use super::{LimitError, LimitOverrides};

    fn call() -> Call {
        Call {
            function: "run".to_owned(),
            arguments: Vec::new(),
            wire_arguments: Vec::new(),
            snapshot: None,
            accounting: default_accounting(100),
            supplied: filament_semantic_value::supplied_stop::SuppliedLimits::default(),
            backend: Default::default(),
            clocks: Default::default(),
            live_occurrences: fsl_replay::spine::DEFAULT_LIVE_OCCURRENCES,
        }
    }

    fn clause_request() -> fsl_replay::spine::ClauseRunRequest {
        fsl_replay::spine::ClauseRunRequest {
            source: fsl_replay::spine::ClauseRunSource::Program {
                identity: fsl_foundation::SourceIdentity::new("issuer", "unit"),
                path: "unit.fsl".to_owned(),
                bytes: Vec::new(),
            },
            package_name: "unit".to_owned(),
            packages: Default::default(),
            dependencies: Default::default(),
            snapshots: Default::default(),
            invocations: Default::default(),
            selection: fsl_replay::spine::ClauseRunSelection::Function {
                name: "yes".to_owned(),
                arguments: Vec::new(),
                snapshot: fsl_semantics::model::observation::DocumentRef {
                    authority: "issuer".to_owned(),
                    identity: "snapshot".to_owned(),
                },
                snapshot_path: None,
            },
            limits: SpineLimits::default(),
            observation_limits: fsl_replay::ObservationLimits::default(),
            accounting: default_accounting(17),
            clocks: Default::default(),
        }
    }

    #[ix_trace_rs::trace("TC-010", "FR-003-AC-4")]
    #[test]
    fn clause_limits_apply_to_the_actual_observation_and_accounting_bounds() {
        let mut request = clause_request();
        let original = request.observation_limits;
        let (setting, value) = original.bounds()[0];
        let value = value.saturating_sub(1);
        let limits = LimitOverrides::parse(&[
            format!("{}={value}", setting.name()),
            "accounting.work_units=3".to_owned(),
        ])
        .expect("parse owner setting names");
        limits
            .apply_to_clause(&mut request)
            .expect("clause exposes observation and accounting");
        assert_eq!(
            request
                .observation_limits
                .bounds()
                .into_iter()
                .find(|(name, _)| *name == setting),
            Some((setting, value))
        );
        assert_eq!(
            AccountingLimits(request.accounting)
                .bounds()
                .into_iter()
                .find(|(name, _)| *name == fsl_foundation::AccountingSetting::WorkUnits)
                .map(|(_, value)| value),
            Some(3)
        );
        assert_eq!(request.limits, SpineLimits::default());
    }

    #[ix_trace_rs::trace("TC-010", "FR-003-AC-5")]
    #[test]
    fn invalid_clause_limit_is_atomic_across_all_actual_bounds() {
        let mut request = clause_request();
        let original_spine = request.limits;
        let original_observation = request.observation_limits;
        let original_accounting = request.accounting;
        let limits = LimitOverrides::parse(&[
            "s1.input_bytes=1".to_owned(),
            "accounting.work_units=0".to_owned(),
            "explore.states=1".to_owned(),
        ])
        .expect("all settings are recognized");
        assert!(limits.apply_to_clause(&mut request).is_err());
        assert_eq!(request.limits, original_spine);
        assert_eq!(request.observation_limits, original_observation);
        assert_eq!(request.accounting, original_accounting);
    }

    /// Trace: FR-003-AC-1, FR-003-AC-4, FR-003-AC-6.
    #[test]
    fn one_limit_override_preserves_other_defaults() {
        let mut before = SpineLimits::default();
        let prior_tokens = 17_u64;
        assert!(SettingLimits::set_bound(
            &mut before,
            Setting::S1Tokens,
            prior_tokens
        ));
        let mut limits = before;
        let mut call = call();
        let before_accounting = AccountingLimits(call.accounting).bounds();
        let override_ = LimitOverrides::parse(&["s1.input_bytes=5".to_owned()])
            .expect("FSL parses its setting operand");
        assert_eq!(
            override_.values(),
            &[(SettingName::Stage(Setting::S1InputBytes), 5)]
        );
        override_
            .apply_to(&mut limits, &mut call)
            .expect("setting is exposed");
        assert_eq!(limits.source.source_bytes, 5);
        assert_eq!(
            AccountingLimits(call.accounting).bounds(),
            before_accounting
        );
        assert_eq!(limits.source.tokens, before.source.tokens);
        assert_eq!(limits.checking, before.checking);
        assert_eq!(limits.model, before.model);
    }

    /// Trace: FR-003-AC-5, applying a later unavailable setting is atomic.
    #[test]
    fn unavailable_setting_leaves_all_prior_bounds_unchanged() {
        let before = SpineLimits::default();
        let mut limits = before;
        let mut call = call();
        let before_accounting = AccountingLimits(call.accounting).bounds();
        let overrides = LimitOverrides::parse(&[
            "s1.input_bytes=5".to_owned(),
            "explore.states=11".to_owned(),
        ])
        .expect("both setting names are recognized by FSL");

        assert_eq!(
            overrides.apply_to(&mut limits, &mut call),
            Err(LimitError::OperationUnavailable {
                setting: SettingName::Stage(Setting::ExploreStates),
                operation: "run",
            })
        );
        assert_eq!(limits, before);
        assert_eq!(
            AccountingLimits(call.accounting).bounds(),
            before_accounting
        );
    }

    /// Trace: FR-003-AC-5, a later unavailable group setting is atomic.
    #[ix_trace_rs::trace("FR-003-AC-5")]
    #[test]
    fn unavailable_group_setting_leaves_prior_bounds_unchanged() {
        let before = SpineLimits::default();
        let mut limits = before;
        let overrides = LimitOverrides::parse(&[
            "s1.input_bytes=5".to_owned(),
            "observation.input_bytes=7".to_owned(),
        ])
        .expect("both settings are recognized by FSL");
        let mut groups: [&mut dyn SettingLimits; 1] = [&mut limits];

        assert_eq!(
            overrides.apply_to_setting_group(&mut groups, "check"),
            Err(LimitError::OperationUnavailable {
                setting: SettingName::Stage(Setting::ObservationInputBytes),
                operation: "check",
            })
        );
        assert_eq!(limits, before);
    }

    /// Trace: FR-003-AC-1, FR-003-AC-4, FR-003-AC-6.
    #[test]
    fn every_exposed_setting_updates_only_its_fsl_bound() {
        let defaults = SpineLimits::default();
        let default_bounds = defaults.bounds();
        for (setting, original) in default_bounds.iter().copied() {
            let requested = if original < u64::MAX {
                original + 1
            } else {
                original - 1
            };
            let operand = format!("{}={requested}", setting.name());
            let overrides = LimitOverrides::parse(&[operand]).expect("FSL parses exposed name");
            let mut effective = defaults;
            let mut call = call();
            overrides
                .apply_to(&mut effective, &mut call)
                .expect("FSL exposes setting to run");
            for (observed, bound) in effective.bounds() {
                assert_eq!(
                    bound,
                    if observed == setting {
                        requested
                    } else {
                        default_bounds
                            .iter()
                            .copied()
                            .find(|(default_setting, _)| *default_setting == observed)
                            .map(|(_, default_bound)| default_bound)
                            .expect("default has the same exposed setting")
                    },
                    "only {setting} should change"
                );
            }
        }
    }

    /// Trace: FR-003-AC-2, FR-003-AC-7, FR-003-AC-8.
    #[test]
    fn fsl_parser_refuses_unknown_malformed_and_duplicate_operands() {
        assert!(matches!(
            LimitOverrides::parse(&["normalization.work_units=1".to_owned()]),
            Err(UsageRefusal {
                cause: UsageCause::UnknownSetting,
                ..
            })
        ));
        assert!(matches!(
            LimitOverrides::parse(&["s1.nodes=ten".to_owned()]),
            Err(UsageRefusal {
                cause: UsageCause::NotAnInteger,
                ..
            })
        ));
        assert!(matches!(
            LimitOverrides::parse(&["s1.nodes=1".to_owned(), "s1.nodes=2".to_owned()]),
            Err(UsageRefusal {
                cause: UsageCause::Repeated,
                ..
            })
        ));
    }

    /// Trace: FR-003-AC-5, recognized settings outside run bounds are distinct refusals.
    #[test]
    fn recognized_but_unavailable_setting_has_operation_specific_error() {
        let overrides = LimitOverrides::parse(&["explore.states=11".to_owned()])
            .expect("FSL recognizes exploration settings");
        let mut limits = SpineLimits::default();
        let mut call = call();
        assert_eq!(
            overrides.apply_to(&mut limits, &mut call),
            Err(LimitError::OperationUnavailable {
                setting: SettingName::Stage(Setting::ExploreStates),
                operation: "run",
            })
        );
    }

    /// Trace: FR-003-AC-1, FR-003-AC-6, FSL applies the exposed setting.
    #[test]
    fn s3_nodes_changes_only_the_checking_node_ceiling() {
        let overrides =
            LimitOverrides::parse(&["s3.nodes=7".to_owned()]).expect("FSL recognizes the setting");
        let mut limits = SpineLimits::default();
        let mut call = call();
        let original_checking = limits.checking;
        overrides
            .apply_to(&mut limits, &mut call)
            .expect("FSL exposes the typed node setting");
        assert_eq!(limits.checking, original_checking.with_nodes(7));
    }

    /// Trace: FR-003-AC-9, FR-003-AC-10. FSL-named accounting settings update only the call.
    #[test]
    fn each_named_accounting_setting_updates_only_call_accounting() {
        let defaults = call().accounting;
        let default_bounds = AccountingLimits(defaults).bounds();
        for setting in AccountingSetting::ALL {
            let operand = format!("{}=7", setting.name());
            let overrides = LimitOverrides::parse(&[operand]).expect("FSL parses counter name");
            assert_eq!(overrides.values(), &[(SettingName::Accounting(setting), 7)]);
            let mut limits = SpineLimits::default();
            let before_limits = limits;
            let mut call = call();
            overrides
                .apply_to(&mut limits, &mut call)
                .expect("accounting budgets are available to execute");
            assert_eq!(limits, before_limits);
            for (observed, bound) in AccountingLimits(call.accounting).bounds() {
                let expected = if observed == setting {
                    7
                } else {
                    default_bounds
                        .iter()
                        .find_map(|(default_setting, bound)| {
                            (*default_setting == observed).then_some(*bound)
                        })
                        .expect("FSL exposes every accounting default")
                };
                assert_eq!(bound, expected, "only {setting} should change");
            }
        }
    }

    /// Trace: FR-003-AC-1, typed defaults are inherited when no override is supplied.
    #[test]
    fn an_empty_limit_set_is_a_noop() {
        let mut limits = SpineLimits::default();
        let mut call = call();
        let expected = limits;
        let overrides = LimitOverrides::parse(&[]).expect("empty set");
        overrides
            .apply_to(&mut limits, &mut call)
            .expect("empty set applies");
        assert_eq!(limits, expected);
        assert!(overrides.values().is_empty());
    }
}

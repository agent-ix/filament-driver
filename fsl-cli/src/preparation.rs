// SPDX-License-Identifier: AGPL-3.0-or-later
//! The enforcement point for the caller's monitor plan preparation allowances
//! (FR-003-AC-13, FR-003-AC-17).
//!
//! FSL carries `monitor.preparation_work_units` and `monitor.preparation_storage`
//! in `CommandLimits` and enforces neither; the driver's own
//! `filament_driver::monitor::prepare_monitor` does. This module converts the
//! caller's bounds into that preparation's limits and a reached bound back into
//! FSL's typed refusal, which names the setting and says how to raise it.

use filament_driver::monitor::{MonitorPreparationError, MonitorPreparationLimits as DriverLimits};
use fsl_command::command::{CommandLimits, PreparationBound, PreparationRefusal};

/// The allowances `prepare_monitor` runs under for `caller`.
pub fn driver_limits(caller: &CommandLimits) -> DriverLimits {
    let bounds = caller.preparation();
    DriverLimits {
        work: bounds.work,
        storage: bounds.storage,
    }
}

/// FSL's refusal for a preparation that reached one of `caller`'s allowances,
/// or `None` for any other preparation failure.
pub fn refusal(
    error: &MonitorPreparationError,
    caller: &CommandLimits,
) -> Option<PreparationRefusal> {
    let (bound, consumed, next) = match error {
        MonitorPreparationError::Work { consumed, next, .. } => {
            (PreparationBound::Work, *consumed, *next)
        }
        MonitorPreparationError::Storage { consumed, next, .. } => {
            (PreparationBound::Storage, *consumed, *next)
        }
        _ => return None,
    };
    let actual = u128::from(consumed) + u128::from(next);
    Some(caller.preparation().refusal(bound, actual))
}

#[cfg(test)]
mod tests {
    use super::{driver_limits, refusal};
    use filament_driver::monitor::MonitorPreparationError;
    use fsl_command::command::{CommandLimits, MonitorPreparationLimits};
    use fsl_foundation::{Setting, SettingName};
    use ix_trace_rs::trace;

    fn caller(storage: u64) -> CommandLimits {
        CommandLimits::monitor(MonitorPreparationLimits {
            work: u64::MAX,
            storage,
        })
    }

    /// FR-003-AC-13: the caller's bounds are the bounds preparation runs under.
    #[trace("TC-010", "FR-003-AC-13")]
    #[test]
    fn preparation_runs_under_the_callers_bounds() {
        let limits = driver_limits(&caller(268_435_456));
        assert_eq!((limits.work, limits.storage), (u64::MAX, 268_435_456));
        let mut raised = caller(1);
        raised
            .set(SettingName::Stage(Setting::MonitorPreparationWorkUnits), 7)
            .expect("monitor exposes the work bound");
        let limits = driver_limits(&raised);
        assert_eq!((limits.work, limits.storage), (7, 1));
    }

    /// FR-003-AC-17: a reached storage bound is FSL's typed refusal naming
    /// `monitor.preparation_storage` and how to raise it.
    #[trace("TC-010", "FR-003-AC-17")]
    #[test]
    fn a_reached_storage_bound_names_its_setting() {
        let error = MonitorPreparationError::Storage {
            limit: 125_611_543,
            consumed: 125_611_000,
            next: 544,
        };
        let refused = refusal(&error, &caller(125_611_543)).expect("storage is a bound");
        let exceeded = refused.exceeded();
        assert_eq!(exceeded.setting(), Setting::MonitorPreparationStorage);
        assert_eq!(exceeded.configured_bound(), 125_611_543);
        assert_eq!(exceeded.actual(), 125_611_544);
        let text = refused.to_string();
        assert!(text.contains("monitor.preparation_storage"), "{text}");
        assert!(
            text.contains("--limit monitor.preparation_storage=<n>"),
            "{text}"
        );
    }

    /// FR-003-AC-12: the work bound is its own setting; other failures are no bound.
    #[trace("TC-010", "FR-003-AC-12")]
    #[test]
    fn work_is_its_own_setting_and_other_failures_are_no_bound() {
        let work = MonitorPreparationError::Work {
            limit: 9,
            consumed: 9,
            next: 1,
        };
        let refused = refusal(&work, &caller(1)).expect("work is a bound");
        assert_eq!(
            refused.exceeded().setting(),
            Setting::MonitorPreparationWorkUnits
        );
        let cancelled = MonitorPreparationError::Cancelled(filament_exact::CancelCause::Requested);
        assert!(refusal(&cancelled, &caller(1)).is_none());
    }
}

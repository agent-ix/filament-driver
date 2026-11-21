// SPDX-License-Identifier: AGPL-3.0-or-later
//! Resource workloads shared by the Rust process-provider fixtures.

/// The resource a test provider consumes after admitting its request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceWorkload {
    /// Reserve more than the small test process memory ceiling.
    Memory,
    /// Consume CPU until the OS ceiling or the test watchdog stops the process.
    Cpu,
}

/// Run the selected workload. A refused memory reservation exits with status 71;
/// the CPU workload runs until the process is terminated by its owner.
pub fn consume_resource(workload: ResourceWorkload) {
    match workload {
        ResourceWorkload::Memory => {
            let mut allocation = Vec::<u8>::new();
            if allocation.try_reserve_exact(256 * 1024 * 1024).is_err() {
                std::process::exit(71);
            }
        }
        ResourceWorkload::Cpu => loop {
            std::hint::black_box(1_u64.wrapping_mul(3));
        },
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entry point for the user-facing `fsl` executable.

mod analyze_provider;
#[allow(
    dead_code,
    reason = "FR-331 request members await the remaining FSL projection support"
)]
mod backend_provider;
mod cli;
mod clocks;
mod extension_set;
mod options;
#[allow(
    dead_code,
    reason = "FR-003 monitor preparation allowances await a CLI path that prepares a plan"
)]
mod preparation;
mod process_provider;
mod provider_config;
mod signals;
#[cfg(test)]
mod test_support;

fn main() -> std::process::ExitCode {
    cli::entrypoint()
}

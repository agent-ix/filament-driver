// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entry point for the user-facing `fsl` executable.

#[allow(
    dead_code,
    reason = "FR-331 request members await the remaining FSL projection support"
)]
mod analyze_provider;
mod backend_provider;
mod cli;
mod clocks;
#[allow(
    dead_code,
    reason = "typed process results remain unmapped pending FSL settlement APIs"
)]
mod process_provider;
mod provider_config;
mod signals;
#[cfg(test)]
mod test_support;

fn main() -> std::process::ExitCode {
    cli::entrypoint()
}

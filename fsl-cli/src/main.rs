// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entry point for the user-facing `fsl` executable.

mod analyze_provider;
mod backend_provider;
mod cli;
mod clocks;
mod process_provider;
mod provider_config;
mod signals;
#[cfg(test)]
mod test_support;

fn main() -> std::process::ExitCode {
    cli::entrypoint()
}

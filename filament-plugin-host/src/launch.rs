// SPDX-License-Identifier: AGPL-3.0-or-later
//! Apply process resource limits before replacing this process with a provider.

use std::io;

#[cfg(unix)]
fn launch() -> io::Result<()> {
    use std::os::unix::process::CommandExt;

    use rlimit::Resource;

    let mut arguments = std::env::args_os().skip(1);
    let mut limit = || {
        arguments
            .next()
            .and_then(|value| value.to_str().and_then(|value| value.parse::<u64>().ok()))
            .filter(|value| *value > 0)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "expected a positive limit"))
    };
    let memory_bytes = limit()?;
    let cpu_seconds = limit()?.div_ceil(1000);
    let executable = arguments.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a provider executable",
        )
    })?;
    // Never raise an inherited hard ceiling. Both limits survive exec.
    for (resource, maximum) in [
        (Resource::AS, memory_bytes),
        (Resource::DATA, memory_bytes),
        (Resource::CPU, cpu_seconds),
    ] {
        if resource.is_supported() {
            let inherited = resource.get_hard().map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("reading {}: {error}", resource.as_name()),
                )
            })?;
            let hard = inherited.min(maximum);
            resource.set(hard, hard).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("setting {} to {hard}: {error}", resource.as_name()),
                )
            })?;
        }
    }
    // A failed provider must not leave an unbounded core file behind.
    Resource::CORE.set(0, 0)?;
    Err(std::process::Command::new(executable)
        .args(arguments)
        .exec())
}

#[cfg(not(unix))]
fn launch() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "the resource-limit launcher requires Unix",
    ))
}

fn main() -> io::Result<()> {
    launch()
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! `--clocks <json-file>` on `fsl compile` and `fsl run` (FR-023).
//!
//! The file's whole content is the request's FR-640 `clocks` object. The
//! driver checks that it is readable and one JSON object, then hands it to
//! FSL as the request-override `clocks`, which FSL reads exactly as the
//! request member and refuses when the request carries one too.

use std::io::Read;
use std::path::Path;

use fsl_foundation::diagnostic::Code;
use serde_json::value::RawValue;

/// Why `--clocks` could not be served, with the diagnostic it records.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ClocksRefusal {
    pub(crate) code: Code,
    pub(crate) cause: &'static str,
    pub(crate) message: String,
}

impl ClocksRefusal {
    fn invalid(cause: &'static str, message: String) -> Self {
        Self {
            code: Code::InvalidRequest,
            cause,
            message,
        }
    }
}

/// Read the `--clocks` file as one JSON object, keeping its text as written
/// so a repeated member name reaches FSL's own reader.
pub(crate) fn read(
    path: &Path,
    limit: usize,
    option: Option<&str>,
) -> Result<Box<RawValue>, ClocksRefusal> {
    let mut bytes = Vec::new();
    let cap = u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1);
    std::fs::File::open(path)
        .and_then(|file| file.take(cap).read_to_end(&mut bytes))
        .map_err(|error| ClocksRefusal {
            code: Code::IoError,
            cause: "clocks-file-unreadable",
            message: format!(
                "{}: could not read {}: {error}",
                crate::options::CLOCKS,
                path.display()
            ),
        })?;
    if bytes.len() > limit {
        let raise = option.map_or_else(String::new, |option| format!("; raise {option}"));
        return Err(ClocksRefusal {
            code: Code::ResourceExhausted,
            cause: "clocks-file-too-large",
            message: format!(
                "{}: {} exceeds the {limit} byte limit{raise}",
                crate::options::CLOCKS,
                path.display()
            ),
        });
    }
    let not_an_object = || {
        ClocksRefusal::invalid(
            "clocks-file-not-an-object",
            format!(
                "{}: {} is not one JSON object of clock configurations",
                crate::options::CLOCKS,
                path.display()
            ),
        )
    };
    let text = String::from_utf8(bytes).map_err(|_| not_an_object())?;
    let raw = RawValue::from_string(text.trim().to_owned()).map_err(|_| not_an_object())?;
    if !raw.get().starts_with('{') {
        return Err(not_an_object());
    }
    Ok(raw)
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `--extensions <file>` document of `parse` and `check` (FSpec FR-562).
//!
//! The document is `{"format": "filament.extension-set/v1", "extensions":
//! [...]}`; each entry names one definition by identity, its document file
//! (resolved against the set document's directory), its dependency edges and
//! its capabilities. The caller names every definition explicitly, so a
//! result depends only on the inputs it names.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use fsl_command::command::CheckExtensionInput;
use fsl_foundation::diagnostic::Code;
use fsl_foundation::selection::DefinitionRef;
use fsl_semantics::library::bundle::{CapabilityId, DefinitionCatalog, ExtensionDefinitionSource};
use serde::Deserialize;

/// The `format` member every set document carries.
const FORMAT: &str = "filament.extension-set/v1";

/// Largest set document or definition document the driver reads.
const MAX_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetFile {
    format: String,
    extensions: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    authority: String,
    identity: String,
    file: String,
    #[serde(default)]
    depends_on: Vec<Reference>,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    authority: String,
    identity: String,
}

/// Why a set document or a definition it names was refused.
#[derive(Debug)]
pub(crate) struct SetRefusal {
    /// The FSL code of the refusal.
    pub code: Code,
    /// The message naming the member or file at fault.
    pub message: String,
}

impl SetRefusal {
    fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, SetRefusal> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(MAX_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|error| {
            SetRefusal::new(
                Code::IoError,
                format!("could not read {}: {error}", path.display()),
            )
        })?;
    if u64::try_from(bytes.len()).map_or(true, |length| length > MAX_BYTES) {
        return Err(SetRefusal::new(
            Code::InvalidRequest,
            format!("{} is larger than {MAX_BYTES} bytes", path.display()),
        ));
    }
    Ok(bytes)
}

/// Read the set document at `path` into the entries FSL's `check` takes.
pub(crate) fn read(path: &Path) -> Result<Vec<CheckExtensionInput>, SetRefusal> {
    let bytes = read_bounded(path)?;
    let set: SetFile = serde_json::from_slice(&bytes).map_err(|error| {
        SetRefusal::new(
            Code::InvalidRequest,
            format!("{} is not a valid extension set: {error}", path.display()),
        )
    })?;
    if set.format != FORMAT {
        return Err(SetRefusal::new(
            Code::InvalidRequest,
            format!(
                "{}: member `format` must be {FORMAT}, found `{}`",
                path.display(),
                set.format
            ),
        ));
    }
    let directory = path.parent().unwrap_or_else(|| Path::new(""));
    let mut identities = BTreeSet::new();
    let mut entries = Vec::with_capacity(set.extensions.len());
    for (index, entry) in set.extensions.into_iter().enumerate() {
        for (member, value) in [
            ("authority", &entry.authority),
            ("identity", &entry.identity),
            ("file", &entry.file),
        ] {
            if value.trim().is_empty() {
                return Err(SetRefusal::new(
                    Code::InvalidRequest,
                    format!("extensions[{index}].{member} must be a non-blank string"),
                ));
            }
        }
        if !identities.insert((entry.authority.clone(), entry.identity.clone())) {
            return Err(SetRefusal::new(
                Code::InvalidPackage,
                format!(
                    "conflicting-definition: extensions[{index}] repeats {}:{}",
                    entry.authority, entry.identity
                ),
            ));
        }
        entries.push(CheckExtensionInput {
            authority: entry.authority,
            identity: entry.identity,
            file: directory.join(entry.file),
            depends_on: entry
                .depends_on
                .into_iter()
                .map(|reference| (reference.authority, reference.identity))
                .collect(),
            capabilities: entry.capabilities,
        });
    }
    Ok(entries)
}

/// Read each entry's definition document into the catalog `parse` selects
/// extensions against.
pub(crate) fn catalog(entries: &[CheckExtensionInput]) -> Result<DefinitionCatalog, SetRefusal> {
    let mut definitions = Vec::with_capacity(entries.len());
    for entry in entries {
        let dependencies = entry
            .depends_on
            .iter()
            .map(|(authority, identity)| {
                DefinitionRef::new(authority.clone(), identity.clone()).map_err(|error| {
                    SetRefusal::new(
                        Code::InvalidRequest,
                        format!(
                            "extension {}:{}: depends_on: {error:?}",
                            entry.authority, entry.identity
                        ),
                    )
                })
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        let capabilities = entry
            .capabilities
            .iter()
            .map(|capability| {
                CapabilityId::complete(capability)
                    .map_err(|error| SetRefusal::new(Code::InvalidPackage, error.to_string()))
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        definitions.push(ExtensionDefinitionSource {
            authority: entry.authority.clone(),
            identity: entry.identity.clone(),
            exact_bytes: read_bounded(&entry.file)?,
            dependencies,
            capabilities,
        });
    }
    DefinitionCatalog::read_extensions(definitions)
        .map_err(|error| SetRefusal::new(Code::InvalidPackage, error.to_string()))
}

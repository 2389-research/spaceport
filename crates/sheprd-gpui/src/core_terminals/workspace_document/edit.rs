//! Applies one Spaceport grouping edit against Core's versioned document.
//! Callers run this blocking operation on a worker, never the UI thread.

mod error;

pub(in crate::core_terminals) use error::{Error, Result};

use super::{Document, NAME, VersionedDocument};
use sheprd_core_client::{Core, Method};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::core_terminals) enum Edit {
    // keep-sorted start
    AddMember {
        id: Uuid,
        core_id: String,
        terminal_id: String,
    },
    Archive {
        id: Uuid,
        archived: bool,
    },
    Create {
        id: Uuid,
        label: String,
    },
    Delete {
        id: Uuid,
    },
    RemoveMember {
        id: Uuid,
        core_id: String,
        terminal_id: String,
    },
    Rename {
        id: Uuid,
        label: String,
    },
    // keep-sorted end
}

impl Edit {
    fn id(&self) -> Uuid {
        match self {
            Self::Create { id, .. }
            | Self::AddMember { id, .. }
            | Self::Rename { id, .. }
            | Self::Archive { id, .. }
            | Self::RemoveMember { id, .. }
            | Self::Delete { id } => *id,
        }
    }

    fn change(&self, document: &mut Document) -> super::Result<()> {
        match self {
            Self::AddMember {
                id,
                core_id,
                terminal_id,
            } => document.add_member(*id, core_id, terminal_id),
            Self::Create { id, label } => document.create(*id, label),
            Self::Rename { id, label } => document.rename(*id, label),
            Self::Archive { id, archived } => document.archive(*id, *archived),
            Self::Delete { id } => document.delete(*id),
            Self::RemoveMember {
                id,
                core_id,
                terminal_id,
            } => document.remove_member(*id, core_id, terminal_id),
        }
    }

    /// Only the target workspace may determine this edit's meaning.
    fn disjoint(&self, base: &Document, latest: &Document) -> bool {
        let current = latest.workspaces().iter().find(|item| item.id == self.id());
        if matches!(self, Self::Create { .. }) {
            return current.is_none();
        }
        base.workspaces().iter().find(|item| item.id == self.id()) == current
    }
}

/// Reads the local Spaceport document, including Core's CAS version.
pub(in crate::core_terminals) fn read(core: &Core) -> Result<VersionedDocument> {
    Ok(VersionedDocument::read(core.doc_get(NAME, None)?)?)
}

/// Commits one edit. A definite version conflict gets one disjoint rebase.
/// A timed-out write carries the version it tried, including after a rebase.
pub(in crate::core_terminals) fn apply(
    core: &Core,
    base: VersionedDocument,
    edit: &Edit,
) -> Result<VersionedDocument> {
    apply_with(
        base,
        edit,
        || read(core),
        |value, version| core.doc_set(NAME, value, version, None),
    )
}

fn apply_with(
    base: VersionedDocument,
    edit: &Edit,
    mut read_latest: impl FnMut() -> Result<VersionedDocument>,
    mut write: impl FnMut(
        &serde_json::Value,
        u64,
    ) -> std::result::Result<u64, sheprd_core_client::Error>,
) -> Result<VersionedDocument> {
    let mut desired = base.value.clone();
    edit.change(&mut desired)?;
    match write(&desired.write()?, base.version) {
        Ok(version) => Ok(VersionedDocument {
            version,
            value: desired,
        }),
        Err(error) if is_conflict(&error) => {
            let latest = read_latest()?;
            if !edit.disjoint(&base.value, &latest.value) {
                return Err(Error::Changed(edit.id()));
            }
            let mut merged = latest.value;
            edit.change(&mut merged)?;
            match write(&merged.write()?, latest.version) {
                Ok(version) => Ok(VersionedDocument {
                    version,
                    value: merged,
                }),
                Err(error) if is_conflict(&error) => Err(Error::Conflict),
                Err(error) => Err(write_error(latest.version, error)),
            }
        }
        Err(error) => Err(write_error(base.version, error)),
    }
}

fn write_error(version: u64, error: sheprd_core_client::Error) -> Error {
    // A malformed success or an internal refusal can follow a committed CAS.
    // Only outbound failures and Core's pre-write refusals are definite.
    let definite = match &error {
        sheprd_core_client::Error::Busy
        | sheprd_core_client::Error::Frame(_)
        | sheprd_core_client::Error::Json(_)
        | sheprd_core_client::Error::Unsupported(_) => true,
        sheprd_core_client::Error::Refused {
            kind: Some(kind), ..
        } if kind == "conflict" || kind == "invalid_params" => true,
        _ => false,
    };
    if definite {
        Error::Core(error)
    } else {
        Error::WriteUnknown {
            version,
            source: error,
        }
    }
}

fn is_conflict(error: &sheprd_core_client::Error) -> bool {
    matches!(
        error,
        sheprd_core_client::Error::Refused {
            method: Method::DocSet,
            kind: Some(kind),
            ..
        } if kind == "conflict"
    )
}

#[cfg(test)]
mod tests;

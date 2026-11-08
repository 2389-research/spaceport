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
    Archive { id: Uuid, archived: bool },
    Create { id: Uuid, label: String },
    Delete { id: Uuid },
    Rename { id: Uuid, label: String },
    // keep-sorted end
}

impl Edit {
    fn id(&self) -> Uuid {
        match self {
            Self::Create { id, .. }
            | Self::Rename { id, .. }
            | Self::Archive { id, .. }
            | Self::Delete { id } => *id,
        }
    }

    fn change(&self, document: &mut Document) -> super::Result<()> {
        match self {
            Self::Create { id, label } => document.create(*id, label),
            Self::Rename { id, label } => document.rename(*id, label),
            Self::Archive { id, archived } => document.archive(*id, *archived),
            Self::Delete { id } => document.delete(*id),
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
/// All other Core errors, including timeouts, keep their unknown outcome.
pub(in crate::core_terminals) fn apply(
    core: &Core,
    base: VersionedDocument,
    edit: &Edit,
) -> Result<VersionedDocument> {
    let mut desired = base.value.clone();
    edit.change(&mut desired)?;
    match core.doc_set(NAME, &desired.write()?, base.version, None) {
        Ok(version) => Ok(VersionedDocument {
            version,
            value: desired,
        }),
        Err(error) if is_conflict(&error) => {
            let latest = read(core)?;
            if !edit.disjoint(&base.value, &latest.value) {
                return Err(Error::Changed(edit.id()));
            }
            let mut merged = latest.value;
            edit.change(&mut merged)?;
            match core.doc_set(NAME, &merged.write()?, latest.version, None) {
                Ok(version) => Ok(VersionedDocument {
                    version,
                    value: merged,
                }),
                Err(error) if is_conflict(&error) => Err(Error::Conflict),
                Err(error) => Err(Error::Core(error)),
            }
        }
        Err(error) => Err(Error::Core(error)),
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

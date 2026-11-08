//! Spaceport's versioned presentation grouping, stored as an opaque Core document.
//! This module changes view metadata only; terminal processes remain Core-owned.

pub(super) mod edit;
mod error;

pub use error::{Error, Result};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sheprd_core_client::wire::Doc;
use std::collections::{BTreeMap, HashSet};
use uuid::Uuid;

const SCHEMA_VERSION: u64 = 1;
const MAX_LABEL_CHARS: usize = 200;
pub(super) const NAME: &str = "spaceport";

#[derive(Clone, Debug)]
pub(super) struct VersionedDocument {
    pub version: u64,
    pub value: Document,
}

impl VersionedDocument {
    pub fn read(doc: Doc) -> Result<Self> {
        Ok(Self {
            version: doc.version,
            value: Document::read(doc.value)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    schema_version: u64,
    workspaces: Vec<Workspace>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub label: String,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub launch_on: Option<String>,
    #[serde(default)]
    pub members: Vec<TerminalRef>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerminalRef {
    pub core_id: String,
    pub terminal_id: String,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

impl Document {
    /// Core returns null and version zero before this name has been written.
    pub fn read(value: Value) -> Result<Self> {
        if value.is_null() {
            return Ok(Self {
                schema_version: SCHEMA_VERSION,
                workspaces: Vec::new(),
                extra: BTreeMap::new(),
            });
        }
        if let Some(version) = value.get("schema_version").and_then(Value::as_u64) {
            if version > SCHEMA_VERSION {
                return Err(Error::NewerSchema(version));
            }
            if version != SCHEMA_VERSION {
                return Err(Error::UnsupportedSchema(version));
            }
        }
        let document: Self = serde_json::from_value(value)?;
        document.validate()?;
        Ok(document)
    }

    pub fn write(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    pub fn create(&mut self, id: Uuid, label: &str) -> Result<()> {
        if id.is_nil() {
            return Err(Error::InvalidWorkspaceId);
        }
        validate_label(label)?;
        if self.workspaces.iter().any(|workspace| workspace.id == id) {
            return Err(Error::DuplicateWorkspace(id));
        }
        self.workspaces.push(Workspace {
            id,
            label: label.to_owned(),
            archived: false,
            launch_on: None,
            members: Vec::new(),
            extra: BTreeMap::new(),
        });
        Ok(())
    }

    pub fn rename(&mut self, id: Uuid, label: &str) -> Result<()> {
        validate_label(label)?;
        self.find_mut(id)?.label = label.to_owned();
        Ok(())
    }

    pub fn archive(&mut self, id: Uuid, archived: bool) -> Result<()> {
        self.find_mut(id)?.archived = archived;
        Ok(())
    }

    pub fn add_member(&mut self, id: Uuid, core_id: &str, terminal_id: &str) -> Result<()> {
        if !valid_core_id(core_id) || !valid_terminal_id(terminal_id) {
            return Err(Error::InvalidTerminalRef);
        }
        let workspace = self.find_mut(id)?;
        if workspace
            .members
            .iter()
            .any(|member| member.core_id == core_id && member.terminal_id == terminal_id)
        {
            return Err(Error::DuplicateTerminalRef);
        }
        workspace.members.push(TerminalRef {
            core_id: core_id.to_owned(),
            terminal_id: terminal_id.to_owned(),
            extra: BTreeMap::new(),
        });
        Ok(())
    }

    pub fn remove_member(&mut self, id: Uuid, core_id: &str, terminal_id: &str) -> Result<()> {
        let workspace = self.find_mut(id)?;
        let Some(index) = workspace
            .members
            .iter()
            .position(|member| member.core_id == core_id && member.terminal_id == terminal_id)
        else {
            return Err(Error::MissingTerminalRef);
        };
        workspace.members.remove(index);
        Ok(())
    }

    pub fn delete(&mut self, id: Uuid) -> Result<()> {
        let Some(index) = self
            .workspaces
            .iter()
            .position(|workspace| workspace.id == id)
        else {
            return Err(Error::MissingWorkspace(id));
        };
        self.workspaces.remove(index);
        Ok(())
    }

    fn find_mut(&mut self, id: Uuid) -> Result<&mut Workspace> {
        self.workspaces
            .iter_mut()
            .find(|workspace| workspace.id == id)
            .ok_or(Error::MissingWorkspace(id))
    }

    fn validate(&self) -> Result<()> {
        let mut ids = HashSet::new();
        for workspace in &self.workspaces {
            if workspace.id.is_nil() {
                return Err(Error::InvalidWorkspaceId);
            }
            if !ids.insert(workspace.id) {
                return Err(Error::DuplicateWorkspace(workspace.id));
            }
            validate_label(&workspace.label)?;
            if workspace
                .launch_on
                .as_deref()
                .is_some_and(|id| !valid_core_id(id))
            {
                return Err(Error::InvalidCoreId);
            }
            let mut members = HashSet::new();
            for member in &workspace.members {
                if !valid_core_id(&member.core_id) || !valid_terminal_id(&member.terminal_id) {
                    return Err(Error::InvalidTerminalRef);
                }
                if !members.insert((&member.core_id, &member.terminal_id)) {
                    return Err(Error::DuplicateTerminalRef);
                }
            }
        }
        Ok(())
    }
}

fn validate_label(label: &str) -> Result<()> {
    if label.trim().is_empty()
        || label.chars().count() > MAX_LABEL_CHARS
        || label.chars().any(char::is_control)
    {
        return Err(Error::InvalidLabel);
    }
    Ok(())
}

fn valid_core_id(id: &str) -> bool {
    !id.trim().is_empty()
        && id.len() <= 128
        && !id.contains('/')
        && !id.chars().any(char::is_control)
}

/// Store the local ID Core reported, never a machine-qualified address.
fn valid_terminal_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= sheprd_backend::MAX_TERMINAL_ID
        && !id.contains('/')
        && !id.chars().any(char::is_control)
}

#[cfg(test)]
mod tests;

//! What choosing a row does, and the identity two rows share.

use crate::{
    OwnedNavigationTarget,
    project::{NewProject, Project},
};
use spaceport_client::protocol::ClientShellCommandAction;
use spaceport_commands::Command;

#[derive(Clone)]
pub enum Action {
    Native(Command),
    /// A Go To destination, qualified by the host and daemon boot it was listed from.
    Go {
        endpoint: String,
        boot: String,
        target: OwnedNavigationTarget,
    },
    Configured(String, ClientShellCommandAction),
    Project(Project),
    /// A new project directory to create or clone, named by the query.
    NewProject(NewProject),
    /// A workspace whose checkout has a note, gone to as Go To goes to it.
    Note {
        endpoint: String,
        boot: String,
        workspace: String,
    },
}

/// The same destination reached by another row: a note on a workspace is its
/// workspace, and a configured command is its command ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Identity {
    Native(Command),
    Go(String, OwnedNavigationTarget),
    Configured(String),
    Project(std::path::PathBuf),
    NewProject(std::path::PathBuf),
    Note(String, String),
}

impl Action {
    pub fn identity(&self) -> Identity {
        match self {
            Self::Native(command) => Identity::Native(*command),
            Self::Go {
                endpoint, target, ..
            } => Identity::Go(endpoint.clone(), target.clone()),
            Self::Configured(id, _) => Identity::Configured(id.clone()),
            Self::Project(project) => Identity::Project(project.path.clone()),
            Self::NewProject(project) => Identity::NewProject(project.path.clone()),
            Self::Note {
                endpoint,
                workspace,
                ..
            } => Identity::Note(endpoint.clone(), workspace.clone()),
        }
    }
}

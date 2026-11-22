//! Where the sidebar lists agents while `show_agents` is on.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentsPlacement {
    /// Their own section below the workspaces, every host's agents in one
    /// order, with a split between the two.
    #[default]
    Panel,
    /// Under the workspace each one runs in, so the workspaces take the
    /// sidebar's full height.
    Nested,
}

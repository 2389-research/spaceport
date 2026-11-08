//! Machine shape shared with the retained provider client.
use serde::{Deserialize, Serialize};

/// What kind of machine a provider with more than one makes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    /// The provider's usual machine, such as an Azure VM.
    #[default]
    Machine,
    /// A lighter sandbox, such as an Azure Container Apps session.
    Sandbox,
}

#[cfg(test)]
mod tests;

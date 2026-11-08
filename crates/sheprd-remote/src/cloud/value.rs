//! What a cloud provider refused as not valid or not UTF-8. A setting or
//! environment variable is named as written; every other value is a
//! variant, so the app can translate each sentence whole. `Display` writes
//! each to open the sentence it starts.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    // keep-sorted start
    Account,
    AzureAccessToken,
    AzureClientSecret,
    AzureDeviceAccount,
    AzureSandboxEndpoint,
    AzureSandboxId,
    AzureSshPublicKey,
    ContainerEngine,
    ExecdEndpoint,
    ExecSessionId,
    /// A setting such as `azure.subscription_id` or an environment variable,
    /// shown as written in every language.
    Key(&'static str),
    RemoteCommand,
    SandboxId,
    WorkspaceName,
    // keep-sorted end
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            // keep-sorted start
            Self::Account => "Account",
            Self::AzureAccessToken => "Azure access token",
            Self::AzureClientSecret => "Azure client secret",
            Self::AzureDeviceAccount => "Saved Azure device account",
            Self::AzureSandboxEndpoint => "Azure sandbox endpoint",
            Self::AzureSandboxId => "Azure sandbox ID",
            Self::AzureSshPublicKey => "Azure SSH public key",
            Self::ContainerEngine => "Container engine",
            Self::ExecdEndpoint => "Execd endpoint",
            Self::ExecSessionId => "Exec session ID",
            Self::Key(key) => key,
            Self::RemoteCommand => "Remote command",
            Self::SandboxId => "Sandbox ID",
            Self::WorkspaceName => "Workspace name",
            // keep-sorted end
        })
    }
}

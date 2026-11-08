//! What a cloud provider refused, shared by every provider's `Field` and
//! `Encoding`: one message per sentence, chosen by `$value`, with a setting
//! or environment variable named as written.

use crate::i18n::fl;
use sheprd_remote::cloud::Value;

/// `value` is not valid.
pub(super) fn invalid(value: Value) -> String {
    match value {
        Value::Key(key) => fl!("error-remote-cloud-value-invalid-key", key = key),
        value => fl!("error-remote-cloud-value-invalid", value = selector(value)),
    }
}

/// `value` is not UTF-8. Every provider but containers reads text that
/// may not be.
#[cfg(any(feature = "azure", feature = "sprites"))]
pub(super) fn not_utf8(value: Value) -> String {
    match value {
        Value::Key(key) => fl!("error-remote-cloud-value-encoding-key", key = key),
        value => fl!("error-remote-cloud-value-encoding", value = selector(value)),
    }
}

/// The Fluent selector for each value but a key.
fn selector(value: Value) -> &'static str {
    match value {
        Value::Account => "account",
        Value::AzureAccessToken => "azure-access-token",
        Value::AzureClientSecret => "azure-client-secret",
        Value::AzureDeviceAccount => "azure-device-account",
        Value::AzureSandboxEndpoint => "azure-sandbox-endpoint",
        Value::AzureSandboxId => "azure-sandbox-id",
        Value::AzureSshPublicKey => "azure-ssh-public-key",
        Value::ContainerEngine => "container-engine",
        Value::ExecSessionId => "exec-session-id",
        Value::ExecdEndpoint => "execd-endpoint",
        Value::RemoteCommand => "remote-command",
        Value::SandboxId => "sandbox-id",
        Value::SpriteName => "sprite-name",
        Value::SpritePrefix => "sprite-prefix",
        Value::SpritesApiToken => "sprites-api-token",
        Value::WorkspaceName => "workspace-name",
        Value::Key(_) => "key",
    }
}

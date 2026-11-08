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

/// The Fluent selector for each value but a key.
fn selector(value: Value) -> &'static str {
    match value {
        Value::Account => "account",
        Value::ContainerEngine => "container-engine",
        Value::ExecSessionId => "exec-session-id",
        Value::ExecdEndpoint => "execd-endpoint",
        Value::RemoteCommand => "remote-command",
        Value::SandboxId => "sandbox-id",
        Value::WorkspaceName => "workspace-name",
        Value::Key(_) => "key",
    }
}

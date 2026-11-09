//! Usage observation and host-shell errors, localized without account details.
//! Shell process context remains a placeable when the operation has no known label.

use super::Localize;
use crate::i18n::fl;
use sheprd_usage::Error;

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Client(error) => error.localize(),
            Self::Connect => fl!("error-usage-connect"),
            Self::NoPlan => fl!("error-usage-no-plan"),
            Self::Process { operation, .. } => fl!(
                "error-usage-process",
                kind = process_step(operation),
                operation = *operation
            ),
            Self::RateLimited => fl!("error-usage-rate-limited"),
            Self::Size => fl!("error-usage-size"),
            Self::Status(status) => fl!("error-usage-status", status = *status),
            Self::Timeout => fl!("error-usage-timeout"),
            Self::Unavailable => fl!("usage-unavailable"),
            Self::Unreachable => fl!("error-usage-unreachable"),
            Self::WslHostUnsupported => fl!("error-usage-wsl-host-unsupported"),
            // keep-sorted end
        }
    }
}

/// Keep the existing host-shell operation label; other process failures retain their context.
fn process_step(operation: &str) -> &'static str {
    match operation {
        "start the remote usage shell" => "remote-shell", // i18n: data
        _ => "other",
    }
}

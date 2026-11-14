//! Host-shell errors, from `error-shell.ftl`. Process context remains a
//! placeable when the operation has no known label.

use super::Localize;
use crate::i18n::fl;
use sheprd_usage::shell::{Error, START};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Client(error) => error.localize(),
            Self::Process { operation, .. } => fl!(
                "error-shell-process",
                kind = process_step(operation),
                operation = *operation
            ),
            Self::Size => fl!("error-shell-size"),
            Self::Timeout => fl!("error-shell-timeout"),
            Self::Unreachable => fl!("error-shell-unreachable"),
            Self::WslHostUnsupported => fl!("error-shell-wsl-host-unsupported"),
            // keep-sorted end
        }
    }
}

/// The shell's own start has a message; other process failures keep their context.
fn process_step(operation: &str) -> &'static str {
    match operation {
        START => "host-shell", // i18n: data
        _ => "other",
    }
}

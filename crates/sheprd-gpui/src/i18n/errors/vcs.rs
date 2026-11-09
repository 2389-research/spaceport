//! `sheprd_vcs`'s errors from `error-vcs.ftl`: Git commands.
//! GitHub failures go through `sheprd_github`'s mapping. Git's own output
//! stays a placeable.

use super::Localize;
use crate::i18n::fl;
use sheprd_vcs::{Error, git};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Git(error) => error.localize(),
            Self::GitHub(error) => error.localize(),
            // keep-sorted end
        }
    }
}

impl Localize for git::Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Failed { operation, details } => fl!(
                "error-vcs-git-failed",
                kind = failed_step(operation),
                operation = *operation,
                details = details.as_str()
            ),
            Self::Process { operation, .. } => fl!(
                "error-vcs-git-process",
                kind = match *operation {
                    "start the Git worker" => "start-worker", // i18n: data
                    _ => "other",
                },
                operation = *operation
            ),
            Self::Worker => fl!("error-vcs-git-worker"),
            // keep-sorted end
        }
    }
}

/// The message key for a failed Git command's `operation`, which its
/// callers word in English; one not listed is shown as it is.
fn failed_step(operation: &str) -> &'static str {
    match operation {
        // keep-sorted start
        "check uncommitted files" => "check-uncommitted", // i18n: data
        "check unpushed commits" => "check-unpushed",     // i18n: data
        "count working tree changes" => "count-working-tree", // i18n: data
        "fetch the pull request branch" => "fetch-pull-request", // i18n: data
        "find the checkout" => "find-checkout",           // i18n: data
        "find where the branch started" => "find-branch-start", // i18n: data
        "list attribute files" => "list-attribute-files", // i18n: data
        "list branches" => "list-branches",               // i18n: data
        "list untracked files" => "list-untracked",       // i18n: data
        "list working tree changes" => "list-working-tree", // i18n: data
        "prepare the remote pull request or issue checkout" => "prepare-remote-checkout", // i18n: data
        "read attributes" => "read-attributes", // i18n: data
        "read HEAD" => "read-head",             // i18n: data
        "read the remote's default branch" => "read-default-branch", // i18n: data
        "read working tree changes" => "read-working-tree-changes", // i18n: data
        "read working tree state" => "read-working-tree-state", // i18n: data
        // keep-sorted end
        _ => "other",
    }
}

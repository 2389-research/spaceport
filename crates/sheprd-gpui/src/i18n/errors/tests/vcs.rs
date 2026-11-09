use super::*;
use sheprd_vcs::{Error, git};
use std::io;

fn io() -> io::Error {
    io::Error::other("pipe closed")
}

#[test]
fn every_variant_has_a_message() {
    assert_localized(&[
        Error::Git(git::Error::Worker),
        Error::GitHub(sheprd_github::Error::Query),
    ]);
}

#[test]
fn every_git_variant_has_a_message() {
    let mut errors = vec![git::Error::Worker];
    errors.extend(
        [
            "check uncommitted files",
            "check unpushed commits",
            "count working tree changes",
            "fetch the pull request branch",
            "find the checkout",
            "find where the branch started",
            "list attribute files",
            "list branches",
            "list untracked files",
            "list working tree changes",
            "prepare the remote pull request or issue checkout",
            "read HEAD",
            "read attributes",
            "read the remote's default branch",
            "read working tree changes",
            "read working tree state",
            "rebase onto the moon",
        ]
        .map(|operation| git::Error::Failed {
            operation,
            details: "fatal: not a git repository".into(),
        }),
    );
    errors.extend(["start the Git worker", "start the moon"].map(|operation| {
        git::Error::Process {
            operation,
            source: io(),
        }
    }));
    assert_localized(&errors);
}

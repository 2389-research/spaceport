//! The checkout of a directory on this machine, found with Git, for the
//! `scripts list`, run in this process for the current directory's checkout.
use crate::{Error, Result};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

/// One line of a Git query in `dir`, or `None` when Git refused it.
fn git(dir: &Path, args: &[&str]) -> Result<Option<String>> {
    let output = sheprd_client::new_command("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(Error::LocalGit)?;
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.trim_end_matches(['\n', '\r']);
    Ok((output.status.success() && !line.is_empty()).then(|| line.to_owned()))
}

/// The root of the checkout `dir` is in.
pub(super) fn root(dir: &Path) -> Result<PathBuf> {
    git(dir, &["rev-parse", "--show-toplevel"])?
        .map(PathBuf::from)
        .ok_or(Error::NotInCheckout)
}

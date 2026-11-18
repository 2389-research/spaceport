// ABOUTME: Selects the expected state root for the Cargo-built app under test.
// ABOUTME: Shares compiled build metadata across CLI and native GUI consumers.
use std::path::{Path, PathBuf};

pub(super) fn app_state_path(root: &Path) -> PathBuf {
    let worktree = (env!("HERDR_BUILD_WORKTREE") == "1").then_some(env!("CARGO_MANIFEST_DIR"));
    let namespace = sheprd_client::local_path::state_namespace(
        env!("SPACEPORT_BUILD_CHANNEL") == "macos-preview",
        env!("SPACEPORT_BUILD_PROFILE"),
        worktree,
    )
    .unwrap();
    root.join(namespace)
}

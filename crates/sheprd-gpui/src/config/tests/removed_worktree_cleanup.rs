// ABOUTME: Checks an old merged-worktree setting cannot restore automatic deletion.
// ABOUTME: Rejects both retired worktree cleanup and workspace close preferences.
use super::*;

#[test]
fn retired_automatic_worktree_cleanup_setting_is_unknown() -> anyhow::Result<()> {
    let config =
        Config::parse("remove_merged_worktrees = true\nconfirm_close_workspace = false\n")?;
    assert_eq!(
        config.unknown_keys,
        ["remove_merged_worktrees", "confirm_close_workspace"]
    );
    Ok(())
}

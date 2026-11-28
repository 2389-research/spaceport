// ABOUTME: Checks removed terminal editor settings cannot select a pane launch.
// ABOUTME: Keeps ordinary file links usable with stale user configuration.
use super::*;

#[test]
fn terminal_editor_settings_are_unknown() -> anyhow::Result<()> {
    let config = Config::parse("open_files_in = 'editor'\neditor_command = 'nvim'")?;
    assert!(config.unknown_keys.contains(&"open_files_in".into()));
    assert!(config.unknown_keys.contains(&"editor_command".into()));
    Ok(())
}

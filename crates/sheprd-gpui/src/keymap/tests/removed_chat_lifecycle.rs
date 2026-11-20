// ABOUTME: Keeps saved chat conversion shortcuts from becoming active commands.
// ABOUTME: Loads real files before resolving retained catalog shortcuts.
use super::*;
use crate::config::Config;
use std::fs;

#[test]
fn removed_chat_lifecycle_shortcut_is_unknown() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("config.toml");
    fs::write(
        &path,
        "[keybindings]\nconvert_to_chat = \"cmd-k\"\nthemes = \"cmd-j\"",
    )?;
    let config = Config::load_fixture(&path, &temp.path().join("view.toml"))?;
    let keymap = crate::config::keymap(&config)?;
    assert_eq!(config.unknown_keys, ["keybindings.convert_to_chat"]);
    assert_eq!(keymap.primary(Command::Themes), "cmd-j");
    Ok(())
}

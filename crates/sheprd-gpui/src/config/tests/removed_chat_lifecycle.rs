// ABOUTME: Keeps saved chat conversion shortcuts from becoming active commands.
// ABOUTME: Retained shortcuts still load through the shared catalog.
use super::*;

#[test]
fn removed_chat_lifecycle_shortcut_is_unknown() -> anyhow::Result<()> {
    let config = Config::parse("[keybindings]\nconvert_to_chat = \"cmd-k\"\nthemes = \"cmd-j\"")?;
    assert_eq!(config.unknown_keys, ["keybindings.convert_to_chat"]);
    assert_eq!(config.keybindings.primary(crate::Command::Themes), "cmd-j");
    Ok(())
}

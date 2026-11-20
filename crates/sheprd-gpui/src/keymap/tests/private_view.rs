// ABOUTME: Verifies prefix keys come from Spaceport's private view file.
// ABOUTME: Keeps runtime preferences separate while preserving terminal Ctrl+B input.
use super::*;
use crate::config::{Config, view_settings_path};
use std::fs;

#[test]
fn ctrl_a_in_private_view_settings_releases_ctrl_b() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let runtime = root.path().join("herdr-config.toml");
    fs::write(&runtime, "[keys]\nprefix = 'ctrl+x'\n")?;
    let path = view_settings_path(|name| match name {
        "XDG_CONFIG_HOME" => Some(root.path().to_path_buf().into_os_string()),
        "HERDR_CONFIG_PATH" => Some(runtime.clone().into_os_string()),
        _ => None,
    });
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, "[keys]\nprefix = 'ctrl+a'\n")?;
    let config = Config::load_fixture(&root.path().join("config.toml"), &path)?;
    let keymap = crate::config::keymap(&config)?;
    let key = |value| Keystroke::parse(value).unwrap();
    assert!(keymap.is_prefix(&key("ctrl-a")));
    assert!(!keymap.is_prefix(&key("ctrl-b")));
    assert!(!keymap.is_prefix(&key("ctrl-x")));
    assert_eq!(fs::read_to_string(runtime)?, "[keys]\nprefix = 'ctrl+x'\n");
    Ok(())
}

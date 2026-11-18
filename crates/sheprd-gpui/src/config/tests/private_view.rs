// ABOUTME: Verifies prefix keys come from Spaceport's private view file.
// ABOUTME: Keeps runtime preferences separate while preserving terminal Ctrl+B input.
use super::*;

#[test]
fn ctrl_a_in_private_view_settings_releases_ctrl_b() -> anyhow::Result<()> {
    let root = TempDirectory::new()?;
    let runtime = root.0.join("herdr-config.toml");
    fs::write(&runtime, "[keys]\nprefix = 'ctrl+x'\n")?;
    let path = view_settings_path(|name| match name {
        "XDG_CONFIG_HOME" => Some(root.0.clone().into_os_string()),
        "HERDR_CONFIG_PATH" => Some(runtime.clone().into_os_string()),
        _ => None,
    });
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, "[keys]\nprefix = 'ctrl+a'\n")?;
    let config = Config::parse_layers([""], &daemon_settings(&path))?;
    let key = |value| gpui::Keystroke::parse(value).unwrap();
    assert!(config.keybindings.is_prefix(&key("ctrl-a")));
    assert!(!config.keybindings.is_prefix(&key("ctrl-b")));
    assert!(!config.keybindings.is_prefix(&key("ctrl-x")));
    assert_eq!(fs::read_to_string(runtime)?, "[keys]\nprefix = 'ctrl+x'\n");
    Ok(())
}

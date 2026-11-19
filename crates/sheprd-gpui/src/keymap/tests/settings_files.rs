// ABOUTME: Verifies loaded view chords and GUI overrides build the real app keymap.
// ABOUTME: Preserves platform-derived defaults and fallback after malformed view files.
use super::*;
use crate::config::Config;
use std::fs;

/// The private view's `[keys]` reach the GUI keymap under the GUI's own
/// `[keybindings]`, and an invalid view file falls back to
/// Herdr's defaults instead of failing the GUI config.
#[test]
fn view_keys_layer_under_gui_keybindings() -> anyhow::Result<()> {
    use crate::Command;
    let temp = tempfile::tempdir()?;
    let gui = temp.path().join("config.toml");
    let local = gui.with_extension("local.toml");
    let view = temp.path().join("view.toml");
    let load = || crate::config::keymap(&Config::load_fixture(&gui, &view)?);
    fs::write(&gui, "")?;
    let shortcuts = |config: &Keymap, command| {
        config
            .shortcuts(command)
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };

    // No private view file: Herdr's defaults.
    assert_eq!(
        shortcuts(&load()?, Command::Tab),
        [crate::controls::platform_default("cmd-t"), "ctrl-b c"]
    );

    fs::write(
        &view,
        "[keys]\nprefix = \"ctrl+a\"\nsplit_vertical = [\"prefix+v\", \"prefix+\\\\\"]\nswitch_tab = [\"prefix+1..9\", \"alt+1..9\"]\n",
    )?;
    let config = load()?;
    assert_eq!(
        shortcuts(&config, Command::SplitRight),
        [
            crate::controls::platform_default("cmd-d"),
            "ctrl-a v",
            "ctrl-a \\"
        ]
    );
    let mut expected = COMMANDS
        .iter()
        .find(|info| info.command == Command::TabNumber(2))
        .unwrap()
        .default_shortcuts()
        .iter()
        .map(|key| (*key).to_owned())
        .collect::<Vec<_>>();
    expected.extend(["ctrl-a 2".to_owned(), "alt-2".to_owned()]);
    assert_eq!(shortcuts(&config, Command::TabNumber(2)), expected);
    assert!(
        config
            .bindings()
            .any(|binding| binding == (Command::TabNumber(2), "alt-2"))
    );

    // The GUI's own entry replaces the command's list, daemon chords too.
    fs::write(&local, "[keybindings]\nsplit_right = \"cmd-d\"\n")?;
    assert_eq!(shortcuts(&load()?, Command::SplitRight), ["cmd-d"]);

    fs::write(&local, "")?;
    fs::write(&view, "[keys\nprefix = ")?;
    assert_eq!(
        shortcuts(&load()?, Command::Tab),
        [crate::controls::platform_default("cmd-t"), "ctrl-b c"]
    );
    Ok(())
}

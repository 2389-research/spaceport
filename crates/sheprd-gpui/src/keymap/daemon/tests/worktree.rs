use super::*;
use crate::keymap::{Binding, Keymap};
use std::collections::BTreeMap;

/// Herdr leaves the open worktree action unbound, so it binds only what the
/// user writes, prefix chords and direct keys alike.
#[test]
fn worktree_open_is_unbound_by_default_and_binds_what_the_user_writes() {
    let defaults = DaemonKeys::default();
    assert!(bound(&defaults, Command::OpenWorktree).is_empty());
    let keys = keys(
        r#"
        [keys]
        open_worktree = "prefix+shift+o"
        "#,
    );
    assert_eq!(bound(&keys, Command::OpenWorktree), [prefixed("shift-o")]);
    let keymap = Keymap::with_overrides(&BTreeMap::new(), &Default::default(), &keys).unwrap();
    assert_eq!(
        keymap.chord(&parsed("shift-o")),
        Some(Command::OpenWorktree)
    );
    for value in ["''", "[]"] {
        let disabled = self::keys(&format!("[keys]\nopen_worktree = {value}"));
        assert!(bound(&disabled, Command::OpenWorktree).is_empty());
    }
    // Navigate mode's pane keys stay unmapped and bind nothing.
    let navigate = self::keys("[keys]\nnavigate_pane_left = \"h\"");
    assert_eq!(navigate, DaemonKeys::default());
}

/// A `[keybindings]` entry replaces the daemon's bindings for that command,
/// and a GUI-configured keystroke wins over a daemon one for another.
#[test]
fn gui_keybindings_take_precedence_over_daemon_worktree_keys() {
    let keys = keys(
        r#"
        [keys]
        open_worktree = ["prefix+o", "ctrl+alt+o"]
        "#,
    );
    let overrides = BTreeMap::from([
        (
            "open_worktree".to_owned(),
            Binding::One("ctrl-alt-shift-o".into()),
        ),
        ("new_tab".to_owned(), Binding::One("ctrl-alt-r".into())),
    ]);
    let keymap = Keymap::with_overrides(&overrides, &Default::default(), &keys).unwrap();
    assert_eq!(
        keymap.shortcuts(Command::OpenWorktree).collect::<Vec<_>>(),
        ["ctrl-alt-shift-o"]
    );
    // `prefix+o` stays Herdr's default notification target.
    assert_eq!(
        keymap.chord(&parsed("o")),
        Some(Command::OpenNotificationTarget)
    );
    assert!(!keymap.triggers(Command::OpenWorktree, &parsed("ctrl-alt-o"), false));
    // The GUI binding for new_tab still takes its direct key.
    assert!(keymap.triggers(Command::Tab, &parsed("ctrl-alt-r"), false));

    // The palette and `[keybindings]` know the retained command.
    let named = BTreeMap::from([(
        "open_worktree".to_owned(),
        Binding::One("ctrl-alt-shift-o".into()),
    )]);
    let keymap =
        Keymap::with_overrides(&named, &Default::default(), &DaemonKeys::default()).unwrap();
    assert_eq!(keymap.primary(Command::OpenWorktree), "ctrl-alt-shift-o");
}

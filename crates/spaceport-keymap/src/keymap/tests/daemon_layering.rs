use super::*;

/// The daemon's own table, as `spaceport-keymap` reads it, so these two
/// layering rules can be checked against the app's `Keymap`.
fn daemon_keys(text: &str) -> DaemonKeys {
    let table: toml::Table = text.parse().unwrap();
    DaemonKeys::from_table(table.get("keys").and_then(toml::Value::as_table))
}

/// Herdr drops a default that collides with a binding the user wrote, so the
/// user's binding is the one the keymap gives the key to.
#[test]
fn a_configured_daemon_binding_precedes_a_default() {
    let keys = daemon_keys("[keys]\nreload_config = 'prefix+r'");
    let keymap = Keymap::with_overrides(&Default::default(), &Default::default(), &keys).unwrap();
    assert_eq!(keymap.chord(&keystroke("r")), Some(Command::ReloadConfig));
}

/// A binding the daemon file sets is the one that answers its chord, even
/// where another command's catalog default spells the same key after the
/// prefix.
#[test]
fn a_configured_daemon_chord_answers_before_a_default_one() {
    let keys = daemon_keys("[keys]\nnew_tab = 'prefix+q'");
    let keymap = Keymap::with_overrides(&Default::default(), &Default::default(), &keys).unwrap();
    assert_eq!(keymap.chord(&keystroke("q")), Some(Command::Tab));
}

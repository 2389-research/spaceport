use super::*;
use crate::keymap::{Error, daemon};

fn invalid() -> gpui::InvalidKeystrokeError {
    gpui::Keystroke::parse("cmd-n-t").unwrap_err()
}

#[test]
fn every_keymap_variant_has_a_message() {
    let keystroke = || "cmd-t".to_owned();
    assert_localized(&[
        Error::DuplicateKeystroke {
            keystroke: keystroke(),
            first: "new_tab",
            second: "close_tab",
        },
        Error::DuplicatePaneKey(keystroke()),
        Error::EmptyPaneKeyText(keystroke()),
        Error::InvalidKeystroke {
            command: "new_tab",
            keystroke: "cmd-n-t".into(),
            source: invalid(),
        },
        Error::InvalidPaneKey {
            keystroke: "cmd-n-t".into(),
            source: invalid(),
        },
        Error::KeystrokeWithoutModifier {
            command: "new_tab",
            keystroke: "n".into(),
        },
        Error::PaneKeyBound {
            keystroke: keystroke(),
            command: "new_tab",
        },
        Error::PaneKeyTextTooLong {
            keystroke: keystroke(),
            limit: 4096,
        },
        Error::PaneKeyWithoutModifier("n".into()),
        Error::TooManyKeystrokes("new_tab"),
        Error::TooManyPaneKeys(64),
        Error::UnknownKeybinding("new_tabb".into()),
        Error::UnsendablePaneKey {
            from: keystroke(),
            to: "cmd-q".into(),
        },
    ]);
}

/// A keybinding that names no command points at the managed file that lists
/// them, the only file this build reads the list from.
#[test]
fn an_unknown_command_points_at_the_keybindings_list() {
    crate::i18n::show(&Locale::FALLBACK);
    let shown = Error::UnknownKeybinding("new_tabb".into()).localize();
    assert!(shown.contains("spaceport/config.toml"), "{shown}");
    assert!(!shown.contains("config-gpui"), "{shown}");
}

#[test]
fn every_keymap_daemon_variant_has_a_message() {
    assert_localized(&[
        daemon::Error::Missing,
        daemon::Error::NoKeys,
        daemon::Error::Parse(toml::from_str::<toml::Table>("=").unwrap_err()),
        daemon::Error::TooLarge { max: 65536 },
    ]);
}

/// A host's published profile is the crate root's one wrapped failure, and it
/// reads as the daemon error behind it.
#[test]
fn a_wrapped_profile_failure_reads_as_its_own() {
    let wrapped = Error::Daemon(daemon::Error::NoKeys);
    assert_eq!(wrapped.localize(), daemon::Error::NoKeys.localize());
}

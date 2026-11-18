// ABOUTME: Checks Spaceport-specific prefix action names through the real private-view parser.
// ABOUTME: Preserves explicit region and full-screen routes without claiming Herdr TUI defines them.
use super::*;

#[test]
fn private_view_can_bind_spaceport_chrome_commands_as_prefix_chords() {
    let keys = keys(
        "[keys]\nfocus_next_region = 'prefix+f6'\nfocus_previous_region = 'prefix+shift+f6'\ntoggle_full_screen = 'prefix+f11'\n",
    );
    for (command, chord) in [
        (Command::FocusNextRegion, "f6"),
        (Command::FocusPreviousRegion, "shift-f6"),
        (Command::ToggleFullScreen, "f11"),
    ] {
        assert_eq!(bound(&keys, command), [prefixed(chord)]);
    }
}

#[test]
fn chrome_prefix_defaults_follow_the_platform_and_can_be_unbound() {
    let defaults = DaemonKeys::default();
    let unbound = keys(
        "[keys]\nfocus_next_region = ''\nfocus_previous_region = ''\ntoggle_full_screen = ''\n",
    );
    for (command, chord) in [
        (Command::FocusNextRegion, "f6"),
        (Command::FocusPreviousRegion, "shift-f6"),
        (Command::ToggleFullScreen, "f11"),
    ] {
        let expected = if crate::controls::CTRL_SHORTCUTS {
            vec![prefixed(chord)]
        } else {
            Vec::new()
        };
        assert_eq!(bound(&defaults, command), expected);
        assert!(bound(&unbound, command).is_empty());
    }
}

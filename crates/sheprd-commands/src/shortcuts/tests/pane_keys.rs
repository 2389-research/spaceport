// ABOUTME: Protects pane program function and tab-navigation keys on both Ctrl profiles.
// ABOUTME: Keeps the command catalog available while prefix routing owns GUI-only alternatives.
use super::*;

#[test]
fn ctrl_profiles_leave_plain_pane_keys_unbound() {
    for profile in [linux::defaults as Profile, windows::defaults] {
        for text in ["f6", "shift-f6", "f11", "ctrl-pageup", "ctrl-pagedown"] {
            let typed = parse(text);
            for info in COMMANDS {
                assert!(
                    profile(info.command).iter().all(|bound| !typed
                        .should_match(&gpui::KeybindingKeystroke::from_keystroke(parse(bound)))),
                    "{text} was taken by {}",
                    info.name
                );
            }
        }
    }
}

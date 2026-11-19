// ABOUTME: Checks Linux's GNOME exclusions against the shared Ctrl catalog.
// ABOUTME: Keeps shared symbol aliases available outside desktop-reserved shortcuts.
use super::{COMMANDS, Command};
use crate::catalog;

#[test]
fn linux_uses_only_the_listed_platform_fallbacks() {
    for info in COMMANDS {
        let expected = if info.command == Command::ResumePalette {
            &["ctrl-shift-r"][..]
        } else if super::gnome::catalog_uses_gnome_chord(info.command) {
            &[][..]
        } else {
            catalog::linux::defaults(info.command)
        };
        assert_eq!(
            super::linux::defaults(info.command),
            expected,
            "{}",
            info.name
        );
    }
}

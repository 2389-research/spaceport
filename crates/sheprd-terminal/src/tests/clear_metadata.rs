// ABOUTME: Preserves hyperlink and style metadata on spaces cleared into terminal history.
// ABOUTME: Checks real wire cells after height growth restores the retained row.
use super::*;

#[test]
fn clear_keeps_metadata_only_spaces_in_history() {
    for bytes in [
        b"\x1b]8;;https://example.test/kept\x07 \x1b]8;;\x07".as_slice(),
        b"\x1b[1m \x1b[0m",
        b"\x1b[2m \x1b[0m",
        b"\x1b[3m \x1b[0m",
    ] {
        let mut terminal = fed(8, 2, bytes);
        let expected = cell(&terminal.frame(), 0, 0).clone();
        assert_ne!(expected, cell(&emulator(8, 2).frame(), 0, 0).clone());
        terminal.feed(b"\x1b[2J");
        assert_eq!(terminal.scroll_metrics().history, 1, "{bytes:?}");
        terminal.resize(size(8, 3));
        assert_eq!(cell(&terminal.frame(), 0, 0), &expected, "{bytes:?}");
    }
}

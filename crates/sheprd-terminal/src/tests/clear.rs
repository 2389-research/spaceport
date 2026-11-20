// ABOUTME: Checks that empty clears preserve cursor placement without erasing real history.
// ABOUTME: Exercises fragmented commands, synchronized dispatch, and meaningful blank cells.
use super::*;

#[test]
fn empty_clear_preserves_the_cursor_through_height_growth() {
    let prefix = b"\x1b[H\x1b[2J";
    for split in 0..=prefix.len() {
        let mut terminal = emulator(80, 24);
        terminal.feed(&prefix[..split]);
        terminal.feed(&prefix[split..]);
        assert_eq!(terminal.scroll_metrics().history, 0, "split {split}");
        terminal.resize(size(112, 34));
        terminal.feed(b"prompt");
        assert_eq!(line(&terminal.frame(), 0), "prompt", "split {split}");
    }
}

#[test]
fn clear_keeps_existing_blank_history() {
    let mut terminal = fed(8, 2, b"\r\n\r\n");
    assert_eq!(terminal.scroll_metrics().history, 1);
    terminal.feed(b"\x1b[2J");
    assert_eq!(terminal.scroll_metrics().history, 1);
}

#[test]
fn clear_keeps_visible_text_and_decorated_spaces_in_history() {
    for bytes in [
        b"kept".as_slice(),
        b"\x1b[41m \x1b[0m",
        b"\x1b[4m \x1b[0m",
        b"\x1b[7m \x1b[0m",
    ] {
        let mut terminal = fed(8, 2, bytes);
        let expected = cell(&terminal.frame(), 0, 0).clone();
        terminal.feed(b"\x1b[2J");
        assert_eq!(terminal.scroll_metrics().history, 1, "{bytes:?}");
        terminal.resize(size(8, 3));
        assert_eq!(cell(&terminal.frame(), 0, 0), &expected, "{bytes:?}");
    }
}

#[test]
fn empty_clear_keeps_later_output_scrolled_in_the_same_feed() {
    let mut terminal = fed(8, 2, b"\x1b[2Jkeep\r\nline\r\nlast");
    assert_eq!(terminal.scroll_metrics().history, 1);
    terminal.scroll(Scroll::Top);
    assert_eq!(line(&terminal.frame(), 0), "keep");
    assert_eq!(line(&terminal.frame(), 1), "line");
}

#[test]
fn synchronized_empty_clear_waits_for_dispatch_and_keeps_later_output() {
    for forced in [false, true] {
        let mut terminal = emulator(8, 2);
        terminal.feed(b"\x1b[?2026h\x1b[2Jkeep\r\nline\r\nlast");
        assert!(terminal.sync_pending());
        assert_eq!(terminal.scroll_metrics().history, 0);
        assert_eq!(line(&terminal.frame(), 0), "");
        if forced {
            terminal.end_sync();
        } else {
            terminal.feed(b"\x1b[?2026l");
        }
        assert!(!terminal.sync_pending());
        assert_eq!(terminal.scroll_metrics().history, 1);
        terminal.scroll(Scroll::Top);
        assert_eq!(line(&terminal.frame(), 0), "keep");
        assert_eq!(line(&terminal.frame(), 1), "line");
    }
}

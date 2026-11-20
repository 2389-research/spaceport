// ABOUTME: Checks the terminal receipt consumer against complete and partial captures.
// ABOUTME: Rejects echoed arguments, other cases, malformed hex, and wrong byte counts.
use super::*;

#[test]
fn only_a_complete_correlated_capture_yields_bytes() {
    let (nonce, id) = ("0123456789abcdef", "pane-f6");
    let prefix = marker(nonce, id, "READY");
    let suffix = marker(nonce, id, "DONE");
    let screen =
        format!("/bin/sh capture.sh '{nonce}' '{id}' 5\n{prefix}\n 1b 5b 31 37 7e\n{suffix}\n");
    assert_eq!(
        read(&screen, nonce, id, 5),
        Capture::Complete(b"\x1b[17~".to_vec())
    );
    assert_eq!(read(&screen, "other", id, 5), Capture::Pending);
    assert_eq!(read(&screen, nonce, "other", 5), Capture::Pending);
    assert_eq!(
        read(&format!("{prefix}\n 1b 5b"), nonce, id, 5),
        Capture::Pending
    );
    assert_eq!(
        read(
            &format!("/bin/sh capture.sh '{nonce}' '{id}' 5"),
            nonce,
            id,
            5
        ),
        Capture::Pending
    );
}

#[test]
fn done_cannot_hide_missing_extra_or_invalid_bytes() {
    let (nonce, id) = ("0123456789abcdef", "pane-f6");
    for body in [
        "",
        "1b",
        "1b 5b 31 37 7e 00",
        "1b 5b 31 xx 7e",
        "1b 5b 31 37 07e",
    ] {
        let screen = format!(
            "{}\n{body}\n{}",
            marker(nonce, id, "READY"),
            marker(nonce, id, "DONE")
        );
        assert_eq!(read(&screen, nonce, id, 5), Capture::Malformed, "{body}");
    }
}

#[test]
fn invalid_capture_bounds_fail_before_reading_output() {
    for count in [0, 65, usize::MAX] {
        assert_eq!(read("", "nonce", "case", count), Capture::Malformed);
    }
}

//! A panic message and backtrace are arbitrary text: these check what reaches
//! the report and the log once it is bounded and made control-character-free.
use super::*;

#[test]
fn a_message_past_the_limit_is_cut_and_marked() {
    let cut = bounded(&"A".repeat(MAX_MESSAGE * 2), None, MAX_MESSAGE);
    assert!(cut.ends_with(TRUNCATED), "{} bytes", cut.len());
    assert_eq!(cut.len(), MAX_MESSAGE + TRUNCATED.len());
}

#[test]
fn content_past_the_limit_does_not_reach_the_report() {
    // A panic that formats what it read: the tail is what a bound removes.
    let mut secret = "A".repeat(MAX_MESSAGE);
    secret.push_str("hf_secret_token_abcdef123456");
    let cut = bounded(&secret, None, MAX_MESSAGE);
    assert!(!cut.contains("hf_secret_token"), "{cut}");
    assert!(cut.contains(TRUNCATED));
}

#[test]
fn control_characters_become_spaces_so_a_message_cannot_forge_a_field() {
    let cut = bounded("boom\nversion: 9.9.9\r\ttab", None, MAX_MESSAGE);
    assert!(
        !cut.contains(['\n', '\r', '\t']),
        "control character in {cut:?}"
    );
    assert_eq!(cut, "boom version: 9.9.9  tab");
}

#[test]
fn the_home_directory_reads_as_a_tilde() {
    let home = Path::new("/Users/someone");
    assert_eq!(
        bounded(
            "/Users/someone/.config/herdr/config.toml",
            Some(home),
            MAX_MESSAGE
        ),
        "~/.config/herdr/config.toml"
    );
    // A one-character home would rewrite every separator in every path.
    assert_eq!(bounded("/x/y", Some(Path::new("/")), MAX_MESSAGE), "/x/y");
}

#[test]
fn a_long_backtrace_is_bounded() {
    let frames = "   0: some_rather_long_frame_name\n".repeat(10_000);
    let cut = bounded(&frames, None, MAX_BACKTRACE);
    assert!(cut.ends_with(TRUNCATED), "{} bytes", cut.len());
    assert!(cut.len() <= MAX_BACKTRACE + TRUNCATED.len());
}

#[test]
fn a_bounded_message_keeps_the_reports_fields_on_their_own_lines() {
    let message = bounded("first\nversion: 9.9.9", None, MAX_MESSAGE);
    let text = report(
        "1.2.3",
        "2026-10-17 23:05:12.042 +02:00",
        "main",
        &message,
        "src/app.rs:1:2",
        "   0: frame",
    );
    assert!(text.contains("message: first version: 9.9.9\n"), "{text}");
    assert!(!text.contains("\nversion: 9.9.9"), "{text}");
}

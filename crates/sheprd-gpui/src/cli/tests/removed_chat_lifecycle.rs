// ABOUTME: Rejects desktop-owned conversion before launch or control dispatch.
// ABOUTME: Keeps the adapter's manual session CLI separate from app policy.
use super::*;

#[test]
fn removed_chat_lifecycle_cli_is_rejected() {
    for args in [
        vec!["chat", "convert"],
        vec!["chat", "convert", "--pane", "w1:p1", "--json"],
    ] {
        assert_eq!(
            LaunchOptions::parse(args).unwrap_err(),
            CliError::UnknownCommand {
                family: "chat",
                command: "convert".into(),
            }
        );
    }
}

#[test]
fn removed_chat_lifecycle_workers_and_record_writer_are_absent() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for source in [
        "sheprd-gpui/src/chat/convert.rs",
        "sheprd-gpui/src/chat/restore.rs",
        "sheprd-acp/src/restore.rs",
        "sheprd-chat-protocol/src/restore.rs",
    ] {
        assert!(!root.join(source).exists(), "{source}");
    }
}

#[test]
fn removed_chat_lifecycle_flag_replay_and_session_validation_are_absent() {
    let agent = include_str!("../../../../sheprd-remote/src/agent.rs");
    for obsolete in ["pub fn kept_flags", "pub fn accepts"] {
        assert!(!agent.contains(obsolete), "{obsolete}");
    }
}

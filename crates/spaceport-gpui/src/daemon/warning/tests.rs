use super::*;
use spaceport_client::{LocalPeerWarning, UntrustedEndpoint};
use std::path::PathBuf;

mod en_xa;
mod repair_command;

#[test]
fn local_peer_warning_identifies_the_directory_and_quotes_the_remedy() {
    let warning = LocalPeerWarning::new(
        UntrustedEndpoint::DirectoryPermissions,
        "/home/test/it's herdr/herdr-client.sock".into(),
    );
    let notice = notice(&warning, std::time::Instant::now());
    assert_eq!(notice.title, "Local Git and PR details unavailable");
    assert_eq!(
        notice.body.as_deref(),
        Some(
            "The daemon socket directory must be owned by your user and not world-writable. Check ownership and remove world-write access, then reconnect the GUI.\nchmod -- o-w '/home/test/it'\\''s herdr'"
        )
    );
    assert!(notice.is_local_feedback());
    assert!(notice.pane_id.is_none());

    let socket = LocalPeerWarning::new(
        UntrustedEndpoint::SocketPermissions,
        "/home/test/herdr/herdr-client.sock".into(),
    );
    assert!(body(&socket).contains("chmod -- go-w '/home/test/herdr/herdr-client.sock'"));
}

#[test]
fn local_peer_warning_never_suggests_a_sanitized_or_truncated_command() {
    for directory in [
        "bad\npath".into(),
        "x".repeat(201),
        "'".repeat(100),
        "bad\u{202e}path".into(),
    ] {
        let warning = LocalPeerWarning::new(
            UntrustedEndpoint::DirectoryPermissions,
            PathBuf::from("/tmp")
                .join(directory)
                .join("herdr-client.sock"),
        );
        assert!(!body(&warning).contains("chmod"));
        assert!(body(&warning).contains("remove world-write access"));
    }
}

use super::*;
use std::time::Instant;

#[test]
fn rejected_dialog_command_preserves_connection_diagnostic() {
    let mut state = LiveState {
        status: ConnectionStatus::Disconnected,
        dialog_response: Some(("create".into(), None)),
        ..LiveState::default()
    };
    state
        .notices
        .show(crate::notice::Cause::Connection, "socket closed".into());
    state.apply(ClientEvent::CommandRejected {
        request_id: Some("create".into()),
        reason: spaceport_client::Error::Disconnected,
    });
    assert_eq!(
        state.status_text(None, Instant::now()),
        "\u{2068}Disconnected\u{2069}: \u{2068}socket closed\u{2069}"
    );
    assert!(matches!(&state.dialog_response, Some((_, Some(Err(_))))));
}

#[test]
fn untracked_daemon_errors_remain_visible_as_readable_messages() {
    let mut state = LiveState {
        status: ConnectionStatus::Connected,
        ..LiveState::default()
    };
    for (payload, expected) in [
        (
            serde_json::json!({"code": "failed", "message": "Input failed"}),
            "Input failed",
        ),
        (serde_json::json!("Legacy failure"), "Legacy failure"),
        (serde_json::json!({"code": "unsupported"}), "unsupported"),
        (
            serde_json::json!({"unexpected": true}),
            "The daemon refused the request without saying why.",
        ),
    ] {
        state.apply(ClientEvent::Response {
            request_id: "input".into(),
            response: serde_json::json!({"error": payload}),
        });
        assert_eq!(
            state.status_text(None, Instant::now()),
            format!("\u{2068}Connected\u{2069}: \u{2068}{expected}\u{2069}")
        );
    }
}

#[test]
fn missing_installation_survives_disconnect_but_clears_on_success() {
    let mut state = LiveState::default();
    assert!(!state.missing_installation);
    state.missing_installation = true;
    state.apply(ClientEvent::Disconnected {
        reason: "Herdr not found".into(),
        ssh: None,
    });
    assert!(state.missing_installation);
    state.set_outer_focus(true);
    assert!(state.missing_installation);
    state.apply(ClientEvent::Snapshot(snapshot()));
    assert!(!state.missing_installation);
}

#[test]
fn daemon_loader_stops_on_success_or_failure() {
    let mut state = LiveState::default();
    assert_eq!(state.status, ConnectionStatus::Connecting);
    state.dirty = false;
    state.daemon_starting();
    assert!(state.dirty);
    assert_eq!(state.status, ConnectionStatus::StartingDaemon);
    assert!(!state.status.is_connected());
    assert_eq!(
        state.status_text(Some("old input error"), Instant::now()),
        "Starting Herdr server..."
    );
    state.apply(ClientEvent::Snapshot(snapshot()));
    assert_eq!(state.status, ConnectionStatus::Connected);

    state.daemon_starting();
    state.apply(ClientEvent::Disconnected {
        reason: "startup failed".into(),
        ssh: None,
    });
    assert_eq!(state.status, ConnectionStatus::Disconnected);
    assert_eq!(state.notices.text(), Some("startup failed"));
}

#[test]
fn connection_status_and_error_priority_follow_lifecycle() {
    let mut state = LiveState::default();
    assert_eq!(state.status, ConnectionStatus::Connecting);
    assert!(!state.status.is_connected());
    assert_eq!(
        state.status_text(Some("old input error"), Instant::now()),
        "Connecting..."
    );
    state.apply(ClientEvent::Snapshot(snapshot()));
    assert!(state.status.is_connected());
    assert_eq!(
        state.status_text(Some("input error"), Instant::now()),
        "\u{2068}Connected\u{2069}: \u{2068}input error\u{2069}"
    );
    state.apply(ClientEvent::Disconnected {
        reason: "socket closed".into(),
        ssh: None,
    });
    assert!(!state.status.is_connected());
    assert_eq!(
        state.status_text(Some("old input error"), Instant::now()),
        "\u{2068}Disconnected\u{2069}: \u{2068}socket closed\u{2069}"
    );
    state.status = ConnectionStatus::Detached;
    state.notices.clear();
    assert!(!state.status.is_connected());
    assert_eq!(
        state.status_text(Some("old input error"), Instant::now()),
        "Detached (daemon still running)"
    );
    assert!(ConnectionStatus::AwaitingSnapshot.is_connected());
    assert_eq!(
        ConnectionStatus::AwaitingSnapshot.to_string(),
        "Connected; waiting for snapshot"
    );
}

#[test]
fn late_untracked_refusal_preserves_disconnection_reason() {
    let mut state = LiveState::default();
    state.apply(ClientEvent::Disconnected {
        reason: "socket closed".into(),
        ssh: None,
    });
    state.apply(ClientEvent::CommandRejected {
        request_id: None,
        reason: spaceport_client::Error::Disconnected,
    });
    assert_eq!(
        state.status_text(None, Instant::now()),
        "\u{2068}Disconnected\u{2069}: \u{2068}socket closed\u{2069}"
    );
}

// ABOUTME: Runs the compiled CLI against isolated scripted Unix socket peers.
// ABOUTME: Checks real disconnection, timeout and protocol mismatch exit codes without retrying actions.
use super::*;
use sheprd_control_protocol::protocol::{
    ErrorCode, PROTOCOL_VERSION, Response,
    app::{CoreCatalog, Status},
};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::mpsc,
};

enum Answer {
    Eof,
    Bytes(Vec<u8>),
    Silent,
}

#[path = "transport/socket_mode.rs"]
mod socket_mode;

fn prepare_peer(stream: &UnixStream) {
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
}

fn scripted(answer: Answer) -> Output {
    scripted_for(&["status", "--json"], "status", answer)
}

fn scripted_for(args: &[&str], method: &str, answer: Answer) -> Output {
    warm_up();
    let state = tempfile::Builder::new()
        .prefix("spcli")
        .tempdir_in("/tmp")
        .unwrap();
    let app_state = app_state_path(state.path());
    std::fs::create_dir(&app_state).unwrap();
    let listener = UnixListener::bind(app_state.join("control.sock")).unwrap();
    listener.set_nonblocking(true).unwrap();
    let (finished, done) = mpsc::channel();
    let method = method.to_owned();
    let peer = thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < until, "CLI never connected");
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        };
        prepare_peer(&stream);
        let mut request = String::new();
        BufReader::new(&stream).read_line(&mut request).unwrap();
        let request: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(request["method"], method);
        let silent = matches!(&answer, Answer::Silent);
        match answer {
            Answer::Eof => {}
            Answer::Bytes(bytes) => stream.write_all(&bytes).unwrap(),
            Answer::Silent => {
                done.recv_timeout(Duration::from_secs(20)).unwrap();
            }
        }
        drop(stream);
        if !silent {
            done.recv_timeout(Duration::from_secs(20)).unwrap();
        }
        // Keep listening through command exit to detect automatic retries.
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    });
    let child = Command::new(env!("CARGO_BIN_EXE_spaceport"))
        .args(args)
        .env("XDG_STATE_HOME", state.path())
        .env_remove("HERDR_SOCKET_PATH")
        .env_remove("HERDR_WORKSPACE_ID")
        .env_remove("HERDR_PANE_ID")
        .env_remove("HC_TERMINAL_ID")
        .env_remove("HC_TERMINAL_SOCKET")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = wait(child, args, Duration::from_secs(20));
    let _ = finished.send(());
    peer.join().unwrap();
    output
}

fn error(answer: Answer, status: i32, code: &str) {
    let output = scripted(answer);
    assert_eq!(output.status.code(), Some(status), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["error"]["code"], code, "{json}");
}

#[test]
fn eof_reports_that_the_app_is_gone() {
    error(Answer::Eof, 3, "not_running");
}

#[test]
fn an_unknown_response_reports_an_unsupported_protocol() {
    error(
        Answer::Bytes(b"{\"status\":\"future_response\"}\n".to_vec()),
        8,
        "unsupported",
    );
}

#[test]
fn a_wrong_response_kind_reports_an_unsupported_protocol() {
    let mut bytes = serde_json::to_vec(&Response::done("unexpected")).unwrap();
    bytes.push(b'\n');
    error(Answer::Bytes(bytes), 8, "unsupported");
}

#[test]
fn a_status_version_mismatch_reports_an_unsupported_protocol() {
    let mut bytes = serde_json::to_vec(&Response::Status(Status {
        version: "fixture".into(),
        protocol: PROTOCOL_VERSION + 1,
        pid: 1,
        windows: Vec::new(),
        core_catalog: CoreCatalog::default(),
    }))
    .unwrap();
    bytes.push(b'\n');
    error(Answer::Bytes(bytes), 8, "unsupported");
}

#[test]
fn a_silent_peer_reports_a_timeout() {
    error(Answer::Silent, 7, "timeout");
}

#[test]
fn chat_creation_prints_the_runtime_ids_without_promising_readiness() {
    use sheprd_control_protocol::protocol::chat::{Reply, Tab};
    for tab in [
        Tab::Herdr {
            tab_id: "w1:t9".into(),
        },
        Tab::Core {
            tab_id: 9,
            terminal_id: "t9".into(),
        },
    ] {
        let expected = serde_json::to_value(&tab).unwrap();
        let mut bytes = serde_json::to_vec(&Response::Chat {
            reply: Reply::Created {
                agent: "claude".into(),
                window_id: 7,
                tab,
                pane_id: "receipt-pane".into(),
            },
        })
        .unwrap();
        bytes.push(b'\n');
        let output = scripted_for(
            &["chat", "new", "claude", "--json"],
            "chat",
            Answer::Bytes(bytes),
        );
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"agent":"claude", "created":true, "window_id":7, "tab":expected, "pane_id":"receipt-pane"})
        );
    }
}

#[test]
fn a_non_utf8_caller_daemon_is_refused_before_connecting() {
    use std::os::unix::ffi::OsStrExt as _;
    warm_up();
    let state = tempfile::Builder::new()
        .prefix("spcli")
        .tempdir_in("/tmp")
        .unwrap();
    let app_state = app_state_path(state.path());
    std::fs::create_dir(&app_state).unwrap();
    let listener = UnixListener::bind(app_state.join("control.sock")).unwrap();
    listener.set_nonblocking(true).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_spaceport"))
        .args(["status", "--json"])
        .env("XDG_STATE_HOME", state.path())
        .env(
            "HERDR_SOCKET_PATH",
            std::ffi::OsStr::from_bytes(b"/tmp/caller-\xff.sock"),
        )
        .env("HERDR_WORKSPACE_ID", "w0")
        .env_remove("HERDR_PANE_ID")
        .env_remove("HC_TERMINAL_ID")
        .env_remove("HC_TERMINAL_SOCKET")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = wait(child, &["status", "--json"], COMMAND_LIMIT);
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["error"]["code"], "invalid_request");
    assert!(
        json["error"]["message"].as_str().unwrap().contains("UTF-8"),
        "{json}"
    );
}

#[test]
fn server_error_codes_survive_the_executable_json_boundary() {
    for (code, expected, status) in [
        (ErrorCode::WorkspaceNotFound, "workspace_not_found", 5),
        (ErrorCode::NoWindow, "no_window", 5),
        (ErrorCode::InvalidRequest, "invalid_request", 1),
        (ErrorCode::Failed, "failed", 1),
    ] {
        let mut bytes = serde_json::to_vec(&Response::error(code, "fixture refusal")).unwrap();
        bytes.push(b'\n');
        error(Answer::Bytes(bytes), status, expected);
    }
}

#[test]
fn empty_remote_lists_have_named_json_envelopes_and_exit_four() {
    use sheprd_control_protocol::protocol::{
        app::Reply as AppReply, hosts::Reply as HostReply, settings::Reply as SettingsReply,
        tabs::Reply as TabReply,
    };
    for (args, method, key, response) in [
        (
            &["font", "list", "--json"][..],
            "settings",
            "fonts",
            Response::Settings {
                reply: SettingsReply::Fonts { fonts: Vec::new() },
            },
        ),
        (
            &["integrations", "list", "--json"][..],
            "settings",
            "integrations",
            Response::Settings {
                reply: SettingsReply::Integrations {
                    integrations: Vec::new(),
                },
            },
        ),
        (
            &["tab", "list", "--json"][..],
            "tab",
            "tabs",
            Response::Tab {
                reply: TabReply::Tabs { tabs: Vec::new() },
            },
        ),
        (
            &["window", "list", "--json"][..],
            "window",
            "windows",
            Response::App {
                reply: AppReply::Windows {
                    windows: Vec::new(),
                },
            },
        ),
        (
            &["host", "list", "--json"][..],
            "host",
            "hosts",
            Response::Host {
                reply: HostReply::Hosts { hosts: Vec::new() },
            },
        ),
    ] {
        let mut bytes = serde_json::to_vec(&response).unwrap();
        bytes.push(b'\n');
        let output = scripted_for(args, method, Answer::Bytes(bytes));
        assert_eq!(output.status.code(), Some(4), "{output:?}");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
            serde_json::json!({key: []})
        );
    }
}

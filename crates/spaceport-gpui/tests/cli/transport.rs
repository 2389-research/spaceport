// ABOUTME: Runs the compiled CLI against isolated scripted Unix socket peers.
// ABOUTME: Checks real disconnection, timeout and protocol mismatch exit codes without retrying actions.
use super::*;
use spaceport_control_protocol::protocol::{
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

/// The peer reads its request on a blocking stream with a failure-only
/// deadline, so a CLI that connects and says nothing fails the test.
fn prepare_peer(stream: &UnixStream) {
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
}

/// Runs `status --json` against a peer that reads the request and answers
/// as scripted, then checks that the command never connects again.
fn scripted(answer: Answer) -> Output {
    warm_up();
    let state = tempfile::Builder::new()
        .prefix("spcli")
        .tempdir_in("/tmp")
        .unwrap();
    let app_state = state.path().join("spaceport");
    std::fs::create_dir(&app_state).unwrap();
    let listener = UnixListener::bind(app_state.join("control.sock")).unwrap();
    let (finished, done) = mpsc::channel();
    let peer = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        prepare_peer(&stream);
        let mut request = String::new();
        BufReader::new(&stream).read_line(&mut request).unwrap();
        let request: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(request["method"], "status");
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
        listener.set_nonblocking(true).unwrap();
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    });
    let args = ["status", "--json"];
    let child = Command::new(env!("CARGO_BIN_EXE_spaceport"))
        .args(args)
        .env("XDG_STATE_HOME", state.path())
        .env_remove("HERDR_SOCKET_PATH")
        .env_remove("HERDR_CLIENT_SOCKET_PATH")
        .env_remove("HERDR_WORKSPACE_ID")
        .env_remove("HERDR_PANE_ID")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = wait(child, &args, Duration::from_secs(20));
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

fn line(response: &Response) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(response).unwrap();
    bytes.push(b'\n');
    bytes
}

#[test]
fn eof_reports_that_the_app_is_gone() {
    error(Answer::Eof, 3, "not_running");
}

#[test]
fn an_answer_that_is_not_json_reports_an_unsupported_protocol() {
    error(Answer::Bytes(b"not json\n".to_vec()), 8, "unsupported");
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
    error(
        Answer::Bytes(line(&Response::done("unexpected"))),
        8,
        "unsupported",
    );
}

#[test]
fn a_status_version_mismatch_reports_an_unsupported_protocol() {
    let status = Response::Status(Status {
        version: "fixture".into(),
        protocol: PROTOCOL_VERSION + 1,
        pid: 1,
        windows: Vec::new(),
        core_catalog: CoreCatalog::default(),
    });
    error(Answer::Bytes(line(&status)), 8, "unsupported");
}

#[test]
fn a_silent_peer_reports_a_timeout() {
    error(Answer::Silent, 7, "timeout");
}

#[test]
fn server_error_codes_survive_the_executable_json_boundary() {
    for (code, expected, status) in [
        (ErrorCode::WorkspaceNotFound, "workspace_not_found", 5),
        (ErrorCode::NoWindow, "no_window", 5),
        (ErrorCode::InvalidRequest, "invalid_request", 1),
        (ErrorCode::Failed, "failed", 1),
    ] {
        let refusal = Response::error(code, "fixture refusal");
        error(Answer::Bytes(line(&refusal)), status, expected);
    }
}

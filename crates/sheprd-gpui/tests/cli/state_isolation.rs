// ABOUTME: Checks build-specific state paths through the actual CLI executable.
// ABOUTME: Keeps development stores and control endpoints away from installed previews.
use super::*;
use std::path::Path;

#[test]
fn development_log_paths_do_not_reuse_the_installed_preview_state() {
    let state = tempfile::tempdir().unwrap();
    let output = cli_in(&["logs", "--path", "--json"], Some(state.path()));
    assert!(output.status.success(), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let path = Path::new(json["path"].as_str().unwrap());
    assert_eq!(
        path,
        sheprd_diagnostics::log_file(&app_state_path(state.path()))
    );
    let preview = state.path().join("spaceport");
    match env!("SPACEPORT_BUILD_CHANNEL") {
        "development" => assert!(
            !path.starts_with(preview),
            "development logs use preview state: {path:?}"
        ),
        "macos-preview" => assert!(path.starts_with(preview)),
        channel => panic!("unknown build channel {channel}"),
    }
}

#[test]
#[ignore = "isolated diagnostics producer subprocess"]
fn diagnostics_producer() {
    let state = std::env::var_os("SPACEPORT_TEST_STATE").unwrap();
    let state = Path::new(&state);
    sheprd_diagnostics::init(
        sheprd_diagnostics::Targets {
            crate_prefix: "",
            roots: &["state_isolation_fixture"],
        },
        Some(state),
    )
    .unwrap();
    let message = std::env::var("SPACEPORT_TEST_MESSAGE").unwrap();
    tracing::warn!(target: "state_isolation_fixture", "{message}");
    assert!(sheprd_diagnostics::flush(Duration::from_secs(2)));
}

fn write_log(state: &Path, message: &str) {
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "state_isolation::diagnostics_producer",
            "--ignored",
            "--nocapture",
        ])
        .env("SPACEPORT_TEST_STATE", state)
        .env("SPACEPORT_TEST_MESSAGE", message)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = wait(child, &["diagnostics fixture"], COMMAND_LIMIT);
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn the_cli_reads_the_selected_diagnostics_writer_without_changing_preview_logs() {
    let state = tempfile::tempdir().unwrap();
    let preview = state.path().join("spaceport");
    let selected = app_state_path(state.path());
    write_log(&preview, "preview-state-record");
    let preview_log = sheprd_diagnostics::log_file(&preview);
    let before = std::fs::read(&preview_log).unwrap();
    if selected != preview {
        write_log(&selected, "development-state-record");
    }
    let output = cli_in(&["logs", "--json"], Some(state.path()));
    assert!(output.status.success(), "{output:?}");
    let records: Vec<sheprd_diagnostics::Record> = serde_json::from_slice(&output.stdout).unwrap();
    let expected = if selected == preview {
        "preview-state-record"
    } else {
        "development-state-record"
    };
    assert!(records.iter().any(|record| record.message == expected));
    if selected != preview {
        assert!(
            !records
                .iter()
                .any(|record| record.message == "preview-state-record")
        );
    }
    assert_eq!(std::fs::read(&preview_log).unwrap(), before);
    assert_eq!(
        sheprd_diagnostics::crash::dir(&selected).unwrap(),
        sheprd_diagnostics::log_file(&selected).parent().unwrap()
    );
}

#[cfg(unix)]
#[test]
fn status_reaches_a_preview_listener_only_for_a_preview_build() {
    use sheprd_control_protocol::protocol::{
        PROTOCOL_VERSION, Response,
        app::{CoreCatalog, Status},
    };
    use std::{
        io::{BufRead, BufReader, Read, Write},
        os::unix::net::UnixListener,
    };

    warm_up();
    let state = tempfile::Builder::new()
        .prefix("sp-state")
        .tempdir_in("/tmp")
        .unwrap();
    let preview = state.path().join("spaceport");
    std::fs::create_dir(&preview).unwrap();
    let listener = UnixListener::bind(preview.join("control.sock")).unwrap();
    listener.set_nonblocking(true).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_spaceport"))
        .args(["status", "--json"])
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
    // A bounded peer wait runs while the CLI executes; a development CLI must not connect.
    let (stop, stopped) = std::sync::mpsc::channel();
    let peer = thread::spawn(move || {
        let mut connected = false;
        loop {
            match stopped.try_recv() {
                Ok(()) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    connected = true;
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut request = String::new();
                    let read = BufReader::new(
                        (&stream).take(sheprd_control_protocol::protocol::MAX_MESSAGE as u64 + 1),
                    )
                    .read_line(&mut request)
                    .unwrap();
                    assert!(read <= sheprd_control_protocol::protocol::MAX_MESSAGE);
                    assert!(request.ends_with('\n'));
                    let request: serde_json::Value = serde_json::from_str(&request).unwrap();
                    assert_eq!(request["method"], "status");
                    stream
                        .set_write_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut bytes = serde_json::to_vec(&Response::Status(Status {
                        version: "preview-fixture".into(),
                        protocol: PROTOCOL_VERSION,
                        pid: 1,
                        windows: Vec::new(),
                        core_catalog: CoreCatalog::default(),
                    }))
                    .unwrap();
                    bytes.push(b'\n');
                    stream.write_all(&bytes).unwrap();
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5))
                }
                Err(error) => panic!("preview listener failed: {error}"),
            }
        }
        connected
    });
    let output = wait(child, &["status", "--json"], COMMAND_LIMIT);
    stop.send(()).unwrap();
    let connected = peer.join().unwrap();
    let is_preview = env!("SPACEPORT_BUILD_CHANNEL") == "macos-preview";
    assert_eq!(
        connected, is_preview,
        "CLI reached the wrong build's endpoint"
    );
    assert_eq!(
        output.status.code(),
        Some(if is_preview { 0 } else { 3 }),
        "{output:?}"
    );
}

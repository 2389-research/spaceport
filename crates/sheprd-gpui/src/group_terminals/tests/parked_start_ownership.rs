// ABOUTME: Exercises parked startup ownership in a private test subprocess.
// ABOUTME: Uses an owned executable and socket to exclude personal daemon discovery.
use super::*;
use sheprd_client::ConnectTarget;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Child, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

const CHILD_ROOT: &str = "SPACEPORT_TEST_PARKED_START_ROOT";
const TEST: &str =
    "group_terminals::tests::parked_start_ownership::parked_groups_never_start_the_local_daemon";

struct TestChild(Child);

impl Drop for TestChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn run_isolated() {
    let root = tempfile::Builder::new().prefix("p").tempdir().unwrap();
    let fake = root.path().join("herdr");
    crate::test_executable::write(
        &fake,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"${0%/*}/started\"\n",
        0o700,
    )
    .unwrap();
    let log_path = root.path().join("child.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut command = sheprd_client::new_command(std::env::current_exe().unwrap());
    command
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env_clear()
        .env(CHILD_ROOT, root.path())
        .env("PATH", root.path())
        .env("HOME", root.path())
        .env("XDG_CONFIG_HOME", root.path().join("config"))
        .env("XDG_STATE_HOME", root.path().join("state"))
        .env("XDG_DATA_HOME", root.path().join("data"))
        .env("XDG_CACHE_HOME", root.path().join("cache"))
        .env("XDG_RUNTIME_DIR", root.path())
        .env("TMPDIR", root.path())
        .env("HERDR_CLIENT_SOCKET_PATH", root.path().join("s"))
        .env("SHELL", root.path().join("missing-shell"))
        .current_dir(root.path())
        .stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log);
    // The all-feature macOS test binary links the pinned CI framework.
    #[cfg(target_os = "macos")]
    if let Some(framework) = std::env::var_os("DYLD_FRAMEWORK_PATH") {
        command.env("DYLD_FRAMEWORK_PATH", framework);
    }
    let mut child = TestChild(command.spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "isolated startup test timed out");
        thread::sleep(Duration::from_millis(10));
    };
    let output = fs::read_to_string(log_path).unwrap();
    assert!(status.success(), "isolated startup test failed:\n{output}");
    assert!(
        root.path().join("checked").is_file(),
        "child test did not run"
    );
}

#[gpui::test]
fn parked_groups_never_start_the_local_daemon(cx: &mut TestAppContext) {
    let Some(root) = std::env::var_os(CHILD_ROOT).map(PathBuf::from) else {
        run_isolated();
        return;
    };
    let fake = root.join("herdr");
    let socket = root.join("s");
    let marker = root.join("started");
    assert!(root.is_absolute() && root.is_dir());
    assert!(fake.is_file() && fs::metadata(&fake).unwrap().permissions().mode() & 0o111 != 0);
    assert_eq!(crate::daemon::executable(), fake);
    assert_eq!(ConnectTarget::Local.socket_path().unwrap(), socket);
    assert!(!socket.exists());
    let stopped = AtomicBool::new(false);
    let error = crate::daemon::connect(&ConnectTarget::Local, Start::WhenMissing, &stopped, || {})
        .err()
        .unwrap();
    assert!(matches!(
        error.get_ref().and_then(|source| source.downcast_ref::<crate::daemon::Error>()),
        Some(crate::daemon::Error::Exited(status)) if status.success()
    ));
    assert_eq!(fs::read_to_string(&marker).unwrap(), "server\n");
    fs::remove_file(&marker).unwrap();

    let peer = MockPeer::new();
    let (view, cx) = fixture_window_on_t0(cx);
    let drained = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let left = parked_startup::prepare_groups(view, window, cx, &socket, &peer);
            let key = view.browser_key().unwrap();
            view.endpoints[0].connection.target = ConnectTarget::Local;
            let same_scope = view.browser_key() == Some(key);
            if same_scope {
                view.reconcile_group_terminals(cx);
            }
            // Restore before assertions or deferred GPUI work, including on the old code.
            view.endpoints[0].connection.target = ConnectTarget::Socket(socket.clone());
            assert!(same_scope, "the private Local and Socket scopes must match");
            let parked = view
                .group_terminals
                .parked
                .iter()
                .find(|parked| parked.group == left)
                .unwrap();
            assert!(matches!(&parked.connection.target, ConnectTarget::Local));
            assert!(
                parked.connection.handle.is_some(),
                "parked attachment must run"
            );
            parked.connection.drained.clone()
        })
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    while !drained.load(Ordering::Acquire) {
        assert!(Instant::now() < deadline, "parked reader did not drain");
        thread::sleep(Duration::from_millis(1));
    }
    assert!(
        !marker.exists(),
        "parked connection started the owned fake Herdr"
    );
    fs::write(root.join("checked"), "done").unwrap();
}

// ABOUTME: Qualifies native Spaceport quit and restart against a private Herdr daemon.
// ABOUTME: Proves the same terminal shell keeps running while the GUI is absent.
#![cfg(all(unix, feature = "integration-test"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)]

#[path = "../../test-support/sandbox.rs"]
mod sandbox;

use rustix::process::{Pid, test_kill_process};
use sandbox::{Sandbox, daemon_binary, stop_children};
use sheprd_client::{
    Client, ClientEvent, ConnectOptions, ConnectTarget, connect,
    protocol::{ClientKeyCode, ClientKeyKind, ClientPaneInputEvent, ClientShellSnapshot},
};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    thread,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(30);
const POLL: Duration = Duration::from_millis(50);
const LOG_LIMIT: u64 = 24 * 1024;

struct Fixture {
    sandbox: Sandbox,
    daemon: Option<Child>,
    gui: Option<Child>,
    shell: Option<Pid>,
}

struct Observer {
    client: Client,
    snapshot: ClientShellSnapshot,
}

impl Observer {
    fn open(socket: PathBuf) -> Self {
        let client = connect(ConnectTarget::Socket(socket), ConnectOptions::default()).unwrap();
        let deadline = Instant::now() + TIMEOUT;
        let snapshot = loop {
            let event = client
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("private daemon did not supply a snapshot");
            match event {
                ClientEvent::Snapshot(snapshot) => break (*snapshot).clone(),
                ClientEvent::Disconnected { reason, .. } => panic!("private client: {reason}"),
                ClientEvent::CommandRejected { reason, .. } => panic!("private input: {reason}"),
                _ => {}
            }
            assert!(Instant::now() < deadline, "private snapshot timed out");
        };
        assert!(snapshot.config_diagnostic.is_none(), "{snapshot:?}");
        Self { client, snapshot }
    }

    fn run(&self, command: String) {
        self.client
            .handle
            .send_input(
                &self.snapshot.boot_id,
                self.snapshot.focused_pane_id.as_deref().unwrap(),
                vec![
                    ClientPaneInputEvent::TextCommit(command),
                    ClientPaneInputEvent::Key {
                        code: ClientKeyCode::Enter,
                        modifiers: 0,
                        kind: ClientKeyKind::Press,
                        repeat_count: 1,
                        shifted_codepoint: None,
                        generated_text: None,
                        tracks_release: false,
                        physical_key_id: None,
                        windows_record: None,
                    },
                ],
            )
            .unwrap();
    }
}

impl Drop for Observer {
    fn drop(&mut self) {
        self.client.handle.disconnect();
        let deadline = Instant::now() + TIMEOUT;
        // Drain until the worker closes, so an observer cannot hide a last-client bug.
        loop {
            match self
                .client
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Err(error) if error.is_disconnected() => break,
                Err(error) => {
                    if thread::panicking() {
                        eprintln!("observer detach during failure: {error}");
                        break;
                    }
                    panic!("observer failed to detach: {error}");
                }
                Ok(_) => {}
            }
        }
    }
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_str().unwrap().replace('\'', "'\"'\"'"))
}

fn bounded_log(path: &Path) -> String {
    let Ok(mut file) = File::open(path) else {
        return String::new();
    };
    let length = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    let _ = file.seek(SeekFrom::Start(length.saturating_sub(LOG_LIMIT)));
    let mut bytes = Vec::new();
    let _ = file.take(LOG_LIMIT).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

fn wait_exit(child: &mut Child, timeout: Duration) -> Option<ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return Some(status);
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(POLL);
    }
}

impl Fixture {
    fn new() -> Self {
        Self {
            sandbox: Sandbox::new(),
            daemon: None,
            gui: None,
            shell: None,
        }
    }

    fn gui_command(&self, log: &str) -> Command {
        let mut command = self.sandbox.command(env!("CARGO_BIN_EXE_spaceport"), log);
        for name in ["DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY"] {
            if let Some(mut value) = std::env::var_os(name) {
                if name == "WAYLAND_DISPLAY"
                    && !value.is_empty()
                    && Path::new(&value).is_relative()
                    && let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR")
                        .filter(|dir| Path::new(dir).is_absolute())
                {
                    value = Path::new(&runtime).join(value).into_os_string();
                }
                command.env(name, value);
            }
        }
        command.arg("--socket").arg(self.sandbox.socket());
        command
    }

    fn cli(&self, args: &[&str], log: &str) -> (ExitStatus, String) {
        let mut child = Some(
            self.sandbox
                .command(env!("CARGO_BIN_EXE_spaceport"), log)
                .args(args)
                .spawn()
                .unwrap(),
        );
        let status = wait_exit(child.as_mut().unwrap(), Duration::from_secs(10));
        stop_children([&mut child]);
        let output = bounded_log(&self.sandbox.dir.join(log));
        (
            status.unwrap_or_else(|| panic!("CLI timed out: {args:?}: {output}")),
            output,
        )
    }

    fn launch(&mut self, generation: usize, workspace: &str) -> u64 {
        let log = format!("gui-{generation}.log");
        self.gui = Some(self.gui_command(&log).spawn().unwrap());
        let deadline = Instant::now() + TIMEOUT;
        loop {
            assert!(
                self.gui.as_mut().unwrap().try_wait().unwrap().is_none(),
                "native GUI exited during startup: {}",
                bounded_log(&self.sandbox.dir.join(&log))
            );
            let (status, output) = self.cli(&["window", "list", "--json"], "window-list.log");
            if status.success() {
                let windows: serde_json::Value = serde_json::from_str(&output).unwrap();
                let windows = windows
                    .as_array()
                    .expect("window list must return an array");
                assert!(windows.len() <= 1, "private GUI opened unexpected windows");
                if let Some(window) = windows.first()
                    && window["workspace"].as_str() == Some(workspace)
                    && window["hosts"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|host| host["selected"] == true && host["status"] == "online")
                {
                    return window["id"].as_u64().unwrap();
                }
            }
            assert!(
                Instant::now() < deadline,
                "GUI readiness timed out: {output}"
            );
            thread::sleep(POLL);
        }
    }

    fn quit(&mut self, window: u64) {
        let window = window.to_string();
        // Submit exactly once: a timeout can mean the app accepted the quit.
        let (status, output) = self.cli(
            &["app", "quit", "--json", "--window", &window],
            "app-quit.log",
        );
        assert!(status.success(), "targeted quit failed: {status}: {output}");
        let status = wait_exit(self.gui.as_mut().unwrap(), TIMEOUT).expect("GUI did not quit");
        assert!(status.success(), "GUI quit failed: {status}");
        self.gui.take();
    }

    fn heartbeat(&self) -> Option<(i32, u64)> {
        let text = fs::read_to_string(self.sandbox.dir.join("heartbeat")).ok()?;
        let (pid, counter) = text.trim().split_once(' ')?;
        Some((pid.parse().ok()?, counter.parse().ok()?))
    }

    fn wait_heartbeat(&self, previous: u64) -> (i32, u64) {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some((pid, counter)) = self.heartbeat()
                && counter > previous
            {
                assert!(test_kill_process(Pid::from_raw(pid).unwrap()).is_ok());
                return (pid, counter);
            }
            assert!(Instant::now() < deadline, "foreground heartbeat stopped");
            thread::sleep(POLL);
        }
    }

    fn assert_daemon(&mut self, pid: u32) {
        let daemon = self.daemon.as_mut().unwrap();
        assert_eq!(daemon.id(), pid);
        assert!(
            daemon.try_wait().unwrap().is_none(),
            "GUI exit killed daemon"
        );
    }

    fn stop_owned_children(&mut self) -> bool {
        // Closing the owned daemon's PTY sends hangup to its foreground shell.
        // Never signal a PID obtained from a file or a process-list observation.
        stop_children([&mut self.gui, &mut self.daemon]);
        self.wait_shell_exit()
    }

    fn stop_shell(&self) -> bool {
        // The foreground loop owns no background children; it waits for its current
        // sleep, observes this private sentinel, then exits its own terminal shell.
        let _ = fs::write(self.sandbox.dir.join("stop"), "stop\n");
        self.wait_shell_exit()
    }

    fn wait_shell_exit(&self) -> bool {
        let Some(shell) = self.shell else {
            return true;
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        while test_kill_process(shell).is_ok() {
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(POLL);
        }
        true
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.stop_shell() {
            eprintln!("fixture terminal shell did not exit after private stop sentinel");
        }
        let cleaned = self.stop_owned_children();
        if !cleaned {
            eprintln!(
                "FAILED CLEANUP: fixture terminal shell survived owned daemon exit; refusing PID signals"
            );
        }
        if thread::panicking() || !cleaned {
            let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/sparkle-lifecycle")
                .join(format!("sparkle-lifecycle-{}", std::process::id()));
            let _ = fs::create_dir_all(&evidence);
            for name in [
                "gui-1.log",
                "gui-2.log",
                "daemon.log",
                "window-list.log",
                "app-quit.log",
                "config/herdr/herdr-server.log",
            ] {
                let log = bounded_log(&self.sandbox.dir.join(name));
                if !log.is_empty() {
                    let _ = fs::write(evidence.join(name.replace('/', "-")), &log);
                    eprintln!("{name} (bounded tail):\n{log}");
                }
            }
            eprintln!("bounded lifecycle failure evidence: {}", evidence.display());
        }
    }
}

#[test]
#[ignore = "requires active native desktop and explicit HERDR_TEST_BINARY; starts a private Herdr server and two real Spaceport GUIs"]
fn native_quit_restart_preserves_daemon_and_terminal() {
    let mut fixture = Fixture::new();
    let app_config = fixture.sandbox.dir.join("config/spaceport");
    fs::create_dir_all(&app_config).unwrap();
    fs::write(
        app_config.join("view.toml"),
        "[ui]\nprompt_new_tab_name = false\nprompt_new_workspace_name = false\n",
    )
    .unwrap();
    fixture.daemon = Some(
        fixture
            .sandbox
            .command(daemon_binary(), "daemon.log")
            .arg("server")
            .spawn()
            .unwrap(),
    );
    fixture
        .sandbox
        .wait_for_daemon(fixture.daemon.as_mut().unwrap(), TIMEOUT);
    let daemon_pid = fixture.daemon.as_ref().unwrap().id();
    let observer = Observer::open(fixture.sandbox.socket());
    let boot = observer.snapshot.boot_id.clone();
    let pane = observer.snapshot.focused_pane_id.clone().unwrap();
    let workspace = observer.snapshot.focused_workspace_id.clone().unwrap();
    let heartbeat = fixture.sandbox.dir.join("heartbeat");
    let temporary = fixture.sandbox.dir.join("heartbeat-next");
    let stop = fixture.sandbox.dir.join("stop");
    let script = fixture.sandbox.dir.join("heartbeat.sh");
    fs::write(&script, format!(
        "# ABOUTME: Runs the private foreground terminal heartbeat for lifecycle qualification.\n# ABOUTME: Exits the terminal shell after the private stop sentinel appears.\nn=0; while [ ! -e {} ]; do n=$((n+1)); printf '%s %s\\n' \"$$\" \"$n\" > {}; mv {} {}; sleep 0.1; done; exit\n",
        shell_quote(&stop), shell_quote(&temporary), shell_quote(&temporary), shell_quote(&heartbeat),
    )).unwrap();
    observer.run(format!("exec /bin/sh {}", shell_quote(&script)));
    let (shell_pid, initial) = fixture.wait_heartbeat(0);
    fixture.shell = Pid::from_raw(shell_pid);
    drop(observer);

    let first_window = fixture.launch(1, &workspace);
    let before_quit = fixture.wait_heartbeat(initial).1;
    fixture.quit(first_window);
    fixture.assert_daemon(daemon_pid);
    let first_exit_baseline = fixture.heartbeat().unwrap().1;
    assert!(first_exit_baseline >= before_quit);
    let (after_shell, first_exit) = fixture.wait_heartbeat(first_exit_baseline);
    assert_eq!(after_shell, shell_pid, "GUI quit replaced terminal shell");

    let second_window = fixture.launch(2, &workspace);
    let observer = Observer::open(fixture.sandbox.socket());
    assert_eq!(observer.snapshot.boot_id, boot, "daemon boot changed");
    assert_eq!(
        observer.snapshot.focused_pane_id.as_deref(),
        Some(pane.as_str())
    );
    assert_eq!(
        observer.snapshot.focused_workspace_id.as_deref(),
        Some(workspace.as_str())
    );
    let (restarted_shell, restarted) = fixture.wait_heartbeat(first_exit);
    assert_eq!(
        restarted_shell, shell_pid,
        "restart replaced terminal shell"
    );
    drop(observer);
    fixture.quit(second_window);
    fixture.assert_daemon(daemon_pid);
    let second_exit_baseline = fixture.heartbeat().unwrap().1;
    assert!(second_exit_baseline >= restarted);
    let (final_shell, second_exit) = fixture.wait_heartbeat(second_exit_baseline);
    assert_eq!(
        final_shell, shell_pid,
        "second quit replaced terminal shell"
    );
    assert!(
        fixture.stop_shell(),
        "fixture terminal shell did not exit cleanly"
    );
    eprintln!(
        "LIFECYCLE native PASS: daemon_pid={daemon_pid} boot={boot} pane={pane} shell_pid={shell_pid} heartbeat={initial}<{before_quit}<{first_exit}<{restarted}<{second_exit}; two targeted quits and restart; observer detached before quits; terminal cleanup verified"
    );
}

#[test]
fn heartbeat_paths_are_shell_quoted() {
    let sandbox = Sandbox::new();
    let path = sandbox.dir.join("one two/it's; touch escaped");
    let mut child = Some(
        sandbox
            .command("/bin/sh", "quote.log")
            .args(["-c", &format!("printf '%s' {}", shell_quote(&path))])
            .spawn()
            .unwrap(),
    );
    let status = wait_exit(child.as_mut().unwrap(), Duration::from_secs(2));
    stop_children([&mut child]);
    assert!(status.unwrap().success());
    assert_eq!(
        bounded_log(&sandbox.dir.join("quote.log")),
        path.to_str().unwrap()
    );
    assert!(!sandbox.dir.join("escaped").exists());
}

#[test]
#[ignore = "requires explicit HERDR_TEST_BINARY; verifies terminal hangup after stopping only its exact owned daemon child"]
fn failed_fixture_cleanup_stops_shell_without_pid_signals() {
    let mut fixture = Fixture::new();
    fixture.daemon = Some(
        fixture
            .sandbox
            .command(daemon_binary(), "daemon.log")
            .arg("server")
            .spawn()
            .unwrap(),
    );
    fixture
        .sandbox
        .wait_for_daemon(fixture.daemon.as_mut().unwrap(), TIMEOUT);
    let observer = Observer::open(fixture.sandbox.socket());
    let script = fixture.sandbox.dir.join("heartbeat.sh");
    let heartbeat = fixture.sandbox.dir.join("heartbeat");
    // Deliberately omit the stop sentinel to exercise failed-fixture cleanup.
    fs::write(&script, format!(
        "# ABOUTME: Exercises terminal hangup when only the owned daemon child stops.\n# ABOUTME: Remains in the foreground until its private PTY closes.\nprintf '%s 1\\n' \"$$\" > {}; while :; do sleep 0.1; done\n",
        shell_quote(&heartbeat),
    )).unwrap();
    observer.run(format!("exec /bin/sh {}", shell_quote(&script)));
    let (shell, _) = fixture.wait_heartbeat(0);
    fixture.shell = Pid::from_raw(shell);
    drop(observer);
    assert!(test_kill_process(fixture.shell.unwrap()).is_ok());
    assert!(!fixture.sandbox.dir.join("stop").exists());
    assert!(
        fixture.stop_owned_children(),
        "shell survived owned daemon PTY hangup"
    );
    eprintln!(
        "LIFECYCLE cleanup PASS: shell_pid={shell}; no sentinel, no PID signals; exact owned daemon child stopped and shell exited"
    );
}

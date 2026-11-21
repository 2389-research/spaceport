// ABOUTME: Qualifies intentional Herdr shutdown and explicit reconnect with the normal Local window.
// ABOUTME: Owns both server processes and captures evidence on a fresh private X11 display.
use super::*;
use rustix::process::{Pid, Signal, kill_process};
use sha2::{Digest, Sha256};
use sheprd_control_protocol::protocol::app::{CoreCatalogState, Status, WindowInfo};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

#[path = "shutdown/display.rs"]
mod display;
#[path = "../../../test-support/executable.rs"]
mod executable;
#[path = "shutdown/witness.rs"]
mod witness;

const TIMEOUT: Duration = Duration::from_secs(30);
const POLL: Duration = Duration::from_millis(100);
// endpoint::MAX_RETRY_DELAY plus a polling margin.
const HOLD: Duration = Duration::from_secs(32);
const DIGEST: &str = "14d9e8bc8b4a9e0118d77c32266f76d461f09b40342eb23752960bd89f59929d";
const SERVER_LOG: &str = "herdr/herdr-server.log";
const REPLACEMENT_PROMPT: &str = "RECOVERY READY";

fn wait(child: &mut Child, operation: &str) -> ExitStatus {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("timed out waiting for {operation}");
        }
        thread::sleep(POLL);
    }
}

fn run(sandbox: &Sandbox, command: &mut Command) -> (ExitStatus, String) {
    let stdout = sandbox.dir.join("command.out");
    let mut child = command
        .stdout(File::create(&stdout).unwrap())
        .spawn()
        .unwrap();
    let status = wait(&mut child, "private fixture command");
    assert!(
        fs::metadata(&stdout).unwrap().len() <= 1024 * 1024,
        "fixture command exceeded its output limit"
    );
    let output = fs::read_to_string(stdout).unwrap();
    (status, output)
}

// Keep only the bounded tail of this fixture's named logs, including on failure.
fn log_tail(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    file.seek(SeekFrom::Start(size.saturating_sub(64 * 1024)))?;
    let mut bytes = Vec::new();
    file.take(64 * 1024).read_to_end(&mut bytes)?;
    Ok(fixture_log_tail(&String::from_utf8_lossy(&bytes)))
}

struct Qualification {
    isolated: Isolated,
    evidence: PathBuf,
    window: Option<u64>,
}

impl Drop for Qualification {
    fn drop(&mut self) {
        // Preserve a failed attempt before Isolated reaps its children and removes its sandbox.
        for server in ["first", "second"] {
            if let Ok(tail) = log_tail(&self.isolated.sandbox.dir.join(server).join(SERVER_LOG)) {
                let _ = fs::write(self.evidence.join(format!("{server}-server.log")), tail);
            }
        }
        for name in [
            "gui.log",
            "first.log",
            "second.log",
            "control.log",
            "xvfb.log",
            "capture.log",
            "ocr.log",
            "state/spaceport/logs/spaceport.jsonl",
            "started",
        ] {
            if let Ok(tail) = log_tail(&self.isolated.sandbox.dir.join(name)) {
                let _ = fs::write(
                    self.evidence.join(Path::new(name).file_name().unwrap()),
                    tail,
                );
            }
        }
    }
}

impl Qualification {
    fn record(&self, name: &str, text: impl AsRef<[u8]>) {
        fs::write(self.evidence.join(name), text).unwrap();
    }

    fn start_server(&mut self, binary: &Path, label: &str, prompt: &str) -> witness::Witness {
        let owned = &mut self.isolated;
        assert!(owned.daemon.is_none());
        assert!(!owned.sandbox.socket().exists());
        owned.daemon = Some(
            owned
                .sandbox
                .command(binary, format!("{label}.log"))
                // Separate restore state gives the replacement its own terminal and prompt.
                .env("XDG_CONFIG_HOME", owned.sandbox.dir.join(label))
                .env(
                    "XDG_STATE_HOME",
                    owned.sandbox.dir.join(label).join("state"),
                )
                .env("HERDR_STARTUP_CWD", &owned.sandbox.dir)
                .env("PS1", format!("{prompt}> "))
                .arg("server")
                .spawn()
                .unwrap(),
        );
        owned
            .sandbox
            .wait_for_daemon(owned.daemon.as_mut().unwrap(), TIMEOUT);
        witness::Witness::open(owned.sandbox.socket(), owned.daemon.as_ref().unwrap().id())
    }

    fn stop_server(&mut self, label: &str) {
        let child = self.isolated.daemon.as_mut().unwrap();
        assert!(
            child.try_wait().unwrap().is_none(),
            "{label} already exited"
        );
        kill_process(Pid::from_child(child), Signal::TERM).unwrap();
        let status = wait(child, label);
        assert!(status.success(), "{label} terminated with {status}");
        self.isolated.daemon = None;
        let path = self.isolated.sandbox.dir.join(label).join(SERVER_LOG);
        let log = log_tail(&path).unwrap();
        self.record(&format!("{label}-server.log"), &log);
        assert!(
            log.lines()
                .any(|line| line.contains("server shutdown initiated") && line.contains("SIGTERM")),
            "{label} did not record its TERM"
        );
        assert!(!self.isolated.sandbox.socket().exists());
    }

    fn check_gui(&mut self) {
        assert!(
            self.isolated
                .gui
                .as_mut()
                .unwrap()
                .try_wait()
                .unwrap()
                .is_none(),
            "GUI exited"
        );
        assert!(
            !self.isolated.sandbox.dir.join("started").exists(),
            "Spaceport attempted to start Herdr"
        );
    }

    fn cli(&self, args: &[&str]) -> (ExitStatus, String) {
        let sandbox = &self.isolated.sandbox;
        let mut command = sandbox.command(env!("CARGO_BIN_EXE_spaceport"), "control.log");
        command.args(args).arg("--json");
        if let Some(id) = self.window {
            command.args(["--window", &id.to_string()]);
        }
        run(sandbox, &mut command)
    }

    fn action(&self, args: &[&str]) {
        let (status, output) = self.cli(args);
        assert!(status.success(), "{args:?} failed: {output}");
    }

    fn state(&mut self) -> Option<WindowInfo> {
        self.check_gui();
        let (code, output) = self.cli(&["status"]);
        if self.window.is_none() && code.code() == Some(3) {
            return None;
        }
        assert!(code.success(), "status failed: {output}");
        let status: Status = serde_json::from_str(&output).unwrap();
        assert_eq!(status.pid, self.isolated.gui.as_ref().unwrap().id());
        assert_eq!(status.core_catalog.state, CoreCatalogState::NotStarted);
        if self.window.is_none() && status.windows.is_empty() {
            return None;
        }
        let [window]: [WindowInfo; 1] = status
            .windows
            .try_into()
            .expect("expected exactly one private GUI window");
        if let Some(id) = self.window {
            assert_eq!(window.id, id);
        }
        self.window = Some(window.id);
        self.record("latest-status.json", output);
        Some(window)
    }

    fn await_state(
        &mut self,
        phase: &str,
        workspace: Option<&str>,
        width: Option<u32>,
        label: &str,
    ) {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(window) = self.state()
                && window
                    .hosts
                    .iter()
                    .any(|h| h.selected && h.id == "local" && h.status == phase)
                && window.workspace.as_deref() == workspace
                && width.is_none_or(|width| (window.width, window.height) == (width as f32, 600.0))
            {
                fs::copy(
                    self.evidence.join("latest-status.json"),
                    self.evidence.join(format!("{label}.json")),
                )
                .unwrap();
                return;
            }
            assert!(Instant::now() < deadline, "waiting for {label}");
            thread::sleep(POLL);
        }
    }
}

#[test]
#[ignore = "requires pinned Linux x64 Herdr, Xvfb, capture tools and a fresh evidence directory"]
fn native_local_shutdown_and_reconnect() {
    let binary = daemon_binary();
    let mut file = File::open(&binary).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 16384];
    loop {
        let count = file.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    let digest: String = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(digest, DIGEST, "expected the Linux x86_64 v0.9.5 release");
    let evidence = PathBuf::from(
        std::env::var_os("SPACEPORT_SHUTDOWN_EVIDENCE").expect("set private evidence path"),
    );
    assert!(evidence.is_absolute());
    fs::create_dir(&evidence).expect("evidence directory must be new");
    let mut test = Qualification {
        isolated: Isolated {
            sandbox: Sandbox::new(),
            daemon: None,
            gui: None,
        },
        evidence,
        window: None,
    };
    test.record("herdr.sha256", format!("{digest}  herdr-linux-x86_64\n"));
    let desktop = display::Display::start(&test.isolated.sandbox);
    let first = test.start_server(&binary, "first", "INITIAL SHELL");
    test.record("first-welcome.json", first.receipt());
    let sandbox = &test.isolated.sandbox;
    let tools = sandbox.dir.join("bin");
    fs::create_dir(&tools).unwrap();
    executable::write(
        tools.join("herdr"),
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"${0%/*}/../started\"\nexit 1\n",
        0o700,
    )
    .unwrap();
    let mut gui = gui_command(sandbox, |_| None);
    desktop.configure(&mut gui);
    gui.env("PATH", tools)
        .env("SHELL", sandbox.dir.join("missing-shell"));
    test.isolated.gui = Some(gui.spawn().unwrap());
    test.await_state("online", Some(&first.workspace), None, "initial-online");
    let native_window = desktop.window(&test.isolated);
    test.stop_server("first");
    first.shutdown(&test.evidence.join("first-events.txt"));
    test.await_state("Herdr stopped", None, None, "stopped");
    let start = Instant::now();
    while start.elapsed() < HOLD {
        let window = test.state().expect("stopped window disappeared");
        assert!(
            window
                .hosts
                .iter()
                .any(|h| h.selected && h.id == "local" && h.status == "Herdr stopped")
        );
        assert!(window.workspace.is_none());
        thread::sleep(Duration::from_millis(500));
    }
    test.record(
        "hold.txt",
        format!(
            "{} ms without a startup attempt\n",
            start.elapsed().as_millis()
        ),
    );
    let mut missing_labels = Vec::new();
    for width in [900, 640] {
        test.action(&["window", "size", &format!("{width}x600")]);
        let label = format!("stopped-{width}");
        test.await_state("Herdr stopped", None, Some(width), &label);
        if !desktop.capture(&test, &native_window, &label, width, true) {
            test.await_state("Herdr stopped", None, Some(width), &label);
            missing_labels.push(width);
        }
    }
    test.check_gui();
    assert!(
        missing_labels.is_empty(),
        "stopped card lacks its painted title or start action at widths {missing_labels:?}"
    );
    let second = test.start_server(&binary, "second", REPLACEMENT_PROMPT);
    test.record("second-welcome.json", second.receipt());
    assert_ne!(first.boot, second.boot);
    test.await_state("Herdr stopped", None, Some(640), "before-reconnect");
    test.action(&["window", "reconnect"]);
    test.await_state("online", Some(&second.workspace), Some(640), "recovered");
    assert!(
        desktop.capture(&test, &native_window, "recovered", 640, false),
        "recovered window must show the replacement prompt with the outage card cleared"
    );
    test.check_gui();
    test.action(&["app", "quit"]);
    assert!(wait(test.isolated.gui.as_mut().unwrap(), "GUI quit").success());
    test.isolated.gui = None;
    test.stop_server("second");
    second.shutdown(&test.evidence.join("second-events.txt"));
    assert!(!test.isolated.sandbox.dir.join("started").exists());
    test.record(
        "complete.txt",
        "Private GUI exited. Replacement survived GUI exit, then exited on its own TERM.\n",
    );
}

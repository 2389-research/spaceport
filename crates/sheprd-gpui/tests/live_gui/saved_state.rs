// ABOUTME: Runs normal GUI children against one owned Herdr on a private Linux desktop.
// ABOUTME: Keeps exact process, control-window and native-window receipts across restarts.
use super::*;
use rustix::process::{Pid, Signal, kill_process};
use sha2::{Digest, Sha256};
use sheprd_control_protocol::protocol::app::{CoreCatalogState, Status, WindowInfo};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

#[path = "saved_state/display.rs"]
mod display;
#[path = "../../../test-support/executable.rs"]
mod executable;
#[path = "saved_state/recovery.rs"]
mod recovery;
#[path = "saved_state/witness.rs"]
mod witness;

const TIMEOUT: Duration = Duration::from_secs(30);
const POLL: Duration = Duration::from_millis(100);
const DIGEST: &str = "14d9e8bc8b4a9e0118d77c32266f76d461f09b40342eb23752960bd89f59929d";
const SERVER_LOG: &str = "herdr/herdr-server.log";

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
        if let Ok(tail) = log_tail(&self.isolated.sandbox.dir.join("server").join(SERVER_LOG)) {
            let _ = fs::write(self.evidence.join("herdr-server.log"), tail);
        }
        for name in [
            "gui.log",
            "server.log",
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

    fn start_server(&mut self, binary: &Path) -> witness::Witness {
        let owned = &mut self.isolated;
        assert!(owned.daemon.is_none());
        assert!(!owned.sandbox.socket().exists());
        owned.daemon = Some(
            owned
                .sandbox
                .command(binary, "server.log")
                // The daemon's state stays separate from the two files under test.
                .env("XDG_CONFIG_HOME", owned.sandbox.dir.join("server"))
                .env(
                    "XDG_STATE_HOME",
                    owned.sandbox.dir.join("server").join("state"),
                )
                .env("HERDR_STARTUP_CWD", &owned.sandbox.dir)
                .env("PS1", "WITNESS> ")
                .arg("server")
                .spawn()
                .unwrap(),
        );
        owned
            .sandbox
            .wait_for_daemon(owned.daemon.as_mut().unwrap(), TIMEOUT);
        witness::Witness::open(owned.sandbox.socket(), owned.daemon.as_ref().unwrap().id())
    }

    fn stop_server(&mut self) {
        let child = self.isolated.daemon.as_mut().unwrap();
        assert!(
            child.try_wait().unwrap().is_none(),
            "owned server already exited"
        );
        kill_process(Pid::from_child(child), Signal::TERM).unwrap();
        let status = wait(child, "server");
        assert!(status.success(), "server terminated with {status}");
        self.isolated.daemon = None;
        let path = self.isolated.sandbox.dir.join("server").join(SERVER_LOG);
        let log = log_tail(&path).unwrap();
        self.record("herdr-server.log", &log);
        assert!(
            log.lines()
                .any(|line| line.contains("server shutdown initiated") && line.contains("SIGTERM")),
            "server did not record its TERM"
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

    fn reply(&self, args: &[&str], label: &str, code: i32) -> serde_json::Value {
        let (status, output) = self.cli(args);
        self.record(&format!("{label}.json"), &output);
        assert_eq!(status.code(), Some(code), "{args:?}: {output}");
        serde_json::from_str(&output).unwrap()
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

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn prepare(binary: &Path) -> Qualification {
    let mut file = File::open(binary).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 16384];
    loop {
        let count = file.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    let digest = hex(&hash.finalize());
    assert_eq!(digest, DIGEST, "expected the Linux x86_64 v0.9.5 release");
    let evidence = PathBuf::from(
        std::env::var_os("SPACEPORT_SAVED_STATE_EVIDENCE").expect("set private evidence path"),
    );
    assert!(evidence.is_absolute());
    fs::create_dir(&evidence).expect("evidence directory must be new");
    let test = Qualification {
        isolated: Isolated {
            sandbox: Sandbox::new(),
            daemon: None,
            gui: None,
        },
        evidence,
        window: None,
    };
    test.record("herdr.sha256", format!("{digest}  herdr-linux-x86_64\n"));
    let sandbox = &test.isolated.sandbox;
    let tools = sandbox.dir.join("bin");
    fs::create_dir(&tools).unwrap();
    executable::write(
        tools.join("herdr"),
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"${0%/*}/../started\"\nexit 1\n",
        0o700,
    )
    .unwrap();
    test
}

impl Qualification {
    fn launch(&mut self, desktop: &display::Display, workspace: &str, label: &str) -> String {
        assert!(self.isolated.gui.is_none());
        self.window = None;
        let sandbox = &self.isolated.sandbox;
        let mut gui = gui_command(sandbox, |_| None);
        desktop.configure(&mut gui);
        gui.env("PATH", sandbox.dir.join("bin"))
            .env("SHELL", sandbox.dir.join("missing-shell"));
        self.isolated.gui = Some(gui.spawn().unwrap());
        self.await_state("online", Some(workspace), None, &format!("{label}-online"));
        let xid = desktop.window(&self.isolated);
        let receipt = serde_json::json!({
            "pid": self.isolated.gui.as_ref().unwrap().id(),
            "control_window": self.window.unwrap(),
            "xid": xid,
        });
        self.record(&format!("{label}-window.json"), receipt.to_string());
        xid
    }

    fn quit(&mut self, label: &str) {
        self.check_gui();
        self.reply(&["app", "quit"], &format!("{label}-quit"), 0);
        let gui = self.isolated.gui.as_mut().unwrap();
        let pid = gui.id();
        let status = wait(gui, "GUI quit");
        self.isolated.gui = None;
        let tail = log_tail(&self.isolated.sandbox.dir.join("gui.log")).unwrap();
        self.record(&format!("{label}-gui.log"), &tail);
        assert_gui_success(status, &tail);
        let server = self.isolated.daemon.as_mut().unwrap();
        assert!(
            server.try_wait().unwrap().is_none(),
            "Herdr exited with {label}"
        );
        let receipt = serde_json::json!({
            "reaped_gui_pid": pid,
            "gui_exit": status.code(),
            "surviving_server_pid": server.id(),
        });
        self.record(&format!("{label}-exit.json"), receipt.to_string());
        assert!(!self.isolated.sandbox.dir.join("started").exists());
    }
}

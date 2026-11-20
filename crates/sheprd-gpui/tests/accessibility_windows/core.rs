// ABOUTME: Qualifies a real isolated Core terminal through native Windows accessibility.
// ABOUTME: Owns the exact runtime, pipe, GUI, and state without installing or discovering a daemon.
use super::{CAPTURE_LIMIT, Duration, GuiTarget, Instant, Running, fs, thread};
use sheprd_core_client::{Core, Endpoint, wire::Attach};
use std::{
    io,
    os::windows::process::CommandExt as _,
    path::PathBuf,
    process::{Child, Command},
};

#[derive(Clone, Copy)]
pub(super) struct Target {
    nonce: uuid::Uuid,
}

impl Target {
    pub(super) fn apply(&self, command: &mut Command) {
        let endpoint = self.endpoint();
        command
            .env("HC_SESSION", endpoint.session)
            .env("HC_SOCKET", endpoint.socket);
    }

    fn endpoint(&self) -> Endpoint {
        Endpoint {
            session: format!("sp-{}", self.nonce.simple()),
            socket: PathBuf::from(format!(r"\\.\pipe\spaceport-core-{}", self.nonce.simple())),
        }
    }
}

struct Fixture {
    binary: PathBuf,
    core: Option<Core>,
    dir: Option<tempfile::TempDir>,
    running: Running,
    runtime: Option<Runtime>,
    target: Target,
}

struct Runtime {
    capture: PathBuf,
    child: Child,
    stop: Command,
}

impl Fixture {
    fn start() -> Self {
        let binary = PathBuf::from(
            std::env::var_os("HERDR_CORE_TEST_BINARY")
                .expect("select the exact staged HERDR_CORE_TEST_BINARY"),
        );
        assert!(
            binary.is_absolute() && binary.is_file(),
            "Core fixture binary must be an absolute file"
        );
        let target = Target {
            nonce: uuid::Uuid::new_v4(),
        };
        let mut fixture = Self {
            binary,
            core: None,
            dir: Some(
                tempfile::Builder::new()
                    .prefix("sp-core-")
                    .tempdir()
                    .unwrap(),
            ),
            running: Running::prepared(),
            runtime: None,
            target,
        };
        let capture = fixture.dir.as_ref().unwrap().path().join("core-stop.log");
        let mut stop = fixture.runtime_command("core-stop.log");
        stop.args(["server", "stop"]);
        let mut command = fixture.runtime_command("core-serve.log");
        command.arg("serve");
        let child = command.spawn().unwrap();
        fixture.runtime = Some(Runtime {
            capture,
            child,
            stop,
        });
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            match Core::connect(&fixture.target.endpoint()) {
                Ok(core) => {
                    fixture.core = Some(core);
                    break;
                }
                Err(error) => {
                    assert!(
                        fixture
                            .runtime
                            .as_mut()
                            .unwrap()
                            .child
                            .try_wait()
                            .unwrap()
                            .is_none(),
                        "owned Core exited: {error}"
                    );
                    assert!(
                        Instant::now() < deadline,
                        "private Core connection failed without relaxing ownership: {error}"
                    );
                    thread::sleep(Duration::from_millis(20));
                }
            }
        }
        assert_eq!(fixture.api().hello().generation, 1);
        assert!(fixture.api().list(false).unwrap().terminals.is_empty());
        fs::write(
            fixture
                .running
                .sandbox
                .dir
                .join("config/spaceport/config.local.toml"),
            "language = \"en-US\"\nherdr_core_enabled = true\n",
        )
        .unwrap();
        fixture.running.start_daemon();
        fixture.running.start_gui(GuiTarget::Core(fixture.target));
        fixture
            .running
            .wait_tree("private startup before Core command", |tree| {
                tree.contains("Document 'Terminal' [Value, Text]")
            });
        fixture
    }

    fn api(&self) -> &Core {
        self.core.as_ref().unwrap()
    }

    fn runtime_command(&self, log: &str) -> Command {
        let root = self.dir.as_ref().unwrap().path();
        let mut command = self.running.sandbox.command(&self.binary, root.join(log));
        self.target.apply(&mut command);
        command
            .creation_flags(sheprd_client::CREATE_NO_WINDOW)
            .env("HOME", root)
            .env("USERPROFILE", root)
            .env("APPDATA", root.join("appdata"))
            .env("LOCALAPPDATA", root.join("localappdata"))
            .env("XDG_CONFIG_HOME", root.join("config"))
            .env("XDG_STATE_HOME", root.join("state"))
            .env("XDG_DATA_HOME", root.join("data"))
            .env("XDG_CACHE_HOME", root.join("cache"))
            .env("XDG_RUNTIME_DIR", root)
            .env("TMPDIR", root)
            .env("TEMP", root)
            .env("TMP", root)
            .current_dir(root);
        command
    }

    fn open_terminal(&mut self) -> String {
        self.running
            .control(&["run", "--json", "new_core_terminal"]);
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let terminals = self.api().list(false).unwrap().terminals;
            if let [terminal] = terminals.as_slice() {
                let address = terminal.address.clone();
                let tabs: serde_json::Value =
                    serde_json::from_str(&self.running.control_output(&["tab", "--json", "list"]))
                        .unwrap();
                if let Some(tab) = tabs["tabs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|tab| tab["kind"] == "core_terminal" && tab["location"] == address)
                {
                    let tab = tab["id"].as_u64().unwrap().to_string();
                    self.running.control(&["tab", "--json", "focus", &tab]);
                    return address;
                }
            }
            assert!(
                Instant::now() < deadline,
                "Core command did not create exactly one app-owned terminal"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn cleanup(&mut self) -> io::Result<()> {
        super::stop_children([
            &mut self.running.command,
            &mut self.running.observer,
            &mut self.running.gui,
        ]);
        self.core.take();
        let Some(mut runtime) = self.runtime.take() else {
            return Ok(());
        };
        let stopped = self
            .running
            .command_output(runtime.stop, &runtime.capture, Duration::from_secs(10))
            .map(|_| ());
        let ended = sheprd_client::wait::exit(
            &mut runtime.child,
            Instant::now() + Duration::from_secs(10),
            || false,
        )
        .and_then(|outcome| match outcome {
            sheprd_client::wait::Exit::Exited(status) if status.success() => Ok(()),
            sheprd_client::wait::Exit::Exited(status) => {
                Err(io::Error::other(format!("owned Core exited with {status}")))
            }
            sheprd_client::wait::Exit::Stopped(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "owned Core did not exit after private shutdown",
            )),
        });
        stopped.and(ended)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            let preserved = self.dir.take().map(tempfile::TempDir::keep);
            eprintln!("CORE_NATIVE cleanup failed; preserved private state {preserved:?}: {error}");
            if !thread::panicking() {
                panic!("private Core cleanup failed: {error}");
            }
        }
    }
}

#[test]
#[ignore = "requires Windows desktop plus explicitly selected Herdr and Core fixture binaries"]
fn native_core_text_caret_bounds_and_output_changes_match_the_real_runtime() {
    let mut fixture = Fixture::start();
    let terminal = fixture.open_terminal();
    fixture
        .running
        .wait_tree("native Core terminal text provider", |tree| {
            tree.contains("Document 'Terminal' [Value, Text]")
        });
    let info = fixture.api().terminal_get(&terminal).unwrap();
    let observed = fixture
        .api()
        .attach(&Attach {
            cols: info.cols,
            rows: info.rows,
            terminal: &terminal,
            observe: true,
            ..Attach::default()
        })
        .unwrap();
    for (name, source) in [
        (
            "uia_terminal.ps1",
            include_str!("../support/uia_terminal.ps1"),
        ),
        ("uia_live.cs", include_str!("../support/uia_live.cs")),
    ] {
        fs::write(fixture.running.sandbox.dir.join(name), source).unwrap();
    }
    let capture = fixture
        .running
        .sandbox
        .dir
        .join("core-terminal-capture.log");
    let command = fixture.running.powershell(
        &fixture.running.sandbox.dir.join("uia_terminal.ps1"),
        "core-terminal-capture.log",
    );
    let output = fixture
        .running
        .command_output(command, &capture, Duration::from_secs(30))
        .unwrap();
    for receipt in [
        "TERMINAL_A11Y text=true native_text_changed=true unselected_caret=true caret_line_bounds=true",
        "TERMINAL_A11Y observer_removed=true",
    ] {
        assert!(output.contains(receipt), "{output}");
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let marker = b"SP_A11Y_7";
    let mut seen = Vec::new();
    while !seen.windows(marker.len()).any(|bytes| bytes == marker) {
        assert!(
            Instant::now() < deadline,
            "native text did not reach this exact Core terminal"
        );
        let chunk = observed.read(Duration::from_millis(100)).unwrap();
        assert!(
            chunk.end.is_none(),
            "owned Core terminal ended during native text check"
        );
        assert!(
            seen.len() + chunk.bytes.len() <= CAPTURE_LIMIT,
            "Core fixture output exceeds 1 MiB"
        );
        seen.extend(chunk.bytes);
    }
    assert_eq!(fixture.api().list(false).unwrap().terminals.len(), 1);
    eprintln!(
        "CORE_NATIVE text=true native_text_changed=true caret_line_bounds=true exact_runtime_output=true observer_removed=true"
    );
}

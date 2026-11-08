//! Against a real herdr-core in a private home and socket. Opt in with
//! `HERDR_CORE_TEST_BINARY=/path/to/herdr-core cargo test -p sheprd-core-client -- --ignored`.

use crate::{
    Core, Endpoint, Error,
    wire::{Attach, Run},
};
use std::{
    ffi::OsString,
    io,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

#[cfg(all(unix, feature = "adapter-live-test"))]
mod acp;
#[cfg(unix)]
mod file_read;
#[cfg(unix)]
mod lifecycle;
#[cfg(unix)]
mod machines;
#[cfg(unix)]
mod observation;
#[cfg(unix)]
mod observer;
#[cfg(unix)]
mod remote_document;
#[cfg(unix)]
mod remote_terminal;
#[cfg(unix)]
mod resize;
#[cfg(unix)]
mod termination;

struct Sandbox {
    _dir: tempfile::TempDir,
    binary: OsString,
    child: Child,
    vars: Vec<(&'static str, OsString)>,
    stop_attempted: bool,
}

impl Sandbox {
    fn start(binary: OsString) -> Self {
        Self::start_with(binary, Vec::new())
    }

    fn start_with(binary: OsString, extra: Vec<(&'static str, OsString)>) -> Self {
        let parent = std::env::var_os("HERDR_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into());
        let dir = tempfile::Builder::new()
            .prefix("hc")
            .tempdir_in(parent)
            .unwrap();
        let at = |name: &str| dir.path().join(name).into_os_string();
        let mut vars = vec![
            (
                "HC_LAUNCHD_DIRECT",
                OsString::from(std::process::id().to_string()),
            ),
            ("HC_SESSION", OsString::from("live")),
            ("HC_SOCKET", at("c.sock")),
            ("HOME", at("home")),
            ("PATH", std::env::var_os("PATH").unwrap_or_default()),
            ("TMPDIR", dir.path().as_os_str().to_owned()),
            ("XDG_CACHE_HOME", at("cache")),
            ("XDG_CONFIG_HOME", at("config")),
            ("XDG_DATA_HOME", at("data")),
            ("XDG_RUNTIME_DIR", dir.path().as_os_str().to_owned()),
            ("XDG_STATE_HOME", at("state")),
        ];
        vars.extend(extra);
        let child = Self::command(&binary, &vars)
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Self {
            _dir: dir,
            binary,
            child,
            vars,
            stop_attempted: false,
        }
    }

    fn command(binary: &OsString, vars: &[(&'static str, OsString)]) -> Command {
        let mut command = Command::new(binary);
        command
            .env_clear()
            .envs(vars.iter().map(|(name, value)| (name, value)));
        command
    }

    fn var(&self, name: &str) -> Option<OsString> {
        self.vars
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.clone())
    }

    fn connect(&self) -> Core {
        let endpoint = Endpoint::resolve(|name| self.var(name)).unwrap();
        assert_eq!(endpoint.socket, Path::new(&self.var("HC_SOCKET").unwrap()));
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match Core::connect(&endpoint) {
                Ok(core) => return core,
                Err(Error::Connect { .. }) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(50))
                }
                Err(error) => panic!("{error}"),
            }
        }
    }

    fn restart(&mut self) {
        assert!(
            self.child.try_wait().unwrap().is_some(),
            "private Core still runs"
        );
        self.child = Self::command(&self.binary, &self.vars)
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        self.stop_attempted = false;
    }

    fn wait_exact(child: &mut Child) -> io::Result<ExitStatus> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(status);
            }
            if Instant::now() >= deadline {
                child.kill()?;
                child.wait()?;
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "private Core child did not exit",
                ));
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    }

    fn stop(&mut self) -> io::Result<()> {
        self.stop_attempted = true;
        let mut stopper = Self::command(&self.binary, &self.vars)
            .args(["server", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let stop_status = Self::wait_exact(&mut stopper);
        if !matches!(stop_status, Ok(ref status) if status.success()) {
            let _ = self.child.kill();
        }
        let serve_status = Self::wait_exact(&mut self.child);
        if !stop_status?.success() {
            return Err(io::Error::other("private Core server stop failed"));
        }
        serve_status?;
        Ok(())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        if !self.stop_attempted
            && let Err(error) = self.stop()
        {
            eprintln!("private Core cleanup failed: {error}");
        }
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn a_real_core_runs_streams_and_stores() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut sandbox = Sandbox::start(binary);
    let core = sandbox.connect();
    assert_eq!(core.hello().generation, 1);
    assert!(core.machine_list().unwrap().machines.is_empty());

    let argv = [
        "sh".to_owned(),
        "-c".to_owned(),
        "printf hi-core; sleep 30".to_owned(),
    ];
    let started = core
        .run(&Run {
            argv: &argv,
            name: Some("live"),
            ..Run::default()
        })
        .unwrap();
    assert_eq!(
        core.terminal_get(&started.address).unwrap().address,
        started.address
    );
    let subscription = core.subscribe().unwrap();
    assert!(
        subscription
            .terminals
            .iter()
            .any(|terminal| terminal.address == started.address)
    );

    let attachment = core
        .attach(&Attach {
            cols: 80,
            rows: 24,
            terminal: &started.address,
            ..Attach::default()
        })
        .unwrap();
    let mut seen = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !seen.windows(7).any(|window| window == b"hi-core") {
        assert!(
            Instant::now() < deadline,
            "no output: {:?}",
            String::from_utf8_lossy(&seen)
        );
        let chunk = attachment.read(Duration::from_millis(100)).unwrap();
        assert_eq!(chunk.end, None);
        seen.extend(chunk.bytes);
    }

    let doc = core.doc_get("herdr-ui-live", None).unwrap();
    assert_eq!(doc.version, 0);
    let version = core
        .doc_set("herdr-ui-live", &serde_json::json!({"tabs": 1}), 0, None)
        .unwrap();
    assert_eq!(
        core.doc_get("herdr-ui-live", None).unwrap().value,
        serde_json::json!({"tabs": 1})
    );
    assert!(matches!(
        core.doc_set("herdr-ui-live", &serde_json::json!(2), 0, None),
        Err(Error::Refused { .. })
    ));
    assert!(version > 0);

    core.kill(&started.address).unwrap();
    sandbox.stop().unwrap();
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn a_real_core_through_the_backend_traits() {
    use sheprd_backend::{
        Backend, ClientPaneInputEvent, Role, Size, Spawn, SurfaceUpdate, Terminal as _,
    };

    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut sandbox = Sandbox::start(binary);
    let backend = crate::CoreBackend::new(sandbox.connect(), 1000);
    // A chat's terminal carries its endpoint as a label every client lists.
    let spawn = Spawn {
        argv: vec!["sh".into()],
        labels: [("acp".to_owned(), "ab".repeat(16))].into(),
        name: Some("traits".into()),
        ..Spawn::default()
    };
    let info = backend.spawn(&spawn).unwrap();
    assert_eq!(info.labels, spawn.labels);
    assert!(
        backend
            .terminals()
            .unwrap()
            .iter()
            .any(|terminal| terminal.id == info.id && terminal.labels == spawn.labels)
    );

    let mut terminal = backend
        .attach(&info.id, Size::new(40, 6).unwrap(), Role::Active)
        .unwrap();
    terminal
        .input(&ClientPaneInputEvent::TextCommit("echo $((6*7))".into()))
        .unwrap();
    terminal
        .input(&ClientPaneInputEvent::Key {
            code: sheprd_protocol::ClientKeyCode::Enter,
            modifiers: 0,
            kind: sheprd_protocol::ClientKeyKind::Press,
            repeat_count: 1,
            shifted_codepoint: None,
            generated_text: None,
            tracks_release: false,
            physical_key_id: None,
            windows_record: None,
        })
        .unwrap();

    // Apply frames and row patches to a local copy, as the app's painter would.
    let mut screen: Vec<sheprd_protocol::CellData> = Vec::new();
    let mut width = 0usize;
    let text = |screen: &[sheprd_protocol::CellData]| {
        screen
            .iter()
            .map(|cell| cell.symbol.as_str())
            .collect::<String>()
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    while !text(&screen).contains("42") {
        assert!(
            Instant::now() < deadline,
            "no answer on screen: {:?}",
            text(&screen)
        );
        for update in terminal.next(Duration::from_millis(100)).unwrap() {
            match update {
                SurfaceUpdate::Frame(frame) => {
                    width = frame.width.into();
                    screen = frame.cells;
                }
                SurfaceUpdate::Rows { rows, .. } => {
                    for row in rows {
                        let start = usize::from(row.y) * width + usize::from(row.x);
                        for (offset, cell) in row.cells.into_iter().enumerate() {
                            if let Some(slot) = screen.get_mut(start + offset) {
                                *slot = cell;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    backend.kill(&info.id).unwrap();
    sandbox.stop().unwrap();
}

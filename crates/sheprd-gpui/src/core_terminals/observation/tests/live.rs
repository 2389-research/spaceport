use super::*;
use sheprd_core_client::{Core, Endpoint};
use std::{
    ffi::OsString,
    io,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
};

mod remote;

struct Sandbox {
    _dir: tempfile::TempDir,
    binary: OsString,
    child: Child,
    socket: PathBuf,
    home: PathBuf,
    stop_attempted: bool,
}

impl Sandbox {
    fn start(binary: OsString) -> Self {
        Self::start_with(binary, Vec::new())
    }

    fn start_with(binary: OsString, extra: Vec<(&'static str, OsString)>) -> Self {
        let parent = std::env::var_os("HERDR_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into());
        let dir = tempfile::Builder::new()
            .prefix("hcobs")
            .tempdir_in(parent)
            .unwrap();
        let socket = dir.path().join("c.sock");
        let home = dir.path().join("home");
        let child = Self::command(&binary, &socket, &home)
            .envs(extra)
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Self {
            _dir: dir,
            binary,
            child,
            socket,
            home,
            stop_attempted: false,
        }
    }

    fn restart(&mut self) {
        assert!(self.child.try_wait().unwrap().is_some());
        self.child = Self::command(&self.binary, &self.socket, &self.home)
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        self.stop_attempted = false;
    }

    fn command(binary: &OsString, socket: &PathBuf, home: &PathBuf) -> Command {
        let mut command = Command::new(binary);
        command
            .env_clear()
            .env("HC_LAUNCHD_DIRECT", std::process::id().to_string())
            .env("HC_SESSION", "observe")
            .env("HC_SOCKET", socket)
            .env("HOME", home)
            .env("TMPDIR", socket.parent().unwrap())
            .env("XDG_CACHE_HOME", home.join("cache"))
            .env("XDG_CONFIG_HOME", home.join("config"))
            .env("XDG_DATA_HOME", home.join("data"))
            .env("XDG_RUNTIME_DIR", socket.parent().unwrap())
            .env("XDG_STATE_HOME", home.join("state"))
            .env("PATH", std::env::var_os("PATH").unwrap_or_default());
        command
    }

    fn connect(&self) -> Core {
        let endpoint = Endpoint {
            socket: self.socket.clone(),
            session: "observe".into(),
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match Core::connect(&endpoint) {
                Ok(core) => return core,
                Err(sheprd_core_client::Error::Connect { .. }) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(30));
                }
                Err(error) => panic!("private Core did not connect: {error}"),
            }
        }
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
        let mut stopper = Self::command(&self.binary, &self.socket, &self.home)
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
        let stopped = if self.stop_attempted {
            Ok(())
        } else {
            self.stop()
        };
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if !std::thread::panicking() {
            assert!(
                stopped.is_ok(),
                "private Core server stop failed: {stopped:?}"
            );
        }
    }
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn worker_refetches_local_document_after_notice() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY")
        .expect("set HERDR_CORE_TEST_BINARY to an explicit private test binary");
    let mut sandbox = Sandbox::start(binary);
    let core = sandbox.connect();
    let backend = CoreBackend::new(sandbox.connect(), 1000);
    assert!(
        sandbox.child.try_wait().unwrap().is_none(),
        "private Core detached"
    );
    let mailbox = Arc::new(Mutex::new(None));
    let stop = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));
    let worker = {
        let mailbox = Arc::clone(&mailbox);
        let stop = Arc::clone(&stop);
        let running = Arc::clone(&running);
        std::thread::spawn(move || {
            run(
                backend,
                1,
                stop,
                running,
                mailbox,
                Arc::new(Mutex::new(RemoteSelection::default())),
            )
        })
    };
    let wait_for_version = |version| {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(latest) = take_latest(&mailbox)
                && latest.document().is_some_and(|doc| doc.version == version)
                && latest.machines().is_some()
            {
                return latest;
            }
            assert!(
                Instant::now() < deadline,
                "worker did not read document version {version}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    };
    let initial = wait_for_version(0);
    assert_eq!(initial.machines(), Some([].as_slice()));
    assert_eq!(
        core.doc_set(
            NAME,
            &json!({"schema_version": 1, "workspaces": []}),
            0,
            None
        )
        .unwrap(),
        1
    );
    wait_for_version(1);
    stop.store(true, Ordering::Release);
    worker.join().unwrap();
    assert!(!running.load(Ordering::Acquire));
    sandbox.stop().unwrap();
}

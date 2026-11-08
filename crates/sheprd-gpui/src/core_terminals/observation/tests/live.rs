use super::*;
use sheprd_core_client::{Core, Endpoint};
use std::{
    ffi::OsString,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

struct Sandbox {
    _dir: tempfile::TempDir,
    binary: OsString,
    child: Child,
    socket: PathBuf,
    home: PathBuf,
}

impl Sandbox {
    fn start(binary: OsString) -> Self {
        let parent = std::env::var_os("HERDR_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into());
        let dir = tempfile::Builder::new()
            .prefix("hcobs")
            .tempdir_in(parent)
            .unwrap();
        let socket = dir.path().join("c.sock");
        let home = dir.path().join("home");
        let child = Self::command(&binary, &socket, &home)
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
        }
    }

    fn command(binary: &OsString, socket: &PathBuf, home: &PathBuf) -> Command {
        let mut command = Command::new(binary);
        command
            .env_clear()
            .env("HC_SESSION", "observe")
            .env("HC_SOCKET", socket)
            .env("HOME", home)
            .env("XDG_CONFIG_HOME", home.join("config"))
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
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = Self::command(&self.binary, &self.socket, &self.home)
            .args(["server", "stop"])
            .output();
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.child.try_wait().ok().flatten().is_none() {
            if Instant::now() >= deadline {
                let _ = self.child.kill();
                let _ = self.child.wait();
                break;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn worker_refetches_local_document_after_notice() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY")
        .expect("set HERDR_CORE_TEST_BINARY to an explicit private test binary");
    let sandbox = Sandbox::start(binary);
    let core = sandbox.connect();
    let backend = CoreBackend::new(sandbox.connect(), 1000);
    let mailbox = Arc::new(Mutex::new(None));
    let stop = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));
    let worker = {
        let mailbox = Arc::clone(&mailbox);
        let stop = Arc::clone(&stop);
        let running = Arc::clone(&running);
        std::thread::spawn(move || run(backend, 1, stop, running, mailbox))
    };
    let wait_for_version = |version| {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(latest) = take_latest(&mailbox)
                && latest.document().is_some_and(|doc| doc.version == version)
            {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "worker did not read document version {version}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    };
    wait_for_version(0);
    assert_eq!(
        core.doc_set(
            DOCUMENT,
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
}

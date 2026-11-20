// ABOUTME: Owns the explicitly selected private Windows Core runtime for native fixtures.
// ABOUTME: Reuses sandbox commands and exact-child waits without owning GUI or state retention.
use super::Sandbox;
use sheprd_core_client::{Core, Endpoint};
use std::{
    io,
    os::windows::process::CommandExt as _,
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub(super) struct Target {
    nonce: uuid::Uuid,
}

impl Target {
    pub(super) fn new() -> Self {
        Self {
            nonce: uuid::Uuid::new_v4(),
        }
    }

    pub(super) fn nonce(&self) -> uuid::Uuid {
        self.nonce
    }

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

pub(super) fn binary() -> PathBuf {
    let binary = PathBuf::from(
        std::env::var_os("HERDR_CORE_TEST_BINARY")
            .expect("select the exact staged HERDR_CORE_TEST_BINARY"),
    );
    assert!(
        binary.is_absolute() && binary.is_file(),
        "Core fixture binary must be an absolute file"
    );
    binary
}

pub(super) struct Runtime {
    capture: PathBuf,
    child: Child,
    stop: Command,
    target: Target,
}

impl Runtime {
    pub(super) fn start(sandbox: &Sandbox, binary: &Path, root: &Path, target: Target) -> Self {
        let capture = root.join("core-stop.log");
        // Sandbox::command opens both stop streams before serve can start.
        let mut stop = command(sandbox, binary, root, target, "core-stop.log");
        stop.args(["server", "stop"]);
        let mut serve = command(sandbox, binary, root, target, "core-serve.log");
        serve.arg("serve");
        let child = serve.spawn().unwrap();
        Self {
            capture,
            child,
            stop,
            target,
        }
    }

    pub(super) fn connect(&mut self) -> Core {
        let deadline = Instant::now() + Duration::from_secs(15);
        let core = loop {
            match Core::connect(&self.target.endpoint()) {
                Ok(core) => break core,
                Err(error) => {
                    assert!(
                        self.child.try_wait().unwrap().is_none(),
                        "owned Core exited: {error}"
                    );
                    assert!(
                        Instant::now() < deadline,
                        "private Core connection failed without relaxing ownership: {error}"
                    );
                    thread::sleep(Duration::from_millis(20));
                }
            }
        };
        assert_eq!(core.hello().generation, 1);
        assert!(core.list(false).unwrap().terminals.is_empty());
        core
    }

    /// The caller drops GUI and client connections before consuming this owner.
    /// Its command runner retains the exact stop child and bounds its output.
    pub(super) fn shutdown(
        mut self,
        run_stop: impl FnOnce(Command, &Path, Duration) -> io::Result<()>,
    ) -> io::Result<()> {
        let stopped = run_stop(self.stop, &self.capture, Duration::from_secs(10));
        let ended = sheprd_client::wait::exit(
            &mut self.child,
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

fn command(sandbox: &Sandbox, binary: &Path, root: &Path, target: Target, log: &str) -> Command {
    let mut command = sandbox.command(binary, root.join(log));
    target.apply(&mut command);
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

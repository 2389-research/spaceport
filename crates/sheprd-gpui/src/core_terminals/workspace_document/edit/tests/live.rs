use super::*;
use sheprd_core_client::{Core, Endpoint};
use std::{
    ffi::OsString,
    io,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

struct Sandbox {
    _dir: tempfile::TempDir,
    binary: OsString,
    child: Child,
    home: PathBuf,
    socket: PathBuf,
}

impl Sandbox {
    fn start() -> Self {
        let binary = std::env::var_os("HERDR_CORE_TEST_BINARY")
            .expect("set HERDR_CORE_TEST_BINARY to an explicit private test binary");
        let parent = std::env::var_os("HERDR_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into());
        let dir = tempfile::Builder::new()
            .prefix("hcedit")
            .tempdir_in(parent)
            .unwrap();
        let home = dir.path().join("home");
        let socket = dir.path().join("c.sock");
        let child = Self::command(&binary, &home, &socket)
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Self {
            _dir: dir,
            binary,
            child,
            home,
            socket,
        }
    }

    fn command(binary: &OsString, home: &PathBuf, socket: &PathBuf) -> Command {
        let mut command = Command::new(binary);
        command
            .env_clear()
            .env("HC_SESSION", "edit")
            .env("HC_LAUNCHD_DIRECT", std::process::id().to_string())
            .env("HC_SOCKET", socket)
            .env("HOME", home)
            .env("XDG_CONFIG_HOME", home.join("config"))
            .env("XDG_STATE_HOME", home.join("state"))
            .env("XDG_DATA_HOME", home.join("data"))
            .env("XDG_CACHE_HOME", home.join("cache"))
            .env("PATH", std::env::var_os("PATH").unwrap_or_default());
        command
    }

    fn connect(&self) -> Core {
        let endpoint = Endpoint {
            socket: self.socket.clone(),
            session: "edit".into(),
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
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let stop = Self::command(&self.binary, &self.home, &self.socket)
            .args(["server", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut child| Self::wait_exact(&mut child));
        let stopped = matches!(&stop, Ok(status) if status.success());
        if !stopped {
            let _ = self.child.kill();
        }
        let served = Self::wait_exact(&mut self.child);
        if !std::thread::panicking() {
            assert!(stopped, "private Core server stop failed: {stop:?}");
            assert!(
                served.is_ok(),
                "private Core serve child was not reaped: {served:?}"
            );
        }
    }
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn create_rename_archive_delete_only_the_grouping_document() {
    let sandbox = Sandbox::start();
    let core = sandbox.connect();
    let base = read(&core).unwrap();
    assert_eq!(base.version, 0);
    let created = apply(
        &core,
        base,
        &Edit::Create {
            id: id(1),
            label: "One".into(),
        },
    )
    .unwrap();
    assert_eq!(created.version, 1);
    let renamed = apply(
        &core,
        created,
        &Edit::Rename {
            id: id(1),
            label: "First".into(),
        },
    )
    .unwrap();
    let archived = apply(
        &core,
        renamed,
        &Edit::Archive {
            id: id(1),
            archived: true,
        },
    )
    .unwrap();
    assert_eq!(archived.value.workspaces()[0].label, "First");
    assert!(archived.value.workspaces()[0].archived);
    let deleted = apply(&core, archived, &Edit::Delete { id: id(1) }).unwrap();
    assert_eq!(deleted.version, 4);
    assert!(read(&core).unwrap().value.workspaces().is_empty());
    assert_eq!(core.doc_get("other-client", None).unwrap().version, 0);
    assert!(core.list(false).unwrap().terminals.is_empty());
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn disjoint_conflict_rebases_and_same_target_conflict_refuses() {
    let sandbox = Sandbox::start();
    let first = sandbox.connect();
    let second = sandbox.connect();
    let empty = read(&first).unwrap();
    let base = apply(
        &first,
        empty,
        &Edit::Create {
            id: id(1),
            label: "One".into(),
        },
    )
    .unwrap();
    let other = apply(
        &second,
        base.clone(),
        &Edit::Create {
            id: id(2),
            label: "Two".into(),
        },
    )
    .unwrap();
    assert_eq!(other.version, 2);
    let merged = apply(
        &first,
        base,
        &Edit::Rename {
            id: id(1),
            label: "First".into(),
        },
    )
    .unwrap();
    assert_eq!(merged.version, 3);
    assert_eq!(merged.value.workspaces().len(), 2);
    let changed = apply(
        &second,
        merged.clone(),
        &Edit::Archive {
            id: id(1),
            archived: true,
        },
    )
    .unwrap();
    assert_eq!(changed.version, 4);
    let refused = apply(&first, merged, &Edit::Delete { id: id(1) });
    assert!(matches!(refused, Err(Error::Changed(workspace)) if workspace == id(1)));
    let saved = read(&first).unwrap();
    assert_eq!(saved.version, 4);
    assert_eq!(saved.value.workspaces().len(), 2);
    assert!(saved.value.workspaces()[0].archived);
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn membership_edits_only_the_private_core_document() {
    let sandbox = Sandbox::start();
    let core = sandbox.connect();
    let core_id = core.hello().core.id.clone();
    let empty = read(&core).unwrap();
    let created = apply(
        &core,
        empty,
        &Edit::Create {
            id: id(1),
            label: "One".into(),
        },
    )
    .unwrap();
    let added = apply(
        &core,
        created,
        &Edit::AddMember {
            id: id(1),
            core_id: core_id.clone(),
            terminal_id: "t7".into(),
        },
    )
    .unwrap();
    assert_eq!(added.value.workspaces()[0].members[0].core_id, core_id);
    assert_eq!(added.value.workspaces()[0].members[0].terminal_id, "t7");
    assert!(core.list(false).unwrap().terminals.is_empty());
    let removed = apply(
        &core,
        added,
        &Edit::RemoveMember {
            id: id(1),
            core_id,
            terminal_id: "t7".into(),
        },
    )
    .unwrap();
    assert!(removed.value.workspaces()[0].members.is_empty());
    assert_eq!(read(&core).unwrap().version, 3);
    assert!(core.list(false).unwrap().terminals.is_empty());
}

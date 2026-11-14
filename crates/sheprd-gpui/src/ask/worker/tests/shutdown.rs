// ABOUTME: Drives the actual Worker and helper through ignored Pi cancellation.
// ABOUTME: Uses isolated fixture state and private stop markers to prevent orphan cleanup signals.
use super::*;
use anyhow::Context as _;
use sheprd_chat_protocol::{ClientKind, ClientMessage, CommandId, Status, write_frame};
use std::os::unix::{fs::PermissionsExt, net::UnixStream};

#[allow(dead_code, clippy::expect_used, clippy::unwrap_used)]
#[path = "../../../../../test-support/sandbox.rs"]
mod sandbox;

struct Fixture {
    owner: Option<Owner>,
    helper_pid: u32,
    pi_pid: u32,
    directory: PathBuf,
    sandbox: sandbox::Sandbox,
}

impl Fixture {
    fn start() -> anyhow::Result<Self> {
        let helper = explicit_binary("SPACEPORT_ASK_TEST_HELPER")?;
        let node = explicit_binary("SPACEPORT_ASK_TEST_NODE")?;
        let sandbox = sandbox::Sandbox::new();
        let root = sandbox.dir.join("ask");
        let id = Uuid::new_v4();
        let directory = root.join(id.to_string());
        fs::create_dir_all(&directory)?;
        fs::write(root.join("current"), id.to_string())?;
        let bin = sandbox.dir.join("bin");
        fs::create_dir(&bin)?;
        let script = include_str!("../../../../../test-support/ask-pi.mjs")
            .split_once('\n')
            .context("fixture shebang")?
            .1;
        let program = bin.join("pi");
        fs::write(&program, format!("#!{}\n{script}", node.display()))?;
        fs::set_permissions(program, fs::Permissions::from_mode(0o700))?;
        let endpoint = EndpointId::random();
        let child = sandbox
            .command(helper, "helper.log")
            .env(
                "PATH",
                std::env::join_paths([bin, PathBuf::from("/usr/bin"), PathBuf::from("/bin")])?,
            )
            .env("SPACEPORT_FIXTURE_IGNORE_ABORT", "1")
            .args(["ask", "--state-dir"])
            .arg(&directory)
            .arg("--endpoint")
            .arg(endpoint.as_str())
            .arg("--broker")
            .arg(sandbox.dir.join("unused.sock"))
            .arg("--conversation")
            .arg(id.to_string())
            .spawn()?;
        let helper_pid = child.id();
        let owner = Owner {
            root,
            broker: sandbox.dir.join("unused.sock"),
            conversation: Some(id),
            child: Some(child),
            diagnostics: None,
        };
        let mut fixture = Self {
            owner: Some(owner),
            helper_pid,
            pi_pid: 0,
            directory,
            sandbox,
        };
        fixture.wait_for(|state| matches!(state.transcript.status, Status::Idle))?;
        fixture.pi_pid = fs::read_to_string(fixture.directory.join("pid"))?
            .trim()
            .parse()?;
        let address = sheprd_chat_protocol::endpoint_path(
            &fixture.sandbox.dir.join("state/herdr/acp"),
            &endpoint,
        );
        let mut socket = UnixStream::connect(address)?;
        write_frame(
            &mut socket,
            &ClientMessage::Hello {
                client: ClientKind::Gui,
                revision: sheprd_chat_protocol::REVISION,
                since: None,
                version: sheprd_chat_protocol::VERSION,
            },
        )?;
        write_frame(
            &mut socket,
            &ClientMessage::Command {
                id: CommandId::random(),
                command: sheprd_chat_protocol::Command::Prompt {
                    text: "hold ignored abort".into(),
                    attachments: vec![],
                },
            },
        )?;
        fixture.wait_for(|state| {
            state
                .request
                .as_ref()
                .is_some_and(|request| request.phase == ask::Phase::Accepted)
        })?;
        Ok(fixture)
    }

    fn wait_for(&mut self, predicate: impl Fn(&Checkpoint) -> bool) -> anyhow::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(6);
        while Instant::now() < deadline {
            if let Some(state) = read_checkpoint(&self.directory)?
                && predicate(&state)
            {
                return Ok(());
            }
            if let Some(child) = self.owner.as_mut().and_then(|owner| owner.child.as_mut())
                && let Some(status) = child.try_wait()?
            {
                anyhow::bail!(
                    "fixture helper exited {status}: {}",
                    fs::read_to_string(self.sandbox.dir.join("helper.log"))?
                );
            }
            thread::sleep(Duration::from_millis(20));
        }
        anyhow::bail!("fixture helper did not reach the expected checkpoint")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::write(self.directory.join("fixture-stop"), b"stop");
        if let Some(owner) = &mut self.owner {
            let _ = owner.stop();
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while self.pi_pid != 0 && process_exists(self.pi_pid) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            self.pi_pid == 0 || !process_exists(self.pi_pid),
            "private fixture Pi did not exit after its cleanup marker"
        );
    }
}

fn explicit_binary(variable: &str) -> anyhow::Result<PathBuf> {
    let file = PathBuf::from(
        std::env::var_os(variable)
            .with_context(|| format!("set {variable} to the explicit fixture executable"))?,
    );
    anyhow::ensure!(
        file.is_absolute() && file.is_file(),
        "{variable} must name an absolute executable"
    );
    Ok(file)
}

fn process_exists(pid: u32) -> bool {
    sheprd_client::new_command("/bin/ps")
        .args(["-p", &pid.to_string(), "-o", "pid="])
        .output()
        .is_ok_and(|output| output.status.success() && !output.stdout.is_empty())
}

#[test]
#[ignore = "requires explicit SPACEPORT_ASK_TEST_HELPER and SPACEPORT_ASK_TEST_NODE fixture executables"]
fn worker_finish_reaps_pi_after_ignored_abort_and_termination() -> anyhow::Result<()> {
    let mut fixture = Fixture::start()?;
    let worker = Worker::from_owner(fixture.owner.take().context("fixture owner")?)?;
    assert!(matches!(
        worker.events.recv_timeout(Duration::from_secs(3))?,
        Event::Restored(Some(_))
    ));
    worker.finish();
    assert!(
        !process_exists(fixture.helper_pid),
        "Worker.finish returned while its helper still existed"
    );
    assert!(
        !process_exists(fixture.pi_pid),
        "Worker.finish killed the helper before it reaped its ignored-abort Pi child"
    );
    let saved = read_checkpoint(&fixture.directory)?.context("final checkpoint")?;
    assert_eq!(saved.outcome, Outcome::Unknown);
    assert_eq!(
        saved.request.map(|request| request.phase),
        Some(ask::Phase::Unknown)
    );
    assert!(fixture.directory.join("abort-observed").exists());
    assert!(fixture.directory.join("termination-observed").exists());
    Ok(())
}

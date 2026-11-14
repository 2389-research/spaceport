// ABOUTME: Runs explicitly approved native Ask qualification with real Pi and private Herdr.
// ABOUTME: Preserves installed agent sign-in without copying credentials into the sandbox.
use super::*;
use std::{ffi::OsStr, path::PathBuf};

fn approved(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}

fn explicit_file(name: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os(name)
            .unwrap_or_else(|| panic!("set {name} to an explicit absolute executable")),
    );
    assert!(
        path.is_absolute() && path.is_file(),
        "{name} must name an existing absolute executable"
    );
    path
}

#[test]
fn real_provider_fixture_requires_explicit_guard() {
    assert!(approved(Some(OsStr::new("1"))));
    for value in [None, Some(OsStr::new("true")), Some(OsStr::new("0"))] {
        assert!(!approved(value));
    }
}

struct ProviderRun {
    isolated: Isolated,
    passed: bool,
}

fn run_result(passed: bool, view_passed: bool, owner_released: Option<bool>) -> serde_json::Value {
    serde_json::json!({
        "native_provider_pass": passed,
        "native_view_pass": view_passed,
        "helper_owner_released": owner_released,
        "scope": "real native controller/helper/Pi; private synthetic Herdr only; no OS IME claim"
    })
}

#[test]
fn provider_artifact_requires_final_harness_verdict_after_view_success() {
    let value = run_result(false, true, Some(true));
    assert_eq!(value["native_provider_pass"], false);
    assert_eq!(value["native_view_pass"], true);
    let completed = run_result(true, true, Some(true));
    assert_eq!(completed["native_provider_pass"], true);
}

impl Drop for ProviderRun {
    fn drop(&mut self) {
        let running = !self
            .isolated
            .gui
            .as_mut()
            .is_none_or(|child| child.try_wait().ok().flatten().is_some());
        // An app timeout must not strand the exact helper owned by this sandbox.
        // Its own quit protocol reaps Pi; no process discovery or group kill is used.
        if running && let Some(directory) = conversation(&self.isolated.sandbox.dir) {
            let _ = fs::write(
                directory.join(sheprd_chat_protocol::ask::QUIT_FILE),
                b"quit",
            );
            let deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < deadline && !unowned(&directory) {
                thread::sleep(Duration::from_millis(25));
            }
        }
        if let Some(root) = std::env::var_os("HERDR_TEST_ASK_PROVIDER_CAPTURE") {
            let root = PathBuf::from(root);
            if root.is_absolute() {
                let _ = fs::create_dir_all(&root);
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&root, fs::Permissions::from_mode(0o700));
                if let Ok(log) = fs::read_to_string(self.isolated.sandbox.dir.join("gui.log")) {
                    let success = self
                        .isolated
                        .gui
                        .as_mut()
                        .and_then(|child| child.try_wait().ok().flatten())
                        .is_some_and(|exit| exit.success());
                    let _ = fs::write(root.join("ask-provider-run.log"), &log);
                    let result = run_result(
                        self.passed,
                        success && log.contains("ASK provider native PASS:"),
                        conversation(&self.isolated.sandbox.dir).map(|path| unowned(&path)),
                    );
                    let _ = fs::write(
                        root.join("ask-provider-result.json"),
                        serde_json::to_vec_pretty(&result).unwrap_or_default(),
                    );
                }
                if let Some(directory) = conversation(&self.isolated.sandbox.dir)
                    && let Ok(bytes) =
                        fs::read(directory.join(sheprd_chat_protocol::ask::CHECKPOINT_FILE))
                {
                    let _ = fs::write(root.join("ask-provider-checkpoint.json"), bytes);
                }
            }
        }
    }
}

fn conversation(root: &Path) -> Option<PathBuf> {
    let ask = root.join("state/spaceport/ask");
    let id = fs::read_to_string(ask.join("current")).ok()?;
    let id = uuid::Uuid::parse_str(id.trim()).ok()?;
    Some(ask.join(id.to_string()))
}

fn unowned(directory: &Path) -> bool {
    fs::File::open(directory.join("owner.lock")).is_ok_and(|file| file.try_lock().is_ok())
}

#[test]
#[ignore = "requires explicit real-provider approval, installed Pi, private Herdr and native desktop"]
fn native_ask_provider_end_to_end() {
    assert!(
        approved(std::env::var_os("SPACEPORT_TEST_REAL_PROVIDER").as_deref()),
        "requires SPACEPORT_TEST_REAL_PROVIDER=1 after explicit approval"
    );
    let daemon = daemon_binary();
    let pi = explicit_file("HERDR_TEST_PI_BINARY");
    let node = explicit_file("HERDR_TEST_NODE_BINARY");
    let agent_dir = PathBuf::from(
        std::env::var_os("PI_CODING_AGENT_DIR")
            .expect("set PI_CODING_AGENT_DIR to Pi's existing own configuration directory"),
    );
    assert!(
        agent_dir.is_absolute() && agent_dir.is_dir(),
        "Pi configuration must be an existing absolute directory; credentials are never copied"
    );
    let mut run = ProviderRun {
        isolated: Isolated {
            sandbox: Sandbox::new(),
            daemon: None,
            gui: None,
        },
        passed: false,
    };
    let sandbox = &run.isolated.sandbox;
    assert!(
        sandbox
            .dir
            .join("state/herdr/acp/00000000000000000000000000000000.sock")
            .as_os_str()
            .as_encoded_bytes()
            .len()
            < 104,
        "set HERDR_TEST_TMPDIR=/tmp so the private Ask socket fits"
    );
    let tools = sandbox.dir.join("bin");
    fs::create_dir(&tools).unwrap();
    std::os::unix::fs::symlink(pi, tools.join("pi")).unwrap();
    std::os::unix::fs::symlink(node, tools.join("node")).unwrap();
    let mut server = sandbox.command(daemon, "daemon.log");
    server.env("LC_ALL", "en_US.UTF-8");
    run.isolated.daemon = Some(server.arg("server").spawn().unwrap());
    sandbox.wait_for_daemon(
        run.isolated.daemon.as_mut().unwrap(),
        Duration::from_secs(20),
    );
    let mut command = gui_command(sandbox, |name| std::env::var_os(name));
    fs::write(
        sandbox.dir.join("config/spaceport/config.local.toml"),
        "[chat]\nask_enabled = true\n",
    )
    .unwrap();
    command
        .env("HERDR_TEST_ASK_PROVIDER", "1")
        .env("SPACEPORT_TEST_REAL_PROVIDER", "1")
        .env("PI_CODING_AGENT_DIR", agent_dir)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", tools.display()),
        );
    if let Some(root) = std::env::var_os("HERDR_TEST_ASK_PROVIDER_CAPTURE") {
        assert!(Path::new(&root).is_absolute());
        fs::create_dir_all(&root).unwrap();
        command.env("HERDR_TEST_ASK_PROVIDER_CAPTURE", root);
    }
    run.isolated.gui = Some(
        command
            .arg("--socket")
            .arg(sandbox.socket())
            .arg("--integration-test")
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(240);
    let status = loop {
        if let Some(status) = run.isolated.gui.as_mut().unwrap().try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "native Ask provider fixture exceeded its bounded deadline; no question is retried"
        );
        thread::sleep(Duration::from_millis(50));
    };
    let log = fs::read_to_string(run.isolated.sandbox.dir.join("gui.log")).unwrap();
    assert_gui_success(status, &log);
    assert!(log.contains("ASK provider native PASS:"), "{log}");
    assert!(
        run.isolated
            .daemon
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none(),
        "native app quit stopped the owned daemon"
    );
    let directory =
        conversation(&run.isolated.sandbox.dir).expect("actual Ask conversation was not persisted");
    assert!(
        unowned(&directory),
        "actual Ask helper retained ownership after app quit"
    );
    let checkpoint: sheprd_chat_protocol::ask::Checkpoint = serde_json::from_slice(
        &fs::read(directory.join(sheprd_chat_protocol::ask::CHECKPOINT_FILE)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        checkpoint.outcome,
        sheprd_chat_protocol::ask::Outcome::Completed
    );
    run.passed = true;
    eprintln!(
        "ASK provider end-to-end PASS: app/helper/Pi shut down; private daemon survives; one native question observed synthetic terminal and navigated exact source"
    );
}

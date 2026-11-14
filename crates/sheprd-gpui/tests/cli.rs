//! Informational and usage-error paths must not reach GPUI or require a desktop/daemon.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)]

#[cfg(target_os = "macos")]
#[path = "cli/clipboard.rs"]
mod clipboard;
#[path = "cli/orchestrator.rs"]
mod orchestrator;
#[path = "cli/phone.rs"]
mod phone;
#[path = "cli/removed_project_open.rs"]
mod removed_project_open;
#[path = "cli/removed_scripts.rs"]
mod removed_scripts;
#[path = "cli/removed_updater.rs"]
mod removed_updater;
#[cfg(unix)]
use std::{
    process::{Child, Command, Output, Stdio},
    sync::Once,
    thread,
    time::{Duration, Instant},
};

/// How long one command may take once the executable has started before.
const COMMAND_LIMIT: Duration = Duration::from_secs(10);
/// The first launch of a fresh build waits for the system to check it and,
/// for an Intel build on an Apple silicon runner, for Rosetta to translate
/// it; that cost belongs to no test's command.
const FIRST_LAUNCH_LIMIT: Duration = Duration::from_secs(120);
static FIRST_LAUNCH: Once = Once::new();

fn cli(args: &[&str]) -> Output {
    cli_in(args, None)
}

/// Runs with `state` as the only state directory, and without the variables
/// a Herdr pane would set, so no running app or daemon is ever reached.
fn cli_in(args: &[&str], state: Option<&std::path::Path>) -> Output {
    warm_up();
    let mut command = Command::new(env!("CARGO_BIN_EXE_spaceport"));
    if let Some(state) = state {
        command
            .env("XDG_STATE_HOME", state)
            .env_remove("HERDR_SOCKET_PATH")
            .env_remove("HERDR_CLIENT_SOCKET_PATH")
            .env_remove("HERDR_WORKSPACE_ID")
            .env_remove("HERDR_PANE_ID");
    }
    let child = command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch the Cargo-built GUI binary");
    wait(child, args, COMMAND_LIMIT)
}

/// Launches the executable once, so later deadlines time only commands.
fn warm_up() {
    FIRST_LAUNCH.call_once(|| {
        let child = Command::new(env!("CARGO_BIN_EXE_spaceport"))
            .arg("--help")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("launch the Cargo-built GUI binary");
        wait(child, &["--help"], FIRST_LAUNCH_LIMIT);
    });
}

fn wait(mut child: Child, args: &[&str], limit: Duration) -> Output {
    let deadline = Instant::now() + limit;
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!("CLI {args:?} did not exit before timeout: {output:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn usage_error(args: &[&str], message: &str) {
    let output = cli(args);
    // An abort or native-window failure is not a successful parser rejection.
    assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
    assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(message), "{args:?}: {stderr}");
    assert!(stderr.contains("Usage: spaceport"), "{stderr}");
}

#[test]
fn help_succeeds_and_documents_connection_options() {
    for flag in ["--help", "-h"] {
        let output = cli(&[flag]);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(
            help.starts_with("spaceport [--socket CLIENT_SOCKET]"),
            "{help}"
        );
        assert!(!help.contains("sheprd COMMAND"), "{help}");
        for option in ["--socket CLIENT_SOCKET", "--build-info"] {
            assert!(help.contains(option), "missing {option}: {help}");
        }
        assert!(!help.contains("--session"), "{help}");
        assert!(!help.contains("--dev"), "{help}");
        assert_eq!(
            help.contains("--integration-test"),
            cfg!(feature = "integration-test"),
            "{help}"
        );
    }
}

#[test]
fn build_info_is_pure_and_matches_compile_identity() {
    let output = cli(&["--build-info"]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "worktree={}\nbranch={}\npr={}\n",
            env!("HERDR_BUILD_WORKTREE"),
            env!("HERDR_BUILD_BRANCH"),
            env!("HERDR_BUILD_PR")
        )
    );
}

#[test]
fn missing_option_values_are_usage_errors() {
    usage_error(&["--socket"], "--socket requires a path");
    usage_error(&["--socket", "--help"], "--socket requires a path");
}

#[test]
fn duplicate_options_are_usage_errors() {
    usage_error(
        &["--socket", "a", "--socket", "b"],
        "--socket may only be specified once",
    );
}

#[test]
fn unknown_flags_and_positional_arguments_are_usage_errors() {
    for arg in ["--unknown", "--socket=/unused.sock", "unexpected", "--"] {
        usage_error(&[arg], &format!("Unknown option: {arg}"));
    }
}

#[test]
fn legacy_connection_flags_are_rejected_before_startup() {
    for flag in ["--session", "--dev"] {
        usage_error(&[flag], &format!("Unknown option: {flag}"));
        usage_error(
            &["--socket", "/unused.sock", flag],
            &format!("Unknown option: {flag}"),
        );
    }
}

#[cfg(not(feature = "integration-test"))]
#[test]
fn native_test_flag_is_rejected_in_normal_builds() {
    usage_error(
        &["--integration-test"],
        "Unknown option: --integration-test",
    );
    usage_error(
        &["--socket", "/unused.sock", "--integration-test"],
        "Unknown option: --integration-test",
    );
}

#[cfg(feature = "integration-test")]
#[test]
fn native_test_flag_requires_explicit_socket() {
    usage_error(
        &["--integration-test"],
        "--integration-test requires an explicit --socket",
    );
}

#[test]
fn skill_command_answers_without_a_window() {
    let output = cli(&["skill"]);
    assert!(output.status.success(), "{output:?}");
    let skill = String::from_utf8(output.stdout).unwrap();
    assert!(skill.starts_with("---\nname: spaceport\n"), "{skill}");
    // The skill names this very executable, which need not be on PATH. The
    // path may be shell-quoted, as Windows paths always are.
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_spaceport"));
    let name = exe.file_name().unwrap().to_str().unwrap();
    let command = skill
        .lines()
        .find(|line| line.contains("Check whether the app runs"))
        .unwrap_or_else(|| panic!("{skill}"));
    assert!(
        command.contains(&format!("{name} status")) || command.contains(&format!("{name}' status")),
        "{command}"
    );
    assert!(!command.starts_with("sheprd status"), "{command}");
}

#[test]
fn browser_commands_are_rejected_before_socket_lookup() {
    let state = tempfile::tempdir().unwrap();
    for command in ["open", "eval", "reload", "list", "skill", "--help"] {
        let output = cli_in(&["browser", command], Some(state.path()));
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Unknown option: browser"),
            "{output:?}"
        );
        assert!(output.stdout.is_empty(), "{output:?}");
    }
}

#[test]
fn code_open_is_rejected_before_socket_lookup() {
    let state = tempfile::tempdir().unwrap();
    let file = state.path().join("main.rs");
    std::fs::write(&file, "fn main() {}\n").unwrap();
    let output = cli_in(
        &["code", "open", file.to_str().unwrap()],
        Some(state.path()),
    );
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Unknown option: code"),
        "{output:?}"
    );
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn code_edit_is_rejected_before_socket_lookup() {
    let state = tempfile::tempdir().unwrap();
    let output = cli_in(&["code", "edit", "main.rs:42"], Some(state.path()));
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Unknown option: code"),
        "{output:?}"
    );
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn removed_design_feedback_is_rejected_before_socket_lookup() {
    let state = tempfile::tempdir().unwrap();
    let output = cli_in(&["browser", "feedback"], Some(state.path()));
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Unknown option: browser"),
        "{output:?}"
    );
    usage_error(&["feedback"], "Unknown option: feedback");
    usage_error(&["wait", "feedback"], "Unknown wait command");
    usage_error(&["--mockup"], "Unknown option: --mockup");
}

#[test]
fn control_commands_list_and_explain_themselves_without_the_app() {
    let output = cli(&["--help"]);
    let help = String::from_utf8(output.stdout).unwrap();
    for family in ["status", "window", "events", "diagnostics"] {
        assert!(help.contains(&format!("  {family} ")), "{family}: {help}");
    }
    let output = cli(&["notes", "--help"]);
    assert!(output.status.success(), "{output:?}");
    let usage = String::from_utf8(output.stdout).unwrap();
    assert!(usage.contains("spaceport notes set TEXT"), "{usage}");
    assert!(usage.contains("Exit status:"), "{usage}");
    let output = cli(&["commands", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let commands: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        commands
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command["name"] == "toggle_sidebar"),
        "{commands}"
    );
    usage_error(&["window", "size", "huge"], "Invalid WIDTHxHEIGHT");
    usage_error(&["app"], "app requires focus|hide|quit");
    usage_error(&["tab", "focus", "x"], "Invalid ID");
}

#[test]
fn control_commands_report_a_missing_app_by_status_and_in_json() {
    let parent = if cfg!(unix) {
        std::path::PathBuf::from("/tmp")
    } else {
        std::env::temp_dir()
    };
    let state = tempfile::Builder::new()
        .prefix("hgc")
        .tempdir_in(parent)
        .unwrap();
    // The state directory names a socket, or on Windows a pipe, of its own.
    let (status, code, message) = (3, "not_running", "Spaceport is not running");
    let output = cli_in(&["status"], Some(state.path()));
    assert_eq!(output.status.code(), Some(status), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(message), "{stderr}");
    let output = cli_in(&["window", "list", "--json"], Some(state.path()));
    assert_eq!(output.status.code(), Some(status), "{output:?}");
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"]["code"], code, "{error}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn usage_errors_of_a_json_command_are_json() {
    let output = cli(&["tab", "focus", "x", "--json"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"]["code"], "usage", "{error}");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Invalid ID"),
        "{error}"
    );
}

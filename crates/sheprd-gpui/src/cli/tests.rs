use super::*;
use control::BrowserCommand;

mod browser_zoom;
mod chat;
mod control_commands;
mod development_links;
mod language;
mod links;
mod removed_automation;
mod removed_checkpoint;
mod removed_cloud;
mod removed_code_edit;
mod removed_github;
mod removed_legacy_launch;
mod removed_legacy_window_session;
mod removed_mods;
mod removed_orchestrator;
mod removed_phone;
mod removed_project_open;
mod removed_px0;
mod removed_review;
mod removed_search;
mod removed_teleport;
mod ui_panel;

#[test]
fn build_info_is_an_informational_mode() {
    assert_eq!(
        LaunchOptions::parse(["--build-info"]).unwrap().mode,
        LaunchMode::BuildInfo
    );
    assert_eq!(
        build_info(),
        concat!(
            "worktree=",
            env!("HERDR_BUILD_WORKTREE"),
            "\nbranch=",
            env!("HERDR_BUILD_BRANCH"),
            "\npr=",
            env!("HERDR_BUILD_PR"),
            "\n"
        )
    );
}

/// A browser command as the parser should produce it: text output, the
/// default window.
fn browser(command: BrowserCommand) -> LaunchMode {
    LaunchMode::Control(control::Invocation {
        command: control::ControlCommand::Browser(command),
        json: false,
        window: None,
    })
}

#[test]
fn removed_browser_open_command_is_rejected() {
    assert!(matches!(
        LaunchOptions::parse(["browser", "open", "https://a.test/"]),
        Err(CliError::UnknownBrowserCommand(_))
    ));
}

#[test]
fn removed_design_feedback_commands_are_rejected() {
    assert!(matches!(
        LaunchOptions::parse(["browser", "feedback"]),
        Err(CliError::UnknownBrowserCommand(_))
    ));
    assert_eq!(
        LaunchOptions::parse(["--mockup"]).unwrap_err(),
        CliError::UnknownOption("--mockup".into())
    );
}

#[test]
fn browser_commands_never_start_the_gui() {
    let mode = |args: &[&str]| LaunchOptions::parse(args.iter().copied()).map(|o| o.mode);
    assert_eq!(
        mode(&["browser", "skill"]).unwrap(),
        browser(BrowserCommand::Skill)
    );
    assert_eq!(
        mode(&["browser", "example"]).unwrap(),
        browser(BrowserCommand::Example)
    );
    assert_eq!(
        mode(&["browser", "reload"]).unwrap(),
        browser(BrowserCommand::Reload)
    );
    assert_eq!(
        mode(&["browser", "--help"]).unwrap(),
        browser(BrowserCommand::Help)
    );
    for (args, expected) in [
        (&["browser"][..], CliError::MissingBrowserCommand),
        (
            &["browser", "eval"],
            CliError::UnknownBrowserCommand("eval".into()),
        ),
        (
            &["browser", "skill", "x"],
            CliError::UnexpectedArgument("x".into()),
        ),
        (
            &["browser", "reload", "x"],
            CliError::UnexpectedArgument("x".into()),
        ),
    ] {
        assert_eq!(mode(args).unwrap_err(), expected, "{args:?}");
    }
    // Only a leading "browser" is the subcommand.
    assert_eq!(
        mode(&["--dev", "browser"]).unwrap_err(),
        CliError::UnknownOption("--dev".into())
    );
}

#[test]
fn connection_selection() {
    assert!(matches!(
        LaunchOptions::parse(std::iter::empty::<OsString>())
            .unwrap()
            .target,
        ConnectTarget::Local
    ));
}

#[test]
fn rejects_missing_and_duplicate_values() {
    for (args, expected, message) in [
        (
            vec!["--socket"],
            CliError::MissingSocketPath,
            "--socket requires a path",
        ),
        (
            vec!["--socket", "--help"],
            CliError::MissingSocketPath,
            "--socket requires a path",
        ),
        (
            vec!["--socket", ""],
            CliError::MissingSocketPath,
            "--socket requires a path",
        ),
        (
            vec!["--socket", "a", "--socket", "b"],
            CliError::DuplicateSocket,
            "--socket may only be specified once",
        ),
        (
            vec!["--unknown"],
            CliError::UnknownOption("--unknown".into()),
            "Unknown option: --unknown",
        ),
    ] {
        let error = LaunchOptions::parse(args).unwrap_err();
        assert_eq!(error, expected);
        assert_eq!(error.to_string(), message);
    }
}

// Windows paths are UTF-16 and have no byte form to round-trip here; the
// parser stays byte-agnostic on both because it never converts to `String`.
#[cfg(unix)]
#[test]
fn socket_paths_need_not_be_utf8() {
    use std::os::unix::ffi::OsStringExt;
    let path = OsString::from_vec(b"/tmp/socket-\xff".to_vec());
    let options = LaunchOptions::parse([OsString::from("--socket"), path.clone()]).unwrap();
    assert!(matches!(options.target, ConnectTarget::Socket(actual) if actual.as_os_str() == path));
    let error = LaunchOptions::parse([path.clone()]).unwrap_err();
    assert_eq!(error, CliError::UnknownOption(path.clone()));
    assert_eq!(
        error.to_string(),
        format!("Unknown option: {}", path.to_string_lossy())
    );
}

#[cfg(feature = "integration-test")]
#[test]
fn test_modes_are_exclusive() {
    for first in ["--integration-test", "--sidebar-test", "--performance-test"] {
        for second in ["--integration-test", "--sidebar-test", "--performance-test"] {
            let error = LaunchOptions::parse([first, second]).unwrap_err();
            assert_eq!(error, CliError::ConflictingTestModes);
            assert_eq!(
                error.to_string(),
                "native test modes are mutually exclusive and may only be specified once"
            );
        }
    }
    let error = LaunchOptions::parse(["--integration-test"]).unwrap_err();
    assert_eq!(error, CliError::MissingIntegrationSocket);
    assert_eq!(
        error.to_string(),
        "--integration-test requires an explicit --socket"
    );
    for flag in ["--sidebar-test", "--performance-test"] {
        let error = LaunchOptions::parse([flag, "--socket", "/private/test.sock"]).unwrap_err();
        assert_eq!(error, CliError::ConflictingFixtureOptions);
        assert_eq!(
            error.to_string(),
            "fixture tests cannot be combined with connection options or --integration-test"
        );
    }
}

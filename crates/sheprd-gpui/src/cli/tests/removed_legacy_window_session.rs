use super::*;
use control::{ControlCommand, Invocation, WindowOp};

#[test]
fn window_new_refuses_named_legacy_session_and_keeps_explicit_window() {
    assert_eq!(
        LaunchOptions::parse(["window", "new", "--session", "legacy"]).unwrap_err(),
        CliError::Unexpected {
            command: "window",
            argument: "--session".into(),
        }
    );
    let mode = LaunchOptions::parse(["window", "new", "--window", "42"])
        .unwrap()
        .mode;
    assert!(matches!(
        mode,
        LaunchMode::Control(Invocation {
            command: ControlCommand::Window(WindowOp::New {}),
            json: false,
            window: Some(42),
        })
    ));
}

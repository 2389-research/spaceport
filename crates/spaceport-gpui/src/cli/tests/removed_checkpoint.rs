// ABOUTME: Guards the retired checkpoint CLI family.
// ABOUTME: Keeps retained work commands bound to their exact window target.
use super::*;

#[test]
fn checkpoint_command_is_rejected_before_control_dispatch() {
    for args in [
        &["checkpoint"][..],
        &["checkpoint", "create", "before"][..],
        &["checkpoint", "diff", "123"][..],
        &["checkpoint", "restore", "123"][..],
    ] {
        assert_eq!(
            LaunchOptions::parse(args.iter().copied()).unwrap_err(),
            CliError::UnknownOption("checkpoint".into())
        );
    }

    assert!(matches!(
        LaunchOptions::parse(["notes", "list", "--window", "7", "--json"])
            .unwrap()
            .mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Notes(control::WorkOp::Notes),
            json: true,
            window: Some(7),
        })
    ));
}

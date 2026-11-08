use super::*;

#[test]
fn teleport_cli_is_absent_while_worktree_targeting_remains() {
    assert_eq!(
        LaunchOptions::parse(["teleport", "--to", "other-host"]).unwrap_err(),
        CliError::UnknownOption("teleport".into())
    );
    let parsed = LaunchOptions::parse(["worktree", "mute", "--window", "7"]).unwrap();
    assert!(matches!(
        parsed.mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Worktree(control::WorktreeCommand::Mute),
            window: Some(7),
            ..
        })
    ));
}

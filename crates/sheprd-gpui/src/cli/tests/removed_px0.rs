// ABOUTME: Rejects the retired px0 command before opening a control socket.
// ABOUTME: Keeps exact targeting of a retained tab command.
use super::*;

#[test]
fn px0_is_rejected_and_tab_target_remains_exact() {
    for args in [&["px0"][..], &["px0", "open"][..], &["px0", "--help"][..]] {
        assert_eq!(
            LaunchOptions::parse(args).unwrap_err(),
            CliError::UnknownOption("px0".into())
        );
    }
    assert_eq!(
        LaunchOptions::parse(["tab", "list", "--window", "42", "--json"])
            .unwrap()
            .mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Tab(control::TabCommand::List),
            json: true,
            window: Some(42),
        })
    );
}

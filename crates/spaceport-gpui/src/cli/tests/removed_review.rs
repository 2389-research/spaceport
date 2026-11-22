use super::*;

#[test]
fn review_command_is_rejected_before_control_dispatch() {
    for args in [&["review"][..], &["review", "open"][..]] {
        assert_eq!(
            LaunchOptions::parse(args.iter().copied()).unwrap_err(),
            CliError::UnknownOption("review".into())
        );
    }

    assert!(matches!(
        LaunchOptions::parse(["devices", "--window", "7", "--json"])
            .unwrap()
            .mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Devices,
            json: true,
            window: Some(7),
        })
    ));
}

use super::*;

#[test]
fn orchestrator_command_is_rejected_before_control_dispatch() {
    for args in [
        &["orchestrator"][..],
        &["orchestrator", "open", "--repo", "/src/app"],
        &["orchestrator", "tasks", "--json", "--window", "7"],
    ] {
        assert_eq!(
            LaunchOptions::parse(args.iter().copied()).unwrap_err(),
            CliError::UnknownOption("orchestrator".into())
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

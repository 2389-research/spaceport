use super::*;

#[test]
fn cloud_commands_are_absent_while_host_and_port_targets_remain() {
    for args in [
        &["cloud"][..],
        &["cloud", "add"][..],
        &["cloud", "images", "build"][..],
        &["cloud", "create", "coder"][..],
        &["cloud", "exec", "box", "--", "true"][..],
        &["cloud", "destroy", "box"][..],
    ] {
        assert_eq!(
            LaunchOptions::parse(args.iter().copied()).unwrap_err(),
            CliError::UnknownOption("cloud".into()),
            "{args:?}"
        );
    }

    let host = LaunchOptions::parse(["host", "list", "--window", "7"]).unwrap();
    assert!(matches!(
        host.mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Host(control::HostOp::List),
            window: Some(7),
            ..
        })
    ));
    let ports = LaunchOptions::parse(["ports", "forward", "3000", "--window", "7"]).unwrap();
    assert!(matches!(
        ports.mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Ports(control::HostOp::Forward { port: 3000 }),
            window: Some(7),
            ..
        })
    ));
    assert_eq!(
        LaunchOptions::parse(["host", "list", "--", "ignored"]).unwrap_err(),
        CliError::Unexpected {
            command: "host",
            argument: "--".into(),
        }
    );
}

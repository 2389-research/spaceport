use super::*;

#[test]
fn github_commands_are_absent_while_host_and_port_targets_remain() {
    for args in [
        &["pr"][..],
        &["pr", "checks"][..],
        &["pr", "comments"][..],
        &["pr", "open"][..],
        &["pr", "quota", "--json"][..],
        &["security"][..],
        &["security", "open", "owner/repo", "42"][..],
    ] {
        assert_eq!(
            LaunchOptions::parse(args.iter().copied()).unwrap_err(),
            CliError::UnknownOption(args[0].into()),
            "{args:?}"
        );
    }

    for args in [
        &["host", "list", "--window", "7"][..],
        &["ports", "forward", "3000", "--window", "7"][..],
    ] {
        let parsed = LaunchOptions::parse(args.iter().copied()).unwrap();
        assert!(matches!(
            parsed.mode,
            LaunchMode::Control(control::Invocation {
                window: Some(7),
                ..
            })
        ));
    }
}

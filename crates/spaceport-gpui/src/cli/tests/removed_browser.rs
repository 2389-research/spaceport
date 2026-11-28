use super::*;

#[test]
fn browser_command_family_is_rejected_before_startup() {
    for args in [
        ["browser", "reload"],
        ["browser", "list"],
        ["browser", "--help"],
        ["browser", "feedback"],
        ["browser", "skill"],
    ] {
        assert_eq!(
            LaunchOptions::parse(args).unwrap_err(),
            CliError::UnknownOption("browser".into()),
            "{args:?}"
        );
    }
    assert_eq!(
        LaunchOptions::parse(["--mockup"]).unwrap_err(),
        CliError::UnknownOption("--mockup".into())
    );
}

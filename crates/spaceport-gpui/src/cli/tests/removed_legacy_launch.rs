use super::*;

#[test]
fn named_herdr_session_launch_flags_are_rejected() {
    for flag in ["--session", "--dev"] {
        let error = LaunchOptions::parse([flag]).unwrap_err();
        assert_eq!(error, CliError::UnknownOption(flag.into()));
    }
    assert_eq!(
        LaunchOptions::parse(["--session", "work"]).unwrap_err(),
        CliError::UnknownOption("--session".into())
    );
    assert_eq!(
        LaunchOptions::parse(std::iter::empty::<&str>())
            .unwrap()
            .target,
        ConnectTarget::Local
    );
    assert_eq!(
        LaunchOptions::parse(["--socket", "/private/test.sock"])
            .unwrap()
            .target,
        ConnectTarget::Socket("/private/test.sock".into())
    );
}

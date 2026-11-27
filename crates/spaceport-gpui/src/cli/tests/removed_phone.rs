use super::*;

#[test]
fn phone_control_commands_are_unavailable() {
    for args in [
        ["remote"].as_slice(),
        ["remote", "pair"].as_slice(),
        ["remote", "restart"].as_slice(),
    ] {
        assert!(
            LaunchOptions::parse(args.iter().copied()).is_err(),
            "{args:?}"
        );
    }
}

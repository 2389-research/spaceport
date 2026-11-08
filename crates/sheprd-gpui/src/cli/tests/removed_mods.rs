use super::*;

#[test]
fn client_mod_commands_are_unavailable() {
    for args in [
        ["mod"].as_slice(),
        ["mod", "check", "/tmp/example"].as_slice(),
    ] {
        assert!(
            LaunchOptions::parse(args.iter().copied()).is_err(),
            "{args:?}"
        );
    }
}

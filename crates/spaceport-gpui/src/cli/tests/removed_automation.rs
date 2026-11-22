use super::*;

#[test]
fn automation_approval_command_is_unavailable() {
    for args in [
        &["automation"][..],
        &["automation", "approve", "/tmp/fixture.luau"][..],
    ] {
        assert!(
            LaunchOptions::parse(args.iter().copied()).is_err(),
            "{args:?}"
        );
    }
}

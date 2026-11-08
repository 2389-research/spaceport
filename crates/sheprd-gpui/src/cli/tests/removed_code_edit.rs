use super::*;

#[test]
fn code_edit_is_rejected_before_control_dispatch() {
    for args in [
        &["code", "edit", "src/main.rs:42"][..],
        &["code", "--help"][..],
    ] {
        assert_eq!(
            LaunchOptions::parse(args).unwrap_err(),
            CliError::UnknownOption("code".into())
        );
    }
}

#[test]
fn tab_window_and_json_options_keep_the_exact_target() {
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

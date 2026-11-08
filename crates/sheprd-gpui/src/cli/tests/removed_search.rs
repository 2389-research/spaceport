use super::*;

#[test]
fn checkout_search_is_rejected_before_local_dispatch() {
    for args in [
        &["search", "symbol", "open_workspace"][..],
        &["search", "file", "src/main.rs"][..],
        &["search", "--help"][..],
    ] {
        assert_eq!(
            LaunchOptions::parse(args.iter().copied()).unwrap_err(),
            CliError::UnknownOption("search".into()),
            "{args:?}"
        );
    }

    assert!(matches!(
        LaunchOptions::parse(["processes", "--pane", "pane-7"])
            .unwrap()
            .mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Processes { pane: Some(pane) },
            ..
        }) if pane == "pane-7"
    ));
}

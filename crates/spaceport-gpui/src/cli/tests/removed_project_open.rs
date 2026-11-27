use super::*;

#[test]
fn removed_folder_open_command_is_rejected_before_starting_the_app() {
    for args in [["open", "."], ["open", "--help"]] {
        assert!(
            matches!(
                LaunchOptions::parse(args),
                Err(CliError::UnknownOption(word)) if word == "open"
            ),
            "{args:?}"
        );
    }
    assert!(matches!(
        LaunchOptions::parse(["app", "focus"]),
        Ok(LaunchOptions {
            mode: LaunchMode::Control(_),
            ..
        })
    ));
}

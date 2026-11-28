use super::*;

#[test]
fn removed_folder_open_exits_before_starting_the_app() {
    usage_error(&["open", "."], "Unknown option: open");
    usage_error(&["open", "--help"], "Unknown option: open");
    let help = cli(&["--help"]);
    assert!(help.status.success(), "{help:?}");
    assert!(!String::from_utf8_lossy(&help.stdout).contains("open DIR"));
}

// ABOUTME: Checks the native tab kinds advertised through command help.
// ABOUTME: Rejects references to removed page and specialized tab features.
use super::*;

#[test]
fn tab_help_lists_only_active_tab_kinds() {
    let help = help::usage(Some("tab"));
    assert!(help.contains("Devices") && help.contains("Core terminals"));
    assert!(!help.contains("pages"));
    assert!(!help.contains("reviews"));
}

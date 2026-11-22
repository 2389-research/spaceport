use super::*;

#[test]
fn tab_help_lists_only_active_tab_kinds() {
    let help = help::usage(Some("tab"));
    assert!(help.contains("Devices, Core terminals"));
    assert!(!help.contains("reviews"));
}

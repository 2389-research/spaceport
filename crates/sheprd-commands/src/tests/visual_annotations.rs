use super::*;

#[test]
fn page_annotation_command_is_absent() {
    assert!(!COMMANDS.iter().any(|info| info.name == "toggle_annotate"));
}

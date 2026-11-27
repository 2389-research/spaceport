use super::*;

#[test]
fn terminal_annotation_command_is_absent() {
    assert!(
        !COMMANDS
            .iter()
            .any(|info| info.name == "annotate_selection")
    );
}

// ABOUTME: Checks accessibility metadata on the nested tool row producer.
// ABOUTME: Keeps visible child titles and their shared derived state readable.
use super::{ToolState, step_row};
use gpui::{Element, Role, accesskit::Node, div};
use sheprd_chat_protocol::EntryId;

#[test]
fn nested_tool_steps_name_the_title_and_state() {
    let row = step_row(
        EntryId(5),
        ToolState::Running,
        "Read fixture results",
        div(),
    );
    assert_eq!(row.a11y_role(), Some(Role::ListItem));
    let mut node = Node::new(Role::ListItem);
    row.write_a11y_info(&mut node);
    assert!(node.label().is_some_and(|label| label.contains("running")));
    assert!(
        node.label()
            .is_some_and(|label| label.contains("Read fixture results"))
    );
}

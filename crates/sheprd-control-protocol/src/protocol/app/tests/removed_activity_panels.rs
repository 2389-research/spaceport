use super::*;

#[test]
fn retired_orchestrator_panels_are_not_control_targets() {
    const { assert!(crate::protocol::PROTOCOL_VERSION >= 29) };
    for name in ["pull-requests", "issues", "tasks"] {
        assert_eq!(ActivityPanel::parse(name), None, "{name}");
        assert!(serde_json::from_str::<ActivityPanel>(&format!("\"{name}\"")).is_err());
        assert!(
            serde_json::from_value::<UiOp>(serde_json::json!({
                "op": "panel",
                "panel": name,
            }))
            .is_err()
        );
    }
    for panel in [
        ActivityPanel::Workspaces,
        ActivityPanel::Overview,
        ActivityPanel::Automations,
        ActivityPanel::Usage,
    ] {
        assert!(ActivityPanel::ALL.contains(&panel));
    }
    let mut names = std::collections::HashSet::new();
    assert!(
        ActivityPanel::ALL
            .iter()
            .all(|panel| names.insert(panel.name()))
    );
}

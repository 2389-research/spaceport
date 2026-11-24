use super::snapshot;
use crate::Target;
use serde_json::json;
use spaceport_client::protocol::{ClientShellCommand, ClientShellCommandAction};

#[test]
fn invocation_uses_captured_ids_not_current_focus() {
    let mut snapshot = snapshot();
    snapshot.commands = vec![ClientShellCommand {
        command_id: "build".into(),
        action: ClientShellCommandAction::Shell,
        description: None,
        binding_label: String::new(),
        binding_labels: Vec::new(),
    }];
    let target = Target::capture(&snapshot);
    snapshot.focused_workspace_id = None;
    snapshot.focused_tab_id = None;
    snapshot.focused_pane_id = None;
    assert_eq!(
        target
            .invocation(&snapshot, "build", ClientShellCommandAction::Shell)
            .unwrap(),
        json!({
            "command_id": "build", "workspace_id": target.workspace,
            "tab_id": target.tab, "pane_id": target.pane,
        })
    );
    let empty_target = Target::capture(&snapshot);
    assert_eq!(
        empty_target
            .invocation(&snapshot, "build", ClientShellCommandAction::Shell)
            .unwrap(),
        json!({"command_id": "build"})
    );
}

#[test]
fn invocation_rejects_stale_boot_command_action_and_membership() {
    let mut original = snapshot();
    original.commands = vec![ClientShellCommand {
        command_id: "build".into(),
        action: ClientShellCommandAction::Shell,
        description: None,
        binding_label: String::new(),
        binding_labels: Vec::new(),
    }];
    let target = Target::capture(&original);
    for change in 0..7 {
        let mut snapshot = original.clone();
        match change {
            0 => snapshot.boot_id.push_str("-new"),
            1 => snapshot.commands.clear(),
            2 => snapshot.commands[0].action = ClientShellCommandAction::Pane,
            3 => snapshot.workspaces.clear(),
            4 => snapshot.tabs.clear(),
            5 => snapshot.panes.clear(),
            _ => {
                for pane in &mut snapshot.panes {
                    pane.tab_id = "foreign".into();
                }
            }
        }
        assert!(
            target
                .invocation(&snapshot, "build", ClientShellCommandAction::Shell)
                .is_err()
        );
    }
    original.commands[0].action = ClientShellCommandAction::Unknown;
    assert!(
        target
            .invocation(&original, "build", ClientShellCommandAction::Unknown)
            .is_err()
    );
}

#[test]
fn workspace_selection_rejects_removed_id_and_restarted_daemon() {
    let mut snapshot = snapshot();
    let target = Target::capture(&snapshot);
    let id = snapshot.workspaces[0].workspace_id.clone();
    assert!(target.workspace_exists(&snapshot, &id).is_ok());
    assert!(target.workspace_exists(&snapshot, "missing").is_err());
    snapshot.boot_id.push_str("-new");
    assert!(target.workspace_exists(&snapshot, &id).is_err());
}

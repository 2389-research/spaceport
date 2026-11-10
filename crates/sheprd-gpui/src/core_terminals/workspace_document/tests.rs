#![allow(clippy::unwrap_used)]

use super::*;
use serde_json::json;
use uuid::Uuid;

fn workspace_id() -> Uuid {
    Uuid::parse_str("b8b1f75d-177b-49ed-a8ce-c0124d02c851").unwrap()
}

#[test]
fn absent_document_is_an_empty_v1_grouping() {
    let document = Document::read(Value::Null).unwrap();
    assert_eq!(document.workspaces().len(), 0);
    assert_eq!(
        document.write().unwrap(),
        json!({"schema_version": 1, "workspaces": []})
    );
}

#[test]
fn rename_keeps_unknown_fields_at_every_level() {
    let id = workspace_id();
    let value = json!({
        "schema_version": 1,
        "workspaces": [{
            "id": id, "label": "Before", "archived": false, "launch_on": null,
            "members": [{"core_id": "core-a", "terminal_id": "t7", "hint": "keep"}],
            "color": "blue"
        }],
        "future": {"keep": true}
    });
    let mut document = Document::read(value).unwrap();
    document.rename(id, "After").unwrap();
    let saved = document.write().unwrap();
    assert_eq!(saved["workspaces"][0]["label"], "After");
    assert_eq!(saved["workspaces"][0]["color"], "blue");
    assert_eq!(saved["workspaces"][0]["members"][0]["hint"], "keep");
    assert_eq!(saved["future"], json!({"keep": true}));
}

#[test]
fn newer_schema_is_read_only() {
    assert!(matches!(
        Document::read(json!({"schema_version": 2, "workspaces": []})),
        Err(Error::NewerSchema(2))
    ));
}

#[test]
fn duplicate_workspace_ids_and_bad_terminal_refs_are_refused() {
    let item = json!({"id": workspace_id(), "label": "One", "members": []});
    assert!(matches!(
        Document::read(json!({"schema_version": 1, "workspaces": [item.clone(), item]})),
        Err(Error::DuplicateWorkspace(_))
    ));
    for terminal_id in ["", "lab/t7", "t7\n"] {
        let value = json!({"schema_version": 1, "workspaces": [{
            "id": workspace_id(), "label": "One",
            "members": [{"core_id": "core-a", "terminal_id": terminal_id}]
        }]});
        assert!(matches!(
            Document::read(value),
            Err(Error::InvalidTerminalRef)
        ));
    }
    for core_id in ["", "lab/core", "core\n"] {
        let value = json!({"schema_version": 1, "workspaces": [{
            "id": workspace_id(), "label": "One", "launch_on": core_id,
            "members": []
        }]});
        assert!(matches!(Document::read(value), Err(Error::InvalidCoreId)));
    }
    let value = json!({"schema_version": 1, "workspaces": [{
        "id": workspace_id(), "label": "One", "members": [
            {"core_id": "core-a", "terminal_id": "t7"},
            {"core_id": "core-a", "terminal_id": "t7"}
        ]
    }]});
    assert!(matches!(
        Document::read(value),
        Err(Error::DuplicateTerminalRef)
    ));
}

#[test]
fn future_core_terminal_ids_are_kept_verbatim() {
    let value = json!({"schema_version": 1, "workspaces": [{
        "id": workspace_id(), "label": "One", "members": [
            {"core_id": "core-a", "terminal_id": "next_7"}
        ]
    }]});
    let saved = Document::read(value).unwrap().write().unwrap();
    assert_eq!(
        saved["workspaces"][0]["members"][0]["terminal_id"],
        "next_7"
    );
}

#[test]
fn core_identity_is_opaque_too() {
    let value = json!({"schema_version": 1, "workspaces": [{
        "id": workspace_id(), "label": "One", "launch_on": "core.next",
        "members": [{"core_id": "core.next", "terminal_id": "t7"}]
    }]});
    let saved = Document::read(value).unwrap().write().unwrap();
    assert_eq!(saved["workspaces"][0]["launch_on"], "core.next");
    assert_eq!(saved["workspaces"][0]["members"][0]["core_id"], "core.next");
}

#[test]
fn nil_workspace_identity_is_refused() {
    let value = json!({"schema_version": 1, "workspaces": [{
        "id": Uuid::nil(), "label": "One", "members": []
    }]});
    assert!(matches!(
        Document::read(value),
        Err(Error::InvalidWorkspaceId)
    ));
    let mut document = Document::read(Value::Null).unwrap();
    assert!(matches!(
        document.create(Uuid::nil(), "One"),
        Err(Error::InvalidWorkspaceId)
    ));
}

#[test]
fn grouping_edits_only_change_document_values() {
    let id = workspace_id();
    let mut document = Document::read(Value::Null).unwrap();
    document.create(id, "Project").unwrap();
    assert!(matches!(
        document.create(id, "Again"),
        Err(Error::DuplicateWorkspace(_))
    ));
    document.archive(id, true).unwrap();
    assert_eq!(document.write().unwrap()["workspaces"][0]["archived"], true);
    document.delete(id).unwrap();
    assert_eq!(document.write().unwrap()["workspaces"], json!([]));
    assert!(matches!(
        document.delete(id),
        Err(Error::MissingWorkspace(_))
    ));
}

#[test]
fn membership_keeps_exact_core_and_local_terminal_identity() {
    let id = workspace_id();
    let mut document = Document::read(Value::Null).unwrap();
    document.create(id, "Project").unwrap();
    document.add_member(id, "core-a", "t7").unwrap();
    document.add_member(id, "core-b", "t7").unwrap();
    assert!(matches!(
        document.add_member(id, "core-a", "t7"),
        Err(Error::DuplicateTerminalRef)
    ));
    assert!(matches!(
        document.add_member(id, "core-a", "box/t7"),
        Err(Error::InvalidTerminalRef)
    ));
    document.remove_member(id, "core-a", "t7").unwrap();
    assert_eq!(document.workspaces()[0].members.len(), 1);
    assert_eq!(document.workspaces()[0].members[0].core_id, "core-b");
    assert!(matches!(
        document.remove_member(id, "core-a", "t7"),
        Err(Error::MissingTerminalRef)
    ));
}

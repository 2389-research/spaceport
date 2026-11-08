use super::*;
use serde_json::json;
use sheprd_backend::{AgentStatus, RunState, TerminalId};
use sheprd_core_client::wire::{Doc, Machine, MachineList, MachineView};

#[cfg(unix)]
mod live;
mod remote_document;

fn terminal(id: &str) -> TerminalInfo {
    TerminalInfo {
        agent: AgentStatus::Unknown,
        cwd: None,
        id: TerminalId::new(id).unwrap(),
        labels: Default::default(),
        name: None,
        state: RunState::Running,
        title: None,
    }
}

#[test]
fn observed_core_identity_does_not_survive_loss_or_new_epoch() {
    let mut projection = Projection::default();
    projection.begin(1);
    projection.core_id = Some("core-a".into());
    projection.initial(1, vec![terminal("t7")], Ok(document(1)));
    assert_eq!(projection.core_id(), Some("core-a"));
    projection.lost(1);
    assert_eq!(projection.core_id(), None);
    projection.begin(2);
    assert_eq!(projection.core_id(), None);
}

fn document(version: u64) -> VersionedDocument {
    parse_document(Ok(Doc {
        value: json!({"schema_version": 1, "workspaces": []}),
        version,
    }))
    .unwrap()
}

fn machine(id: &str, name: &str, state: &str) -> MachineView {
    MachineView {
        machine: Machine {
            host: "fixture.invalid".into(),
            id: id.into(),
            name: name.into(),
            session: None,
            target: "fixture.invalid".into(),
            temporary: false,
            version: "0.1.18".into(),
            via: false,
        },
        since_ms: None,
        state: state.into(),
        why: None,
    }
}

#[test]
fn core_machine_catalog_is_fresh_sorted_and_epoch_fenced() {
    let mut projection = Projection::default();
    projection.begin(1);
    projection.initial(1, vec![], Ok(document(0)));
    projection.refresh_machines(
        1,
        Ok(sort_machines(MachineList {
            machines: vec![
                machine("stable-b", "zeta", "unreachable"),
                machine("stable-a", "alpha", "connected"),
            ],
        })),
    );
    let machines = projection.machines().unwrap();
    assert_eq!(machines[0].machine.id, "stable-a");
    assert_eq!(machines[1].state, "unreachable");
    projection.lost(1);
    assert!(projection.machines().is_none());
    projection.begin(2);
    projection.initial(2, vec![], Ok(document(0)));
    projection.refresh_machines(
        1,
        Ok(MachineList {
            machines: vec![machine("stale", "wrong", "connected")],
        }),
    );
    assert!(projection.machines().is_none());
}

#[test]
fn failed_machine_refresh_clears_old_catalog_without_hiding_the_error() {
    let mut projection = Projection::default();
    projection.begin(1);
    projection.initial(1, vec![], Ok(document(0)));
    projection.refresh_machines(
        1,
        Ok(MachineList {
            machines: vec![machine("stable-a", "lab", "connected")],
        }),
    );
    projection.refresh_machines(
        1,
        Err(Error::Core(sheprd_core_client::Error::Unsupported(
            sheprd_core_client::Method::MachineList,
        ))),
    );
    assert!(projection.machines().is_none());
    assert!(matches!(
        projection.machine_error(),
        Some(Error::Core(sheprd_core_client::Error::Unsupported(
            sheprd_core_client::Method::MachineList
        )))
    ));
    assert!(
        projection.fresh(),
        "terminal facts remain independently fresh"
    );
}

#[test]
fn machine_read_mailbox_keeps_only_the_latest_result() {
    let mailbox: MachineMailbox = Arc::new(Mutex::new(None));
    replace_latest(
        &mailbox,
        Ok(MachineList {
            machines: vec![machine("old", "old", "connected")],
        }),
    );
    replace_latest(
        &mailbox,
        Ok(MachineList {
            machines: vec![machine("new", "new", "connected")],
        }),
    );
    assert_eq!(
        take_latest(&mailbox).unwrap().unwrap().machines[0]
            .machine
            .id,
        "new"
    );
    assert!(take_latest(&mailbox).is_none());
}

#[test]
fn initial_observation_has_terminal_and_document_versions() {
    let mut projection = Projection::default();
    projection.begin(1);
    projection.initial(1, vec![terminal("t1")], Ok(document(7)));
    assert_eq!(projection.terminals().len(), 1);
    assert_eq!(projection.document().unwrap().version, 7);
    assert!(projection.fresh());
}

#[test]
fn missed_snapshot_replaces_facts_and_refreshes_document() {
    let mut projection = Projection::default();
    projection.begin(1);
    projection.initial(1, vec![terminal("t1")], Ok(document(7)));
    projection.batch(
        1,
        CoreBatch {
            changes: vec![Change::Resync(vec![terminal("t2")])],
            documents: vec![],
            missed: true,
        },
        Some(Ok(document(9))),
    );
    assert_eq!(projection.terminals()[0].id.as_str(), "t2");
    assert_eq!(projection.document().unwrap().version, 9);
}

#[test]
fn old_connection_cannot_restore_stale_facts() {
    let mut projection = Projection::default();
    projection.begin(1);
    projection.initial(1, vec![terminal("t1")], Ok(document(7)));
    projection.lost(1);
    assert!(!projection.fresh());
    assert!(projection.terminals().is_empty());
    projection.begin(2);
    projection.initial(1, vec![terminal("old")], Ok(document(8)));
    assert!(projection.terminals().is_empty());
    projection.initial(2, vec![terminal("new")], Ok(document(1)));
    assert_eq!(projection.terminals()[0].id.as_str(), "new");
}

#[test]
fn only_local_spaceport_notice_or_missed_refreshes_the_document() {
    let notice = |machine: Option<&str>, name: &str| CoreBatch {
        changes: vec![],
        documents: vec![sheprd_core_client::DocumentInvalidation {
            machine: machine.map(str::to_owned),
            name: name.into(),
            version: 2,
        }],
        missed: false,
    };
    assert!(needs_document(&notice(None, "spaceport")));
    assert!(!needs_document(&notice(Some("lab"), "spaceport")));
    assert!(!needs_document(&notice(None, "other-app")));
    assert!(needs_document(&CoreBatch {
        changes: vec![],
        documents: vec![],
        missed: true
    }));
}

#[test]
fn invalid_document_clears_previous_value_and_exposes_error() {
    let mut projection = Projection::default();
    projection.begin(1);
    projection.initial(1, vec![], Ok(document(7)));
    projection.batch(
        1,
        CoreBatch {
            changes: vec![],
            documents: vec![],
            missed: true,
        },
        Some(parse_document(Ok(Doc {
            value: json!({"schema_version": 2, "workspaces": []}),
            version: 8,
        }))),
    );
    assert!(projection.document().is_none());
    assert!(projection.error().is_some());
}

#[test]
fn mailbox_keeps_only_the_latest_projection() {
    let mailbox = Arc::new(Mutex::new(None));
    let mut projection = Projection::default();
    projection.begin(1);
    publish(&mailbox, &projection);
    projection.initial(1, vec![terminal("t1")], Ok(document(2)));
    publish(&mailbox, &projection);
    let latest = take_latest(&mailbox).unwrap();
    assert_eq!(latest.terminals()[0].id.as_str(), "t1");
    assert!(take_latest(&mailbox).is_none());
}

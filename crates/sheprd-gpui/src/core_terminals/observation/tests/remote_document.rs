use super::*;

#[test]
fn selected_remote_read_uses_immutable_machine_id_and_ignores_old_results() {
    let mut remote = Remote::default();
    let connected = [machine("stable-a", "lab", "connected")];
    let first = remote
        .select(1, 1, Some("stable-a".into()), Some(&connected))
        .unwrap();
    assert_eq!(first.machine_id, "stable-a");
    let second = remote
        .invalidate(
            1,
            &CoreBatch {
                changes: vec![],
                documents: vec![sheprd_core_client::DocumentInvalidation {
                    machine: Some("renamed-lab".into()),
                    name: NAME.into(),
                    version: 2,
                }],
                missed: false,
            },
        )
        .unwrap();
    assert_eq!(second.machine_id, "stable-a");
    remote.complete(first, Ok(document(1)));
    assert!(matches!(remote.status(), Some(RemoteStatus::Loading)));
    remote.complete(second, Ok(document(2)));
    assert!(matches!(remote.status(), Some(RemoteStatus::Ready(doc)) if doc.version == 2));
}

#[test]
fn remote_document_is_stale_until_machine_reconnect_and_new_epoch_read() {
    let mut remote = Remote::default();
    let connected = [machine("stable-a", "lab", "connected")];
    let first = remote
        .select(1, 1, Some("stable-a".into()), Some(&connected))
        .unwrap();
    remote.complete(first, Ok(document(1)));
    assert!(
        remote
            .catalog(1, Some(&[machine("stable-a", "lab", "unreachable")]))
            .is_none()
    );
    assert!(matches!(remote.status(), Some(RemoteStatus::Stale)));
    let second = remote
        .catalog(1, Some(&[machine("stable-a", "work", "connected")]))
        .unwrap();
    assert_eq!(second.machine_id, "stable-a");
    remote.clear();
    remote.complete(second, Ok(document(2)));
    assert!(remote.status().is_none());
    let third = remote
        .select(2, 1, Some("stable-a".into()), Some(&connected))
        .unwrap();
    remote.complete(third, Ok(document(3)));
    assert!(matches!(remote.status(), Some(RemoteStatus::Ready(doc)) if doc.version == 3));
}

#[test]
fn missed_and_selection_change_refetch_only_the_current_id() {
    let mut remote = Remote::default();
    let machines = [
        machine("stable-a", "lab", "connected"),
        machine("stable-b", "work", "connected"),
    ];
    let first = remote
        .select(1, 1, Some("stable-a".into()), Some(&machines))
        .unwrap();
    let second = remote
        .select(1, 2, Some("stable-b".into()), Some(&machines))
        .unwrap();
    remote.complete(first, Ok(document(10)));
    assert!(matches!(remote.status(), Some(RemoteStatus::Loading)));
    remote.complete(second, Ok(document(3)));
    let missed = remote
        .invalidate(
            1,
            &CoreBatch {
                changes: vec![],
                documents: vec![],
                missed: true,
            },
        )
        .unwrap();
    assert_eq!(missed.machine_id, "stable-b");
    assert!(matches!(remote.status(), Some(RemoteStatus::Loading)));
    remote.complete(missed, Ok(document(4)));
    assert!(matches!(remote.status(), Some(RemoteStatus::Ready(doc)) if doc.version == 4));
}

#[test]
fn unrelated_remote_notices_coalesce_to_one_read_of_the_selected_id() {
    let mut remote = Remote::default();
    let machines = [
        machine("stable-a", "lab", "connected"),
        machine("stable-b", "work", "connected"),
    ];
    let initial = remote
        .select(1, 1, Some("stable-a".into()), Some(&machines))
        .unwrap();
    remote.complete(initial, Ok(document(1)));
    let request = remote
        .invalidate(
            1,
            &CoreBatch {
                changes: vec![],
                documents: vec![
                    sheprd_core_client::DocumentInvalidation {
                        machine: Some("work".into()),
                        name: NAME.into(),
                        version: 9,
                    },
                    sheprd_core_client::DocumentInvalidation {
                        machine: Some("work".into()),
                        name: NAME.into(),
                        version: 10,
                    },
                ],
                missed: false,
            },
        )
        .unwrap();
    assert_eq!(request.machine_id, "stable-a");
    assert!(matches!(remote.status(), Some(RemoteStatus::Loading)));
}

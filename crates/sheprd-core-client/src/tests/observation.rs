//! A Core event subscription carries terminal facts and document invalidations.
//! A missed batch tells the presentation reader to refetch its document.
use super::{MockCore, connected, terminal};
use crate::{CoreBackend, frame::Kind};
use serde_json::json;
use sheprd_backend::Change;
use std::{thread, time::Duration};

mod owner;

/// The caller keeps the backend alive while consuming its watch subscription.
fn watching(backend: &CoreBackend, core: &mut MockCore) -> crate::CoreChanges {
    let watch = thread::scope(|scope| {
        let watch = scope.spawn(|| backend.watch());
        let request = core.request();
        assert_eq!(request["method"], "events.subscribe");
        core.answer(
            &request,
            json!({"channel": 3, "seq": 10, "terminals": [terminal("t1")]}),
        );
        watch.join().unwrap()
    });
    watch.unwrap().1
}

#[test]
fn one_batch_carries_document_and_terminal_changes() {
    let (client, mut core) = connected();
    let backend = CoreBackend::new(client, 1000);
    let mut changes = watching(&backend, &mut core);
    let lines = [
        json!({"method": "event", "params": {"seq": 11, "type": "doc", "machine": "lab", "data": {"name": "spaceport", "version": 2}}}),
        json!({"method": "event", "params": {"seq": 12, "type": "title", "terminal": "t1", "data": {"title": "ready"}}}),
    ];
    let bytes = lines
        .iter()
        .flat_map(|line| {
            let mut bytes = serde_json::to_vec(line).unwrap();
            bytes.push(b'\n');
            bytes
        })
        .collect::<Vec<_>>();
    core.send(3, Kind::Data, &bytes);
    let batch = changes.next_batch(Duration::from_secs(5)).unwrap();
    assert_eq!(batch.documents.len(), 1);
    assert_eq!(batch.documents[0].machine.as_deref(), Some("lab"));
    assert_eq!(batch.documents[0].name, "spaceport");
    assert_eq!(batch.documents[0].version, 2);
    assert!(!batch.missed);
    assert!(
        matches!(batch.changes.as_slice(), [Change::Upsert(info)] if info.title.as_deref() == Some("ready"))
    );
}

#[test]
fn missed_snapshot_marks_documents_stale_and_fences_old_events() {
    let (client, mut core) = connected();
    let backend = CoreBackend::new(client, 1000);
    let mut changes = watching(&backend, &mut core);
    let lines = [
        json!({"method": "event", "params": {"seq": 11, "type": "doc", "data": {"name": "spaceport", "version": 2}}}),
        json!({"method": "missed", "params": {"seq": 20, "terminals": [terminal("t5")]}}),
        json!({"method": "event", "params": {"seq": 19, "type": "doc", "data": {"name": "spaceport", "version": 1}}}),
    ];
    let bytes = lines
        .iter()
        .flat_map(|line| {
            let mut bytes = serde_json::to_vec(line).unwrap();
            bytes.push(b'\n');
            bytes
        })
        .collect::<Vec<_>>();
    core.send(3, Kind::Data, &bytes);
    let batch = changes.next_batch(Duration::from_secs(5)).unwrap();
    assert!(batch.missed);
    assert!(batch.documents.is_empty());
    assert!(
        matches!(batch.changes.as_slice(), [Change::Resync(infos)] if infos.len() == 1 && infos[0].id.as_str() == "t5")
    );
}

// ABOUTME: Coordinates preference worker shutdown without mocking the file writer.
// ABOUTME: Verifies nonblocking owner drop and cleanup after rejected snapshots.
use super::*;
use crate::preferences::Preferences;
use std::{fs, sync::mpsc, thread, time::Duration};

#[test]
fn dropping_the_owner_does_not_join_a_blocked_worker() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("preferences.json");
    let owner = Arc::new(Worker {
        completed: Mutex::new(None),
        writer: Mutex::new(None),
    });
    let lifetime = owner.clone();
    let (ready, initialized) = mpsc::sync_channel(1);
    let (release, blocked) = mpsc::sync_channel(1);
    let writer =
        Writer::start_with_initialization("preferences-blocked", path.clone(), move |_| {
            ready.send(()).unwrap();
            blocked.recv().unwrap();
            lifetime
        })
        .unwrap();
    *owner.writer() = Some(writer);
    let preferences = Preferences {
        loaded: None,
        worker: Some(owner.clone()),
    };
    initialized.recv_timeout(Duration::from_secs(5)).unwrap();
    for width in 1..=256 {
        preferences.save(Chrome {
            sidebar_width: Some(width as f32),
            ..Chrome::default()
        });
    }
    let (dropped, finished) = mpsc::sync_channel(1);
    let dropper = thread::spawn(move || {
        drop(preferences);
        dropped.send(()).unwrap();
    });
    let result = finished.recv_timeout(Duration::from_secs(5));
    // Release before asserting, so a regressed UI join cannot strand the worker.
    release.send(()).unwrap();
    owner.finish();
    dropper.join().unwrap();
    assert!(result.is_ok(), "owner drop waited for disk work");
    assert_eq!(read_chrome(&path).unwrap().sidebar_width, Some(256.0));
}

#[test]
fn a_rejected_snapshot_keeps_the_file_and_removes_its_temporary() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("preferences.json");
    crate::state_file::write(
        &path,
        &Chrome {
            sidebar_width: Some(240.0),
            ..Chrome::default()
        },
    )
    .unwrap();
    let before = fs::read(&path).unwrap();
    let result = crate::state_file::write(
        &path,
        &Chrome {
            sidebar_width: Some(f32::NAN),
            ..Chrome::default()
        },
    );
    assert!(matches!(result, Err(crate::Error::Json(_))));
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(
        fs::read_dir(directory.path())
            .unwrap()
            .all(|entry| entry.unwrap().path() == path)
    );
}

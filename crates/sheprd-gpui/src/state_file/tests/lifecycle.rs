// ABOUTME: Exercises coalescing and temporary-file cleanup with real state writes.
// ABOUTME: Coordinates initialization so a burst has one observable final snapshot.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Snapshot {
    value: u32,
    writes: Arc<AtomicUsize>,
}

impl Serialize for Snapshot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        self.value.serialize(serializer)
    }
}

#[test]
fn closing_a_blocked_writer_coalesces_the_burst_before_it_finishes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state.json");
    let (ready, initialized) = mpsc::sync_channel(1);
    let (release, blocked) = mpsc::sync_channel(1);
    let writer = Writer::start_with_initialization("state-coalescing", path.clone(), move |_| {
        ready.send(()).unwrap();
        blocked.recv().unwrap();
    })
    .unwrap();
    initialized
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let writes = Arc::new(AtomicUsize::new(0));
    for value in 1..=256 {
        writer.save(Snapshot {
            value,
            writes: writes.clone(),
        });
    }
    let (closed, stopped) = mpsc::sync_channel(1);
    let closer = thread::spawn(move || {
        let mut writer = writer;
        writer.stop();
        closed.send(()).unwrap();
        writer.finish();
    });
    let result = stopped.recv_timeout(std::time::Duration::from_secs(5));
    release.send(()).unwrap();
    closer.join().unwrap();
    assert!(result.is_ok(), "stopping waited for initialization");
    assert_eq!(
        serde_json::from_slice::<u32>(&fs::read(&path).unwrap()).unwrap(),
        256
    );
    assert_eq!(writes.load(Ordering::SeqCst), 1);
    assert!(
        fs::read_dir(directory.path())
            .unwrap()
            .all(|entry| entry.unwrap().path() == path)
    );
}

#[test]
fn a_failed_replacement_removes_its_temporary_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state.json");
    fs::create_dir(&path).unwrap();
    assert!(matches!(write(&path, &[1]), Err(crate::Error::Io(_))));
    assert!(path.is_dir());
    assert!(
        fs::read_dir(directory.path())
            .unwrap()
            .all(|entry| entry.unwrap().path() == path)
    );
}

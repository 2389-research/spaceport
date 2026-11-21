#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn missing_files_are_empty_and_oversized_files_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    assert!(read(&path, 8).unwrap().is_none());
    fs::write(&path, b"123456789").unwrap();
    assert!(matches!(read(&path, 8), Err(Error::Size { limit: 8, .. })));
    assert_eq!(read(&path, 9).unwrap().unwrap(), b"123456789");
}

#[test]
fn the_latest_value_is_written_before_finish_returns() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/state.json");
    let writer = Writer::start("state-test", path.clone()).unwrap();
    writer.save(vec![1]);
    writer.save(vec![2, 3]);
    writer.finish();
    let bytes = read(&path, 1024).unwrap().unwrap();
    assert_eq!(serde_json::from_slice::<Vec<i32>>(&bytes).unwrap(), [2, 3]);
}

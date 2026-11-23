#![allow(clippy::unwrap_used)]
use super::*;

fn tagged(records: &[(&str, &[&str])]) -> Tags {
    Tags::tagged(records)
}

#[gpui::test]
fn an_edit_before_the_saved_copy_is_read_joins_it(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(FILE);
    cx.update(|cx| {
        cx.set_global(DeviceTags {
            file: Some(file.clone()),
            ..DeviceTags::default()
        });
        DeviceTags::set(cx, "a", vec!["work".to_owned()]);
    });
    cx.run_until_parked();
    // Saving now would have put one device over everyone's tags.
    assert!(!file.exists());

    cx.update(|cx| DeviceTags::finish_loading(cx, tagged(&[("a", &["old"]), ("b", &["lab"])])));
    cx.run_until_parked();
    let tags = cx.update(|cx| DeviceTags::current(cx));
    assert_eq!(tags.of("a"), ["work"]);
    assert_eq!(tags.of("b"), ["lab"]);
    let saved = read(&file);
    assert_eq!(saved.of("a"), ["work"]);
    assert_eq!(saved.of("b"), ["lab"]);
}

#[gpui::test]
fn a_slow_write_does_not_hold_up_the_next_save(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(FILE);
    let writing = Arc::new(Mutex::new(()));
    cx.update(|cx| {
        cx.set_global(DeviceTags {
            file: Some(file.clone()),
            loaded: true,
            writing: writing.clone(),
            ..DeviceTags::default()
        });
    });
    let held = writing.lock().unwrap();
    // Returns while a write has its turn, as the UI thread must.
    cx.update(|cx| DeviceTags::set(cx, "a", vec!["work".to_owned()]));
    drop(held);
    cx.run_until_parked();
    assert_eq!(read(&file).of("a"), ["work"]);
}

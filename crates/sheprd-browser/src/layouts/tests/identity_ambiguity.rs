// ABOUTME: Checks every raw layout identity before a layout-only rewrite.
// ABOUTME: Preserves ambiguous sources while keeping unrelated invalid entries recoverable.
use super::*;
use std::fs;

const GOOD: &str = r#"{"scope":"local","workspace":"good","layout":{"groups":[{"pick":{"kind":"page","id":2},"share":1}],"active":0}}"#;

fn rewrite(original: &str) -> (tempfile::TempDir, Option<u64>, bool, bool) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("editor-groups.json");
    fs::write(&path, original).unwrap();
    let mut layouts = Layouts::load(Some(path.clone()));
    let good = layouts
        .layouts
        .contains_key(&(Scope::endpoint("local"), "good".to_owned()));
    layouts.save();
    let had_writer = layouts.writer.take().is_some_and(|writer| {
        writer.finish();
        true
    });
    let saved = fs::read(&path).unwrap();
    let floor =
        serde_json::from_slice::<serde_json::Value>(&saved).unwrap()["next_tab_id"].as_u64();
    if !had_writer {
        assert_eq!(saved, original.as_bytes());
    }
    (dir, floor, had_writer, good)
}

fn assert_safe_floor(group: &str) {
    let bad = format!(
        r#"{{"scope":"local","workspace":"bad","layout":{{"groups":[{group}],"active":0}}}}"#
    );
    let original = format!(r#"{{"layouts":[{GOOD},{bad}]}}"#);
    let (_dir, floor, had_writer, good) = rewrite(&original);
    assert!(
        !had_writer || floor.is_some_and(|next| next >= 301),
        "an earlier layout identity occurrence was discarded: {floor:?}"
    );
    assert!(good);
}

#[test]
fn duplicate_page_ids_never_erase_the_earlier_reservation() {
    assert_safe_floor(r#"{"pick":{"kind":"page","id":300,"id":3},"share":1}"#);
}

#[test]
fn duplicate_page_picks_never_erase_the_earlier_reservation() {
    assert_safe_floor(
        r#"{"pick":{"kind":"page","id":300},"pick":{"kind":"page","id":3},"share":1}"#,
    );
}

#[test]
fn duplicate_page_kinds_never_erase_either_reservation() {
    for pick in [
        r#"{"kind":"page","kind":"herdr","id":300}"#,
        r#"{"kind":"herdr","kind":"page","id":300}"#,
    ] {
        assert_safe_floor(&format!(r#"{{"pick":{pick},"share":1}}"#));
    }
}

#[test]
fn nonobject_rejected_layouts_and_groups_leave_valid_siblings_writable() {
    for bad in [
        "false",
        r#"{"scope":"local","workspace":"bad","layout":{"groups":[42],"active":0}}"#,
    ] {
        let original = format!(r#"{{"layouts":[{GOOD},{bad}]}}"#);
        let (_dir, floor, had_writer, good) = rewrite(&original);
        assert!(good);
        assert!(had_writer);
        assert_eq!(floor, Some(3));
    }
}

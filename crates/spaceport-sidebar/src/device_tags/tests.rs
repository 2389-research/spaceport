#![allow(clippy::unwrap_used)]
use super::*;

mod local;

fn tagged(records: &[(&str, &[&str])]) -> Tags {
    Tags::tagged(records)
}

#[test]
fn every_tag_is_listed_once_alphabetically() {
    let tags = tagged(&[("a", &["work", "Lab"]), ("b", &["Work", "client"])]);
    assert_eq!(tags.all(), ["client", "Lab", "work"]);
    assert!(tags.any());
    assert!(!Tags::default().any());
}

#[test]
fn a_device_lists_once_under_its_first_tag_or_the_filter() {
    let tags = tagged(&[("a", &["personal", "work"]), ("b", &[])]);
    assert_eq!(tags.section("a", None), Some("personal"));
    assert_eq!(tags.section("a", Some("WORK")), Some("WORK"));
    assert_eq!(tags.section("a", Some("lab")), Some("personal"));
    assert_eq!(tags.section("b", None), None);
    assert_eq!(tags.section("missing", None), None);
}

#[test]
fn sections_sort_alphabetically_with_untagged_last() {
    let tags = tagged(&[("a", &["work"]), ("b", &["personal"]), ("c", &[])]);
    let all = tags.all();
    let rank = |id| tags.rank(id, None, &all);
    assert!(rank("b") < rank("a"));
    assert!(rank("a") < rank("c"));
}

#[test]
fn moving_replaces_the_section_and_keeps_the_rest() {
    let tags = tagged(&[("a", &["work", "lab"]), ("b", &["Personal"])]);
    assert_eq!(tags.moved_to("a", Some("personal")), ["Personal", "lab"]);
    assert_eq!(tags.moved_to("a", Some("lab")), ["lab"]);
    assert_eq!(tags.moved_to("a", None), Vec::<String>::new());
    assert_eq!(tags.moved_to("c", Some("new")), ["new"]);
}

#[test]
fn toggling_adds_and_removes_one_tag() {
    let tags = tagged(&[("a", &["work"]), ("b", &["Lab"])]);
    assert_eq!(tags.toggled("a", "lab"), ["work", "Lab"]);
    assert_eq!(tags.toggled("a", "WORK"), Vec::<String>::new());
}

#[test]
fn an_edit_that_changes_nothing_is_not_an_edit() {
    let mut tags = tagged(&[("a", &["work"])]);
    assert!(!tags.edit("a", vec!["work".into()], 50));
    assert_eq!(tags.records["a"].updated_ms, 10);
    assert!(tags.edit("a", vec!["lab".into()], 50));
    assert_eq!(tags.records["a"].updated_ms, 50);
}

#[test]
fn the_saved_copy_reads_back_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("deep").join(FILE);
    assert_eq!(read(&file), Tags::default());

    let tags = tagged(&[("ssh:a", &["work"])]);
    write(&file, &tags).unwrap();
    assert_eq!(read(&file), tags);

    std::fs::write(&file, r#"{"ssh:a":{"tags":["a","A"],"updated_ms":3}}"#).unwrap();
    assert_eq!(read(&file).of("ssh:a"), ["a"]);

    std::fs::write(&file, b"not json").unwrap();
    assert_eq!(read(&file), Tags::default());
}

#[test]
fn starters_are_offered_until_a_tag_is_in_use() {
    assert_eq!(offered(&Tags::default()), ["personal", "work"]);
    assert_eq!(offered(&tagged(&[("a", &["lab"])])), ["lab"]);
}

#[test]
fn a_saved_edit_newer_than_an_early_one_stands() {
    let mut saved = Tags::default();
    saved.edit("a", vec!["saved".to_owned()], 2_000);
    let mut early = Tags::default();
    early.edit("a", vec!["early".to_owned()], 1_000);
    assert_eq!(saved.with_edits(&early).of("a"), ["saved"]);
}

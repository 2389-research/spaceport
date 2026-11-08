use super::*;

#[test]
fn saved_local_tags_keep_the_existing_shape_and_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(FILE);
    std::fs::write(
        &file,
        r#"{"ssh:a":{"tags":["  Work  ","work","lab"],"updated_ms":42}}"#,
    )
    .unwrap();
    let tags = read(&file);
    assert_eq!(tags.of("ssh:a"), ["Work", "lab"]);
    write(&file, &tags).unwrap();
    let stored: serde_json::Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(
        stored,
        serde_json::json!({"ssh:a": {"tags": ["Work", "lab"], "updated_ms": 42}})
    );
}

#[test]
fn local_tag_rules_keep_first_spelling_and_limit_count_and_length() {
    let input = [
        "  Work  ",
        "work",
        "abcdefghijklmnopqrstuvwxy",
        "a",
        "b",
        "c",
        "d",
        "e",
        "f",
        "g",
    ];
    let record = ComputerTags::new(input, 42);
    assert_eq!(record.tags.len(), 8);
    assert_eq!(record.tags[0], "Work");
    assert_eq!(record.tags[1], "abcdefghijklmnopqrstuvwx");
    assert_eq!(record.tags.last().unwrap(), "f");
    assert_eq!(record.updated_ms, 42);
}

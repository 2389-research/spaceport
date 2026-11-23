#![allow(clippy::unwrap_used)]
use super::*;

fn checkout(branch: &str) -> Checkout {
    Checkout {
        endpoint: "local".into(),
        repo_key: "/r/.git".into(),
        branch: branch.into(),
    }
}

#[test]
fn saved_notes_load_back_and_damaged_files_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gpui").join(FILE);
    let mut notes = Notes::at(Some(path.clone()));
    notes.set(checkout("feat"), "remember me");
    notes.writer.take().unwrap().finish();
    let reopened = Notes::at(Some(path.clone()));
    assert_eq!(
        reopened.get("local", "/r/.git", "feat"),
        Some("remember me")
    );

    let unclean = br#"{"notes":[{"endpoint":"local","repo_key":"/r/.git","branch":"b","text":"two\nlines"}]}"#;
    assert!(matches!(
        parse(unclean),
        Err(crate::Error::InvalidWorktreeNotes)
    ));
    let empty = br#"{"notes":[{"endpoint":"","repo_key":"/r/.git","branch":"b","text":"x"}]}"#;
    assert!(matches!(
        parse(empty),
        Err(crate::Error::InvalidWorktreeNotes)
    ));
    std::fs::write(&path, b"{not json").unwrap();
    assert_eq!(Notes::at(Some(path)).recent().count(), 0);
}

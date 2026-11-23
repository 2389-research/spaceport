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
fn notes_are_one_bounded_line_of_safe_text() {
    assert_eq!(clean("  wait for\n\tthe FX PR  "), "wait for the FX PR");
    assert_eq!(clean("a\u{202e}b\u{0007}c"), "abc");
    assert_eq!(clean(" \n "), "");
    assert_eq!(
        clean(&"x".repeat(MAX_CHARS + 10)).chars().count(),
        MAX_CHARS
    );
}

#[test]
fn setting_replaces_moves_to_most_recent_and_empty_removes() {
    let mut notes = Notes::default();
    assert!(notes.set(checkout("feat"), "first"));
    assert!(notes.set(checkout("fix"), "second"));
    assert!(!notes.set(checkout("feat"), " first "), "unchanged text");
    assert!(notes.set(checkout("feat"), "edited"));
    assert_eq!(notes.get("local", "/r/.git", "feat"), Some("edited"));
    assert_eq!(notes.get("ssh:box", "/r/.git", "feat"), None);
    let order: Vec<_> = notes.recent().map(|note| note.text.as_str()).collect();
    assert_eq!(order, ["edited", "second"]);
    assert!(notes.set(checkout("feat"), "  \n"));
    assert_eq!(notes.get("local", "/r/.git", "feat"), None);
    assert!(!notes.set(checkout("absent"), ""), "nothing to remove");
}

#[test]
fn the_oldest_edits_go_first_beyond_the_bound() {
    let mut notes = Notes::default();
    for index in 0..MAX_NOTES + 3 {
        notes.set(checkout(&format!("b{index}")), "note");
    }
    assert_eq!(notes.recent().count(), MAX_NOTES);
    assert_eq!(notes.get("local", "/r/.git", "b0"), None);
    assert!(notes.get("local", "/r/.git", "b3").is_some());
}

#[test]
fn saved_notes_the_store_would_not_write_are_refused() {
    let text = "two\nlines";
    let unclean = Note {
        checkout: checkout("b"),
        text: text.into(),
    };
    assert!(Notes::from_saved(vec![unclean]).is_none());

    let empty = Note {
        checkout: Checkout {
            endpoint: String::new(),
            ..checkout("b")
        },
        text: "x".into(),
    };
    assert!(Notes::from_saved(vec![empty]).is_none());

    let too_many: Vec<Note> = (0..MAX_NOTES + 1)
        .map(|index| Note {
            checkout: checkout(&format!("b{index}")),
            text: "note".into(),
        })
        .collect();
    assert!(Notes::from_saved(too_many).is_none(), "over the bound");
    assert!(Notes::from_saved(Vec::new()).is_some());
}

#[test]
fn the_largest_collection_the_store_accepts_reads_back() {
    // Quotes double when escaped, and the text is all four-byte characters.
    let field = "\"".repeat(MAX_FIELD_BYTES);
    let mut notes = Notes::default();
    for index in 0..MAX_NOTES {
        let mut endpoint = field.clone();
        endpoint.replace_range(..8, &format!("{index:08}"));
        let checkout = Checkout {
            endpoint,
            repo_key: field.clone(),
            branch: field.clone(),
        };
        assert!(notes.set(checkout, &"\u{1F4DD}".repeat(MAX_CHARS)));
    }
    let bytes = serde_json::to_vec(&notes.saved()).unwrap();
    assert!(
        bytes.len() as u64 <= MAX_FILE_BYTES,
        "{} bytes",
        bytes.len()
    );
    let saved: Saved = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        Notes::from_saved(saved.notes).unwrap().recent().count(),
        MAX_NOTES
    );

    // A key too long to bound is refused rather than saved unreadable.
    let long = Checkout {
        branch: "b".repeat(MAX_FIELD_BYTES + 1),
        ..checkout("x")
    };
    let revision = notes.revision();
    assert!(!notes.set(long, "note"));
    assert_eq!(notes.revision(), revision);
}

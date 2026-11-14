#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn old_source_payload_is_checked_before_its_tab_is_discarded() {
    let absolute = std::env::temp_dir().join("main.rs");
    let saved = |path: &str| {
        serde_json::json!({"kind": "code", "file": {"path": path, "line": 4}}).to_string()
    };
    let location: SavedLocation = serde_json::from_str(&saved(absolute.to_str().unwrap())).unwrap();
    assert!(matches!(location, SavedLocation::Code { .. }));
    for bad in ["", "relative/main.rs", "/a\nb.rs"] {
        assert!(
            serde_json::from_str::<SavedLocation>(&saved(bad)).is_err(),
            "{bad:?}"
        );
    }
}

// Windows paths need a drive to be absolute.
#[cfg(unix)]
#[test]
fn retired_local_payloads_keep_their_path_checks() {
    let home = Some(Path::new("/Users/me"));
    assert!(servable(
        Path::new("/Users/me/project/design"),
        "mockup v2.html",
        home
    ));
    for (root, entry) in [
        ("relative", "page.html"),
        ("/Users/me", "page.html"),
        ("/Users", "page.html"),
        ("/", "page.html"),
        ("/Users/me/.secret", "page.html"),
        ("/Users/me/project", ".env"),
        ("/Users/me/project/..", "page.html"),
    ] {
        assert!(!servable(Path::new(root), entry, home), "{root}/{entry}");
    }
}

// Windows paths need a drive to be absolute.
#[cfg(unix)]
#[test]
fn saved_local_files_are_checked_again() {
    let saved = r#"{"kind":"local","file":{"root":"/","entry":"etc"}}"#;
    assert!(serde_json::from_str::<SavedLocation>(saved).is_err());
    let saved = r#"{"kind":"local","file":{"root":"/srv/site","entry":"index.html"}}"#;
    assert!(serde_json::from_str::<SavedLocation>(saved).is_ok());
}

#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn old_source_payload_is_checked_before_its_tab_is_discarded() {
    let absolute = std::env::temp_dir().join("main.rs");
    let saved = |path: &str| {
        serde_json::json!({"kind": "code", "file": {"path": path, "line": 4}}).to_string()
    };
    let location: Location = serde_json::from_str(&saved(absolute.to_str().unwrap())).unwrap();
    assert!(matches!(location, Location::RetiredSource { .. }));
    for bad in ["", "relative/main.rs", "/a\nb.rs"] {
        assert!(
            serde_json::from_str::<Location>(&saved(bad)).is_err(),
            "{bad:?}"
        );
    }
}

#[cfg(unix)]
fn file(path: &str) -> crate::Result<LocalFile> {
    LocalFile::new(Path::new(path), Some(Path::new("/Users/me")))
}

// Windows paths need a drive to be absolute.
#[cfg(unix)]
#[test]
fn retired_local_payloads_keep_validated_paths() {
    let page = file("/Users/me/project/design/mockup v2.html").unwrap();
    assert_eq!(page.root, Path::new("/Users/me/project/design"));
    assert_eq!(page.entry, "mockup v2.html");
    let location = Location::Local { file: page };
    assert_eq!(location.default_title(), "mockup v2.html");
    for refused in [
        "relative/page.html",
        "/Users/me/page.html",
        "/Users/page.html",
        "/page.html",
        "/Users/me/.secret/page.html",
        "/Users/me/project/.env",
        "/Users/me/project/../page.html",
    ] {
        assert!(file(refused).is_err(), "{refused}");
    }
}

// Windows paths need a drive to be absolute.
#[cfg(unix)]
#[test]
fn saved_local_files_are_checked_again() {
    let saved = r#"{"kind":"local","file":{"root":"/","entry":"etc"}}"#;
    assert!(serde_json::from_str::<Location>(saved).is_err());
    let saved = r#"{"kind":"local","file":{"root":"/srv/site","entry":"index.html"}}"#;
    assert!(serde_json::from_str::<Location>(saved).is_ok());
}

use super::*;

#[test]
fn a_reported_path_is_only_a_staging_directory_and_a_plain_file() {
    for path in [
        "/tmp/herdr-upload.abcdefghijkl/file.iso",
        "/private/tmp/herdr-upload.123456ABCDEF/screenshot.png",
        "/herdr-upload.123456ABCDEF/file",
        "/a ' $ x/herdr-upload.123456ABCDEF/-a'$(not-executed); file",
    ] {
        assert!(staged_directory_of(path).is_some(), "{path:?}");
    }
    for path in [
        "",
        "/",
        "/tmp",
        "/tmp/herdr-upload.abcdefghijkl",
        "/tmp/herdr-upload.abcdefghijkl/",
        "/tmp/herdr-upload.abcdefghijkl/.",
        "/tmp/herdr-upload.abcdefghijkl/..",
        "/tmp/herdr-upload.abcdefghijkl/../file",
        "/tmp/../herdr-upload.abcdefghijkl/file",
        "/tmp/./herdr-upload.abcdefghijkl/file",
        "//tmp/herdr-upload.abcdefghijkl/file",
        "/tmp//herdr-upload.abcdefghijkl/file",
        "/tmp/herdr-upload.abcdefghijkl//file",
        "tmp/herdr-upload.abcdefghijkl/file",
        "/tmp/herdr-upload.short/file",
        "/tmp/herdr-upload.abcdefghijk;/file",
        "/tmp/herdr-upload.abcdefghijklm/file",
        "/tmp/herdr-upload.abcdefghijkl/sub/file",
        "/tmp/herdr-upload.abcdefghijkl/file\nother",
        "/tmp/herdr-upload.abcdefghijkl/file\0",
    ] {
        assert!(staged_directory_of(path).is_none(), "{path:?}");
    }
    assert!(
        staged_directory_of(&format!(
            "/tmp/herdr-upload.abcdefghijkl/{}",
            "x".repeat(LINE_LIMIT)
        ))
        .is_none()
    );
}

#[test]
fn a_staged_file_is_the_name_the_script_reported() {
    let path = "/private/tmp/herdr-upload.AbcD12345678/screenshot.png";
    assert_eq!(
        staged_directory_of(path),
        Some("/private/tmp/herdr-upload.AbcD12345678")
    );
    assert!(is_staged_file(path, "screenshot.png"));
    assert!(!is_staged_file(path, "other.png"));
    assert!(!is_staged_file(path, ""));
    // A name with a separator can never be the last component.
    assert!(!is_staged_file(path, "tmp/screenshot.png"));
}

#[test]
fn both_scripts_stage_under_the_hosts_own_temporary_directory() {
    let receive = receive_script("file.iso", 7);
    let retain = retain_script("screenshot.png", 7);
    for script in [&receive, &retain] {
        // One definition of the directory: resolved through symlinks, and
        // never a hard-coded /tmp that macOS would spell differently.
        assert!(script.contains("base=$(CDPATH= cd -P -- \"${TMPDIR:-/tmp}\" && pwd -P)"));
        assert!(script.contains("dir=$(mktemp -d \"${base%/}/herdr-upload.XXXXXXXXXXXX\")"));
        assert!(script.contains("[ \"$bytes\" = 7 ]"));
        assert!(script.contains("set -C\n"));
    }
    assert_ne!(receive, retain);
}

#[test]
fn each_script_prints_only_the_directives_its_client_parses() {
    let receive = receive_script("file.iso", 7);
    assert!(receive.contains(&format!("printf '{READY}\\n%s\\n' \"$dir\"")));
    assert!(receive.contains(&format!("[ \"$accept\" = {ACCEPT} ]")));
    assert!(receive.contains(&format!("printf '{CREATED}\\n%s\\n' \"$file\"")));
    assert!(receive.contains(&format!("printf '{COMPLETE}\\n%s\\n' \"$file\"")));
    assert!(!receive.contains(RETAINED));

    let retain = retain_script("screenshot.png", 7);
    assert!(retain.contains(&format!("[ \"$decision\" = {RETAIN} ]")));
    assert!(retain.contains(&format!("printf '{RETAINED}\\n'")));
    assert!(retain.contains(&format!("[ \"$decision\" = {DISCARD} ]")));
    assert!(!retain.contains(READY));
}

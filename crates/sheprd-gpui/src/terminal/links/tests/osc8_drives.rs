// ABOUTME: Distinguishes native Windows drive targets from external URI schemes.
// ABOUTME: Keeps explicit-link bounds and document-link location parsing intact.
use super::*;

#[test]
fn explicit_drive_links_preserve_both_native_separator_forms() {
    for destination in [
        r"C:\repo\sample.rs",
        "d:/repo/sample.rs",
        r"E:\repo\a file.md",
    ] {
        assert_eq!(
            explicit_target(destination),
            RowTarget::Path {
                path: destination.into(),
                line: None,
                explicit: true
            }
        );
    }
}

#[test]
fn one_letter_uri_schemes_remain_external() {
    for destination in [
        "x://host/document",
        "c://service/path",
        "x:opaque",
        "c:relative",
    ] {
        assert!(
            matches!(explicit_target(destination), RowTarget::External(_)),
            "{destination}"
        );
        assert!(file_target(destination).is_none(), "{destination}");
    }
}

#[test]
fn explicit_drive_links_keep_path_length_and_control_bounds() {
    let limit = format!("C:/{}", "x".repeat(MAX_PATH_BYTES - 3));
    assert!(matches!(explicit_target(&limit), RowTarget::Path { .. }));
    assert_eq!(explicit_target(&(limit + "x")), RowTarget::Malformed);
    for destination in ["C:/repo/\nfile", "C:/repo/\0file", "C:/repo/\u{0085}file"] {
        assert_eq!(explicit_target(destination), RowTarget::Malformed);
    }
}

#[test]
fn document_drive_links_keep_locations_and_refuse_network_or_device_paths() {
    assert_eq!(
        file_target(r"C:\repo\sample.rs:42:7"),
        Some(RowTarget::Path {
            path: r"C:\repo\sample.rs".into(),
            line: Some(42),
            explicit: true
        })
    );
    for destination in [
        r"\\server\share\file",
        r"\\?\C:\repo\file",
        r"\\.\pipe\file",
        "//server/share/file",
    ] {
        assert!(file_target(destination).is_none(), "{destination}");
    }
}

// ABOUTME: Verifies private sibling backups preserve exact rejected bytes.
// ABOUTME: Checks that backup failures retain their typed I/O cause.
use super::*;

#[test]
fn preserves_distinct_copies_of_rejected_bytes_beside_the_source() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let first = b"\xff\x00broken\n";
    let second = b"{";
    fs::write(&path, first).unwrap();
    let one = preserve_rejected(&path, first).unwrap();
    fs::write(&path, second).unwrap();
    let two = preserve_rejected(&path, second).unwrap();

    assert_ne!(one, two);
    for (backup, bytes) in [(&one, first.as_slice()), (&two, second.as_slice())] {
        assert_eq!(backup.parent(), Some(dir.path()));
        assert!(
            backup
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("state.json.rejected-")
        );
        assert_eq!(fs::read(backup).unwrap(), bytes);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(backup).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    assert_eq!(fs::read(path).unwrap(), second);
}

#[test]
fn reports_the_typed_cause_when_the_backup_cannot_be_created() {
    use std::error::Error as _;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing").join("state.json");
    let error = preserve_rejected(&path, b"{").unwrap_err();
    assert!(matches!(&error, Error::Io(_)));
    let source = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
}

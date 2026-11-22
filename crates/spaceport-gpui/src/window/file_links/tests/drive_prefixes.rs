// ABOUTME: Resolves canonical Windows drive paths without probing incomplete prefixes.
// ABOUTME: Keeps network and device prefixes outside the local file route.
use super::*;

#[test]
fn canonical_drive_paths_reach_their_document() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("notes.txt");
    std::fs::write(&file, "private fixture").unwrap();
    let canonical = file.canonicalize().unwrap();
    assert!(matches!(
        canonical.components().next(),
        Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::VerbatimDisk(_))
    ));
    assert_eq!(real_path(&canonical), Ok(canonical.clone()));
    assert_eq!(opened(&canonical), Some(canonical));
}

#[test]
fn remote_prefixes_are_refused_before_resolving_a_file() {
    for path in [
        r"\\spaceport.invalid\share",
        r"\\spaceport.invalid\share\notes.txt",
        r"\\?\UNC\spaceport.invalid\share\notes.txt",
        r"\\.\UNC\spaceport.invalid\share\notes.txt",
        r"\\?\GLOBALROOT\Device\Mup\spaceport.invalid\share\notes.txt",
    ] {
        assert_eq!(
            real_path(Path::new(path)),
            Err(Unresolved::Remote),
            "{path}"
        );
    }
}

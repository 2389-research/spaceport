// ABOUTME: Checks settings replacement retains the file's Unix access identity.
// ABOUTME: Uses a real supplementary group to expose temporary-file group changes.
use super::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[test]
fn replacement_preserves_owner_and_group_for_direct_and_linked_files() {
    let dir = tempfile::tempdir().unwrap();
    for linked in [false, true] {
        let target = dir.path().join(format!("settings-{linked}.toml"));
        fs::write(&target, "old").unwrap();
        let original = fs::metadata(&target).unwrap();
        let group = rustix::process::getgroups()
            .unwrap()
            .into_iter()
            .find(|group| group.as_raw() != original.gid());
        let group = group.unwrap_or_else(|| {
            eprintln!("ownership boundary unavailable: no different supplementary group; checking existing identity");
            rustix::fs::Gid::from_raw(original.gid())
        });
        let file = fs::File::open(&target).unwrap();
        rustix::fs::fchown(&file, None, Some(group)).unwrap();
        file.set_permissions(fs::Permissions::from_mode(0o640))
            .unwrap();
        let before = file.metadata().unwrap();
        let path = if linked {
            let link = dir.path().join("config.local.toml");
            std::os::unix::fs::symlink(&target, &link).unwrap();
            link
        } else {
            target.clone()
        };
        replace(&path, "new").unwrap();
        let after = fs::metadata(&target).unwrap();
        assert_eq!(after.uid(), before.uid());
        assert_eq!(
            after.gid(),
            before.gid(),
            "replacement changed the access group"
        );
        assert_eq!(after.permissions().mode() & 0o777, 0o640);
        assert_eq!(fs::read_to_string(&target).unwrap(), "new");
        if linked {
            assert_eq!(fs::read_link(&path).unwrap(), target);
        }
    }
}

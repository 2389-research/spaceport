// ABOUTME: Checks chat-startup diagnostics and shipped-helper lookup.
// ABOUTME: Preserves localized product/helper names and existing helper precedence.
use super::*;
use crate::{chat::start, i18n::Localize};
use sheprd_i18n::{Locale, Pseudo};

const HELPER: &str = if cfg!(windows) {
    "sheprd-acp.exe"
} else {
    "sheprd-acp"
};

#[test]
fn chat_start_failures_name_spaceport() {
    for (error, expected) in [
        (
            StartError::NotConnected,
            "Spaceport is not connected".into(),
        ),
        (
            StartError::ProgramMissing,
            format!("{HELPER} was not found beside Spaceport or on PATH"),
        ),
        (
            StartError::ProgramUnknown,
            format!("Spaceport is still looking for {HELPER}"),
        ),
    ] {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn localized_chat_start_failures_preserve_product_and_helper_names() {
    for (error, helper) in [
        (StartError::NotConnected, None),
        (StartError::ProgramMissing, Some(HELPER)),
        (StartError::ProgramUnknown, Some(HELPER)),
    ] {
        for locale in [
            Locale::FALLBACK,
            Pseudo::Accented.locale(),
            Pseudo::Bidi.locale(),
        ] {
            crate::i18n::show(&locale);
            let shown = error.localize();
            crate::i18n::show(&Locale::FALLBACK);
            assert!(shown.contains("Spaceport"), "{locale:?}: {shown}");
            assert!(!shown.contains("Sheprd"), "{locale:?}: {shown}");
            if let Some(helper) = helper {
                assert!(shown.contains(helper), "{locale:?}: {shown}");
            }
        }
    }
}

#[test]
fn chat_lookup_finds_shipped_helper_without_losing_legacy_precedence() {
    fn helper(dir: &std::path::Path, name: &str) -> PathBuf {
        let file = dir.join(if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.into()
        });
        std::fs::write(&file, b"lookup fixture; never executed").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        file
    }
    let bundle = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let exe = bundle.path().join("spaceport");
    let path = std::env::join_paths([external.path()]).unwrap();
    helper(external.path(), "sheprd-acp");
    let legacy = helper(bundle.path(), "herdr-acp");
    assert_eq!(start::locate_in(Some(&exe), Some(&path)), Some(legacy));
    let shipped = helper(bundle.path(), "sheprd-acp");
    assert_eq!(start::locate_in(Some(&exe), Some(&path)), Some(shipped));
}

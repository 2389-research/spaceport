// ABOUTME: Checks app identities and launcher metadata against their canonical sources.
// ABOUTME: Keeps development and signed preview contracts distinct.
use std::{fs, path::Path};

use super::*;

#[test]
fn running_development_app_has_its_own_os_identity() {
    if option_env!("SPACEPORT_MACOS_PREVIEW_VERSION").is_some() {
        assert_eq!(
            (APP_ID, RELEASE_BUILD),
            (include_str!("../../macos-preview-app-id").trim(), false)
        );
    } else {
        assert_eq!(APP_ID, include_str!("../../development-app-id").trim());
    }
    assert_eq!(LINUX_APP_ID, APP_ID);
    assert_ne!(APP_ID, "so.pen.herdr-gpui");
    assert_eq!(WINDOW_TITLE, "Spaceport");
}

#[test]
fn linux_desktop_entry_identifies_the_development_app() -> anyhow::Result<()> {
    let entry = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/release")
        .join("org.sheprd.Sheprd.desktop");
    let entry = fs::read_to_string(entry)?;
    let class = format!(
        "StartupWMClass={}",
        include_str!("../../development-app-id").trim()
    );
    assert!(entry.lines().any(|line| line == class), "{entry}");
    Ok(())
}

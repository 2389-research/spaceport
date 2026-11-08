use std::{fs, path::Path};

use super::*;

#[test]
fn running_development_app_has_its_own_os_identity() {
    assert_eq!(APP_ID, "spaceport.development");
    assert_eq!(LINUX_APP_ID, APP_ID);
    assert_ne!(APP_ID, "so.pen.herdr-gpui");
    assert_eq!(WINDOW_TITLE, "Spaceport");
}

/// The inherited release desktop entry must not identify a development window.
#[test]
fn the_upstream_linux_desktop_entry_does_not_identify_the_development_app() -> anyhow::Result<()> {
    let entry = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/release")
        .join("org.sheprd.Sheprd.desktop");
    let entry = fs::read_to_string(entry)?;
    let class = format!("StartupWMClass={LINUX_APP_ID}");
    assert!(!entry.lines().any(|line| line == class), "{entry}");
    Ok(())
}

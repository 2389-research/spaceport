// ABOUTME: Checks app identities and launcher metadata against their canonical sources.
// ABOUTME: Keeps development and signed preview contracts distinct.
use std::{fs, path::Path};

use super::*;

// The build script's selector, for the channel names it emits.
#[allow(dead_code)]
#[path = "../../preview_identity.rs"]
mod preview_identity;

#[test]
fn channel_parses_the_build_scripts_two_names_and_nothing_else() {
    assert_eq!(
        Channel::parse(preview_identity::DEVELOPMENT),
        Some(Channel::Development)
    );
    assert_eq!(
        Channel::parse(preview_identity::MACOS_PREVIEW),
        Some(Channel::MacosPreview)
    );
    for name in [
        "",
        "dev",
        "Development",
        "development ",
        "macos_preview",
        "preview",
        "release",
        "ai.2389.spaceport",
    ] {
        assert_eq!(Channel::parse(name), None, "{name:?}");
    }
}

#[test]
fn compiled_channel_is_the_one_the_selector_picks_for_this_build() {
    let selected = preview_identity::select(
        option_env!("SPACEPORT_MACOS_PREVIEW_VERSION"),
        std::env::consts::OS,
    )
    .map(|identity| (Channel::parse(identity.channel), identity.app_id));
    assert_eq!(selected, Some((Some(CHANNEL), APP_ID)));
}

#[test]
fn running_development_app_has_its_own_os_identity() {
    match CHANNEL {
        Channel::MacosPreview => {
            assert_eq!(APP_ID, include_str!("../../macos-preview-app-id").trim());
        }
        Channel::Development => {
            assert_eq!(APP_ID, include_str!("../../development-app-id").trim());
        }
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

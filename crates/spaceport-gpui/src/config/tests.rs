// ABOUTME: Verifies configuration loading, migration and preserved user overrides.
// ABOUTME: Shares private filesystem fixtures across configuration regression modules.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use anyhow::Context as _;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};

mod diagnostic_text;
mod features;
mod font_aliases;
mod font_weight;
mod fonts;
mod keybindings;
mod keybindings_save;
mod missing_fonts;
mod omarchy_theme;
mod theme_files;

/// The diagnostic as English reads it, without the isolation marks Fluent
/// puts around each placeable.
fn plain(diagnostic: Option<String>) -> Option<String> {
    diagnostic.map(|text| text.replace(['\u{2068}', '\u{2069}'], ""))
}

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new() -> std::io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let path = env::temp_dir().join(format!(
                "herdr-theme-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

mod removed_updater;


// ABOUTME: Verifies configuration loading, migration and preserved user overrides.
// ABOUTME: Shares private filesystem fixtures across configuration regression modules.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use anyhow::Context as _;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};

mod accent;
mod activity_bar;
mod agent_skill;
mod agents_header;
mod agents_placement;
mod backdrop;
mod bitmap_fonts;
mod bold_color;
mod branding;
mod browser_page;
mod change_counts;
mod cjk_fonts;
mod core_opt_in;
mod default_fonts;
mod device_footer;
mod device_icons;
mod device_theme;
mod diagnostic_text;
mod discovery;
mod edit;
mod favorite_themes;
mod features;
mod finished_notifications;
mod focus_follows_mouse;
mod follows_herdr;
mod font_aliases;
mod font_weight;
mod fonts;
mod generated_guidance;
mod herdr_2_terminals;
mod icon_font;
mod keybindings;
mod language;
mod legacy_cloud;
mod line_height;
mod loading;
mod lock_access;
mod missing_fonts;
mod notification_settings;
mod omarchy_theme;
mod preferences;
mod removed_checkpoints;
mod removed_github;
mod removed_mods;
mod removed_phone;
mod removed_resource_dashboard;
mod removed_usage_sign_in;
mod removed_workspace_close;
mod removed_worktree_cleanup;
mod repository_icons;
mod retired_azure;
mod retired_boat;
mod retired_boxd;
mod retired_coder;
mod retired_daytona;
mod retired_ide;
mod retired_opensandbox;
mod retired_railway;
mod retired_sprites;
mod retired_terminal_editor;
mod sidebar_scope;
mod sidebar_search;
mod sidebar_settings;
mod sidebar_style;
mod sound_volume;
mod ssh_hosts;
mod status_bar;
mod status_bar_awake_scope;
mod status_icons;
mod system_themes;
mod tab_labels;
mod tab_layout;
mod terminal_annotations;
mod terminal_padding;
mod theme_directory;
mod theme_files;
mod themes;
mod visual_annotations;
mod worktrees;
mod zen;

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

mod removed_chat_lifecycle;

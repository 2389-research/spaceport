// ABOUTME: Verifies configuration loading, migration and preserved user overrides.
// ABOUTME: Shares private filesystem fixtures across configuration regression modules.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::config::{MANAGED_HEADER, app_config_path};
use crate::contrast::Contrast;
use crate::daemon::MAX_DAEMON_CONFIG_BYTES;
use crate::notifications::ClipboardToastSettings;
use crate::*;
use anyhow::Context as _;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

mod accent;
mod activity_bar;
mod agent_skill;
mod agents_header;
mod agents_placement;
mod appearance;
mod backdrop;
mod bitmap_fonts;
mod bold_color;
mod branding;
mod browser_page;
mod change_counts;
mod checked_write;
mod cjk_fonts;
mod core_opt_in;
mod default_fonts;
mod device_footer;
mod device_icons;
mod device_theme;
mod discovery;
mod edit;
mod favorite_themes;
mod finished_notifications;
mod focus_follows_mouse;
mod follows_herdr;
mod generated_guidance;
mod herdr_2_terminals;
mod icon_font;
mod language;
mod legacy_cloud;
mod line_height;
mod loading;
mod lock_access;
mod notification_settings;
mod preference_round_trip;
mod preferences;
mod quiet_after;
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
mod save_queue;
mod settings_catalog;
mod settings_drafts;
mod settings_sections;
mod sidebar_draft;
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
mod theme_names;
mod themes;
mod visual_annotations;
mod worktrees;
mod zen;

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

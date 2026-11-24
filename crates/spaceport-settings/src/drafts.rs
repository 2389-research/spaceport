//! The edits Settings and the theme picker make before anything is written:
//! the theme being previewed, the layout chosen, the sidebar's live spacing.
//!
//! Each draft carries a revision, so a load that started before a choice was
//! committed cannot put the older value back over it. The app keeps them in
//! its one global; nothing here knows a window exists.

use crate::sidebar_draft::SidebarIntent;
use crate::{Config, LayoutMode, Theme};

/// What an unsaved edit shows: the drafts themselves, and nothing that draws.
#[derive(Default)]
pub struct Drafts {
    // keep-sorted start
    layout: LayoutDraft,
    sidebar: SidebarDraft,
    theme: ThemeDraft,
    // keep-sorted end
}

#[derive(Default)]
struct LayoutDraft {
    active: bool,
    mode: Option<LayoutMode>,
    revision: u64,
}

#[derive(Default)]
struct SidebarDraft {
    active: bool,
    intent: Option<SidebarIntent>,
    revision: u64,
}

#[derive(Default)]
struct ThemeDraft {
    active: bool,
    /// Unadjusted colors, so a concurrent contrast edit is applied once.
    appearance: Option<(String, Theme)>,
    revision: u64,
}

impl Drafts {
    /// Whether a previewed theme is the one every window shows.
    pub fn theme_pending(&self) -> bool {
        self.theme.active
    }

    /// The revision a load started now will be fenced against.
    pub fn theme_load_revision(&self) -> u64 {
        self.theme.revision
    }

    /// Puts the previewed theme over whatever a load read.
    pub fn apply_theme_draft(&self, config: &mut Config, theme: &mut Theme) {
        if !self.theme_pending() {
            return;
        }
        if let Some((name, raw)) = &self.theme.appearance {
            config.theme = name.clone();
            *theme = raw.clone().styled_for(config);
        }
    }

    /// Applies a load's theme, unless a draft — live, or committed after this
    /// read began — has a newer one.
    pub fn apply_loaded_theme(&self, config: &mut Config, theme: &mut Theme, revision: u64) {
        if self.theme_pending() {
            self.apply_theme_draft(config, theme);
        } else if revision != self.theme.revision
            && let Some((name, raw)) = &self.theme.appearance
        {
            // This read began before the final draft was committed. Its other
            // fields remain usable, but its older theme must not replace the commit.
            config.theme = name.clone();
            *theme = raw.clone().styled_for(config);
        }
    }

    /// A theme choice began: its preview replaces any earlier one.
    pub fn begin_theme(&mut self) {
        if !self.theme.active {
            self.theme.appearance = None;
        }
        self.theme.active = true;
        self.theme.revision = self.theme.revision.wrapping_add(1);
    }

    /// The colors a resolved choice previews, before contrast or fonts.
    pub fn preview_theme(&mut self, name: String, theme: Theme) {
        self.theme.appearance = Some((name, theme));
    }

    /// The theme is written, or abandoned: the next load's word stands.
    pub fn clear_theme(&mut self) {
        self.theme.active = false;
        self.theme.revision = self.theme.revision.wrapping_add(1);
    }

    /// The revision a load started now will be fenced against.
    pub fn layout_load_revision(&self) -> u64 {
        self.layout.revision
    }

    /// Applies a load's layout mode, unless a draft has a newer one.
    pub fn apply_loaded_layout(&self, config: &mut Config, revision: u64) {
        if (self.layout.active || self.layout.revision != revision)
            && let Some(mode) = self.layout.mode
        {
            // Reads started before commit must not put the old layout back either.
            config.layout.mode = mode;
        }
    }

    pub fn begin_layout(&mut self, mode: LayoutMode) {
        self.layout.active = true;
        self.layout.mode = Some(mode);
        self.layout.revision = self.layout.revision.wrapping_add(1);
    }

    pub fn clear_layout(&mut self) {
        self.layout.active = false;
        self.layout.revision = self.layout.revision.wrapping_add(1);
    }

    /// The revision a load started now will be fenced against.
    pub fn sidebar_load_revision(&self) -> u64 {
        self.sidebar.revision
    }

    /// Applies a load's sidebar style, except the keys a draft changed.
    pub fn apply_loaded_sidebar(&self, config: &mut Config, revision: u64) {
        if (self.sidebar.active || self.sidebar.revision != revision)
            && let Some(intent) = &self.sidebar.intent
        {
            // Reads started before a commit must not put the old spacing back,
            // nor the draft's copy of a key it never changed over the file's.
            intent.overlay(config);
        }
    }

    /// The Sidebar page's live edit, which every window shows until it is saved.
    pub fn edit_sidebar(&mut self, intent: SidebarIntent) {
        self.sidebar.active = true;
        self.sidebar.intent = Some(intent);
        self.sidebar.revision = self.sidebar.revision.wrapping_add(1);
    }

    /// The draft is written, or abandoned. The intent stays for a read that
    /// began before the commit, which must still overlay it.
    pub fn clear_sidebar(&mut self) {
        self.sidebar.active = false;
        self.sidebar.revision = self.sidebar.revision.wrapping_add(1);
    }

    /// The draft is dropped unwritten, so no later read overlays it.
    pub fn discard_sidebar(&mut self) {
        self.clear_sidebar();
        self.sidebar.intent = None;
    }
}

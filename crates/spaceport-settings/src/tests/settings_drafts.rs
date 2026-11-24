//! A draft outlives the load that was already running when it was committed:
//! its revision fences every value an older read would otherwise bring back,
//! and a page's draft takes only the keys it changed from a later read.

use crate::drafts::Drafts;
use crate::sidebar_draft::{SidebarIntent, SidebarTouched};
use crate::{Config, LayoutMode, Theme};

fn nord() -> Theme {
    Theme::builtin("Nord").expect("a bundled theme")
}

#[test]
fn a_previewed_theme_is_what_every_window_shows() {
    let mut drafts = Drafts::default();
    drafts.begin_theme();
    drafts.preview_theme("Nord".to_owned(), nord());
    assert!(drafts.theme_pending());

    let mut config = Config::default();
    let mut theme = Theme::default();
    drafts.apply_loaded_theme(&mut config, &mut theme, drafts.theme_load_revision());
    assert_eq!(config.theme, "Nord");
    assert_ne!(theme, Theme::default(), "the preview's colors are shown");
}

#[test]
fn a_committed_theme_survives_a_read_that_started_before_it() {
    let mut drafts = Drafts::default();
    let revision = drafts.theme_load_revision();
    drafts.begin_theme();
    drafts.preview_theme("Nord".to_owned(), nord());
    drafts.clear_theme();
    assert!(!drafts.theme_pending());

    // The read began before the commit, so its older theme must not win.
    let mut older = Config::default();
    let mut older_theme = Theme::default();
    drafts.apply_loaded_theme(&mut older, &mut older_theme, revision);
    assert_eq!(older.theme, "Nord");

    // A read begun after the commit takes the file's word.
    let mut newer = Config::default();
    let mut newer_theme = Theme::default();
    drafts.apply_loaded_theme(&mut newer, &mut newer_theme, drafts.theme_load_revision());
    assert_eq!(newer.theme, Config::default().theme);
    assert_eq!(newer_theme, Theme::default());
}

#[test]
fn a_chosen_layout_survives_a_read_that_started_before_it() {
    let mut drafts = Drafts::default();
    let revision = drafts.layout_load_revision();
    drafts.begin_layout(LayoutMode::Orca);
    drafts.clear_layout();

    let mut older = Config::default();
    drafts.apply_loaded_layout(&mut older, revision);
    assert_eq!(older.layout.mode, LayoutMode::Orca);

    let mut newer = Config::default();
    drafts.apply_loaded_layout(&mut newer, drafts.layout_load_revision());
    assert_eq!(newer.layout.mode, Config::default().layout.mode);
}

#[test]
fn a_sidebar_draft_takes_only_its_touched_keys_from_a_later_read() {
    let mut drafts = Drafts::default();
    let revision = drafts.sidebar_load_revision();
    let saved = Config::default();
    let mut intent = SidebarIntent::of(&saved);
    intent.style.overrides.indent = Some(20.);
    intent.touched = SidebarTouched::between(&saved.sidebar_style, &intent.style);
    drafts.edit_sidebar(intent);

    // Someone else edits a key the draft never touched.
    let mut read = Config::default();
    read.sidebar_style.overrides.gap = Some(8.);
    drafts.apply_loaded_sidebar(&mut read, revision);
    assert_eq!(read.sidebar_style.overrides.indent, Some(20.));
    assert_eq!(
        read.sidebar_style.overrides.gap,
        Some(8.),
        "an untouched key keeps the file's value"
    );
}

#[test]
fn a_cleared_sidebar_draft_still_fences_a_read_that_started_before_it() {
    let mut drafts = Drafts::default();
    let revision = drafts.sidebar_load_revision();
    let saved = Config::default();
    let mut intent = SidebarIntent::of(&saved);
    intent.style.overrides.indent = Some(20.);
    intent.touched = SidebarTouched::between(&saved.sidebar_style, &intent.style);
    drafts.edit_sidebar(intent);
    drafts.clear_sidebar();

    let mut older = Config::default();
    drafts.apply_loaded_sidebar(&mut older, revision);
    assert_eq!(
        older.sidebar_style.overrides.indent,
        Some(20.),
        "the commit's spacing must not be put back by an older read"
    );

    let mut newer = Config::default();
    drafts.apply_loaded_sidebar(&mut newer, drafts.sidebar_load_revision());
    assert_eq!(newer.sidebar_style.overrides.indent, None);

    // A draft dropped unwritten is never overlaid again.
    drafts.discard_sidebar();
    let mut after = Config::default();
    drafts.apply_loaded_sidebar(&mut after, revision);
    assert_eq!(after.sidebar_style.overrides.indent, None);
}

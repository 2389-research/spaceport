//! A sidebar draft writes only the keys it changed, so a reload keeps every
//! other key as the file read it, while a quit still writes a key dragged
//! back to the value the file already had.

use crate::sidebar_draft::{SidebarIntent, SidebarTouched, diff_preferences, touched_preferences};
use crate::{Config, Preference};

/// A saved config and a draft of it that indents rows and colours one host.
fn saved_and_draft() -> (Config, SidebarIntent) {
    let mut saved = Config::default();
    saved.sidebar_style.hosts.insert("Work".into(), 0x112233);
    let mut intent = SidebarIntent::of(&saved);
    intent.style.overrides.indent = Some(20.);
    intent.touched = SidebarTouched::between(&saved.sidebar_style, &intent.style);
    (saved, intent)
}

#[test]
fn a_commit_writes_only_the_keys_that_differ_from_the_file() {
    let (saved, intent) = saved_and_draft();
    assert_eq!(
        diff_preferences(&saved, &intent),
        [Preference::SidebarIndent(Some(20.))]
    );
}

#[test]
fn a_key_dragged_back_to_the_file_s_value_is_still_written_on_quit() {
    let (saved, mut intent) = saved_and_draft();
    intent.style.overrides.indent = None;
    assert_eq!(
        diff_preferences(&saved, &intent),
        [],
        "a commit writes nothing that matches the file"
    );
    assert_eq!(
        touched_preferences(&intent),
        [Preference::SidebarIndent(None)],
        "a quit cannot know what an in-flight save left, so it writes the revert"
    );
}

#[test]
fn a_host_added_then_dropped_again_is_written_as_removed() {
    let mut saved = Config::default();
    saved.sidebar_style.hosts.insert("Work".into(), 0x112233);
    let mut intent = SidebarIntent::of(&saved);
    intent.style.hosts.clear();
    intent.touched = SidebarTouched::between(&saved.sidebar_style, &intent.style);
    assert_eq!(
        touched_preferences(&intent),
        [Preference::HostColor {
            host: "Work".into(),
            color: None,
        }]
    );
}

#[test]
fn a_rebase_keeps_the_touched_keys_and_takes_the_rest_from_the_file() {
    let (saved, mut intent) = saved_and_draft();
    intent.style.hosts.insert("Work".into(), 0x999999);
    intent.touched = SidebarTouched::between(&saved.sidebar_style, &intent.style);
    let mut loaded = Config::default();
    loaded.sidebar_style.overrides.gap = Some(8.);
    loaded.sidebar_style.hosts.insert("Home".into(), 0x445566);
    intent.rebase(&loaded);
    assert_eq!(intent.style.overrides.indent, Some(20.));
    assert_eq!(intent.style.overrides.gap, Some(8.));
    assert_eq!(
        intent.style.hosts.get("Home").copied(),
        Some(0x445566),
        "a host the draft never touched is the file's"
    );
    assert_eq!(
        intent.style.hosts.get("Work").copied(),
        Some(0x999999),
        "a host the draft changed keeps the draft's colour"
    );
}

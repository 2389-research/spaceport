// ABOUTME: Checks that Settings pages and the search catalog declare the same rows.
// ABOUTME: Compares rendered row ids and drawn text with what search indexes.
//! The catalog is each row's only declaration: every row a page draws has an
//! entry, every entry is drawn, and the page shows the text search matches.
use super::*;
use crate::config::ThemeName;
use crate::i18n::leak_tests::{draw, under_en_xa};
use crate::settings_window::search::RowBounds;
use gpui::{px, size};
use std::collections::BTreeSet;

/// Rows built from data rather than the catalog: one per SSH host.
fn is_dynamic(id: &str) -> bool {
    id.starts_with("tabs-pane-history-host-")
}

/// Rows whose page text is not the entry's description, each for a reason
/// the entry cannot carry: arguments, a state the row reports, or data.
const DESCRIPTION_OVERRIDES: &[&str] = &[
    // Names the pair's two themes.
    "theme-side",
    // Describes the chosen backdrop mode.
    "theme-backdrop",
    // Says the skill is not installed until it is.
    "general-browser-skill",
    // Shows each file's path.
    "config-local",
    "config-shared",
];

/// Entries search opens that are not rows: a heading, the cards and a search
/// box, which show their title but no description.
const ANCHORS: &[&str] = &[
    "indicators-sets",
    "integrations-search",
    "sidebar-host-colours",
];

/// Puts the window in the state where each page renders every row this
/// platform has.
fn show_every_row(view: &mut SettingsWindow, cx: &mut App) {
    crate::core_feature::install(true, cx);
    view.config.features.acp_chats = true;
    view.config.theme = ThemeName::system("Catppuccin Latte", "Catppuccin Mocha");
}

fn selector(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

#[gpui::test]
fn pages_and_catalog_declare_the_same_rows(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.simulate_resize(size(px(960.), px(4000.)));
    cx.update(|_, cx| view.update(cx, |view, cx| show_every_row(view, cx)));
    let mut uncatalogued = BTreeSet::new();
    let mut unrendered = BTreeSet::new();
    for section in Section::ALL.iter().copied() {
        cx.update(|_, cx| {
            cx.default_global::<RowBounds>().0.clear();
            view.update(cx, |view, _| view.section = section);
        });
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        let rendered: BTreeSet<String> = cx.update(|_, cx| {
            cx.default_global::<RowBounds>()
                .0
                .keys()
                .map(ToString::to_string)
                .collect()
        });
        let entries: Vec<_> = catalog::CATALOG
            .iter()
            .filter(|entry| entry.section == section)
            .collect();
        for id in &rendered {
            if !is_dynamic(id) && !entries.iter().any(|entry| entry.id == id) {
                uncatalogued.insert(format!("{section:?}: {id}"));
            }
        }
        for entry in entries {
            let available = view.read_with(cx, |view, cx| view.available(entry.availability, cx));
            if available && cx.debug_bounds(entry.id).is_none() {
                unrendered.insert(format!("{section:?}: {}", entry.id));
            }
        }
    }
    assert!(
        uncatalogued.is_empty(),
        "rows the pages render without a catalog entry: {uncatalogued:#?}"
    );
    assert!(
        unrendered.is_empty(),
        "catalog entries no page renders: {unrendered:#?}"
    );
}

#[test]
fn pages_draw_the_text_search_indexes() {
    under_en_xa("Settings rows match search", |cx, recorder| {
        let main = cx.add_window(crate::sidebar::layout_tests::fixture_window);
        let weak = cx.update(|cx| main.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
        let (view, cx) = cx.add_window_view(|_, cx| {
            let mut view = SettingsWindow::new(weak, cx);
            show_every_row(&mut view, cx);
            view
        });
        cx.simulate_resize(size(px(1200.), px(4000.)));
        // Every line drawn so far: GPUI shapes a line once and reuses it, so
        // a later frame does not shape an earlier frame's text again.
        let mut shown = BTreeSet::new();
        let mut drifted = BTreeSet::new();
        for section in Section::ALL.iter().copied() {
            cx.update(|_, cx| view.update(cx, |view, _| view.section = section));
            draw(cx);
            shown.extend(recorder.take());
            for entry in catalog::CATALOG.iter().filter(|e| e.section == section) {
                let id = entry.id;
                let available =
                    view.read_with(cx, |view, cx| view.available(entry.availability, cx));
                if !available {
                    continue;
                }
                let (title, description) = cx.update(|_, _| (entry.title(), entry.description()));
                // A heading is drawn in capitals.
                if !shown.contains(&title) && !shown.contains(&title.to_uppercase()) {
                    drifted.insert(format!("{id}: the page draws another title"));
                }
                // A disabled row shows why in place of its description.
                let disabled = cx.debug_bounds(selector(format!("{id}-reason"))).is_some();
                if disabled || DESCRIPTION_OVERRIDES.contains(&id) || ANCHORS.contains(&id) {
                    continue;
                }
                let drawn = cx
                    .debug_bounds(selector(format!("{id}-description")))
                    .is_some();
                if description.is_empty() {
                    if drawn {
                        drifted.insert(format!("{id}: the page draws a description search lacks"));
                    }
                } else if !drawn {
                    drifted.insert(format!("{id}: the page draws no description"));
                } else if !shown.contains(&description) {
                    drifted.insert(format!("{id}: the page draws another description"));
                }
            }
        }
        assert!(
            drifted.is_empty(),
            "rows whose page text is not what search indexes: {drifted:#?}"
        );
    });
}

/// Rows the pages rendered before the catalog knew them: a word of each
/// title finds the row wherever the page renders it.
#[gpui::test]
fn search_finds_rows_the_catalog_once_lacked(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.update(|_, cx| view.update(cx, |view, cx| show_every_row(view, cx)));
    let rows = [
        (
            "general-window-controls",
            "window controls",
            cfg!(target_os = "linux"),
        ),
        ("general-chat-layout", "chat layout", true),
        ("general-chat-full-width", "full-width", true),
        ("general-chat-width", "chat width", true),
        ("general-chat-launcher", "launcher", true),
        ("general-chat-glide", "smooth chat", true),
        ("sidebar-activity-bar", "activity bar", true),
    ];
    for (id, query, rendered) in rows {
        let (known, offered) = view.update(cx, |view, cx| {
            view.set_search_query(query, cx);
            let has = |results: &[usize]| {
                results
                    .iter()
                    .any(|index| catalog::CATALOG[*index].id == id)
            };
            (
                has(view.search_results.as_deref().unwrap_or_default()),
                has(&view.visible_results(cx)),
            )
        });
        assert!(known, "{query:?} does not find {id}");
        assert_eq!(offered, rendered, "{query:?} offering {id}");
    }
    // The chat rows follow the ACP chats flag, and the width its switch.
    view.update(cx, |view, cx| {
        let offers = |view: &mut SettingsWindow, cx: &mut Context<SettingsWindow>, id: &str| {
            view.visible_results(cx)
                .iter()
                .any(|index| catalog::CATALOG[*index].id == id)
        };
        view.set_search_query("chat width", cx);
        view.config.chat.max_width = 0;
        assert!(
            !offers(view, cx, "general-chat-width"),
            "full width hides it"
        );
        view.config.chat.max_width = crate::config::chat::DEFAULT_MAX_WIDTH;
        view.config.features.acp_chats = false;
        assert!(!offers(view, cx, "general-chat-width"));
        view.set_search_query("chat layout", cx);
        assert!(!offers(view, cx, "general-chat-layout"));
    });
}

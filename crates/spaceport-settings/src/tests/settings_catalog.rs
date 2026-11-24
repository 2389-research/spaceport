//! The catalog declares each Settings row once: its id says which page shows
//! it, and search matches the messages it carries through the two lookups the
//! app passes for this window's language and for English.

use crate::Feature;
use crate::catalog::{CATALOG, search};
use crate::section::{Group, Section};

/// The app's wording, stood in for by the id itself.
fn name(id: &str) -> String {
    id.to_owned()
}

fn nothing(_: &str) -> String {
    String::new()
}

#[test]
fn search_matches_a_row_s_title_description_and_page() {
    let hits = search("sidebar-indent", name, nothing);
    assert_eq!(hits.len(), 1);
    assert_eq!(CATALOG[hits[0]].id, "sidebar-indent");

    // A page's own message is matched too, so every row it holds is a hit.
    let tabs = search("settings-section-tabs-and-panes", name, nothing);
    assert!(
        tabs.iter()
            .all(|index| CATALOG[*index].section == Section::Tabs)
    );
    assert!(tabs.len() > 1);

    // Results come back in catalog order.
    let mut sorted = tabs.clone();
    sorted.sort_unstable();
    assert_eq!(tabs, sorted);

    assert!(search("", name, name).is_empty());
    assert!(search("   ", name, name).is_empty());
    assert!(search("zzzz", name, name).is_empty());
}

#[test]
fn search_matches_english_through_the_second_lookup() {
    let english = |id: &str| {
        if id == "settings-sidebar-row-gap-description" {
            "row gap".to_owned()
        } else {
            String::new()
        }
    };
    let hits = search("row gap", nothing, english);
    assert_eq!(hits.len(), 1);
    assert_eq!(CATALOG[hits[0]].id, "sidebar-row-gap");
    assert!(
        search("row gap", nothing, nothing).is_empty(),
        "the page's own language alone does not find it"
    );
}

#[test]
fn every_feature_flag_has_a_general_catalog_entry() {
    for feature in Feature::ALL {
        assert!(
            CATALOG
                .iter()
                .any(|entry| entry.id == feature.setting_id() && entry.section == Section::General),
            "{}",
            feature.key()
        );
    }
}

#[test]
fn every_catalog_id_is_unique_and_belongs_to_its_page() {
    let mut seen = std::collections::HashSet::new();
    for entry in CATALOG {
        assert!(seen.insert(entry.id), "duplicate {}", entry.id);
        let prefix = match entry.section {
            Section::Theme => "theme-",
            Section::Sidebar => "sidebar-",
            Section::Indicators => "indicators-",
            Section::Fonts => "fonts-",
            Section::General => "general-",
            Section::Tabs => "tabs-",
            Section::Notifications => "notif-",
            Section::StatusBar => "status-",
            Section::Integrations => "integrations-",
            Section::Configuration => "config-",
        };
        assert!(entry.id.starts_with(prefix), "{}", entry.id);
    }
}

#[test]
fn every_catalog_row_sits_under_one_of_the_groups_or_none() {
    for entry in CATALOG {
        let group = entry.section.group();
        assert!(
            group.is_none_or(|group| Group::ALL.contains(&group)),
            "{} under {group:?}",
            entry.id
        );
    }
}

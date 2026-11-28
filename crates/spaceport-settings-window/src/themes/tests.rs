//! The browser's listing, the row it keeps, and the value the system
//! appearance builds: rules that need no window.
#![allow(clippy::unwrap_used)]
use super::*;
use core::prelude::v1::test;
use spaceport_settings::Theme;
use spaceport_settings::theme::FOLLOW_HERDR;

fn app_names() -> Vec<String> {
    ["Dracula", "Nord", "Nord Light"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn rows(listing: &[Choice]) -> Vec<(&str, Scope)> {
    listing
        .iter()
        .map(|choice| (choice.name.as_str(), choice.scope))
        .collect()
}

#[test]
fn listing_lists_every_word_of_the_query_across_both_libraries() {
    let mut names = app_names();
    names.push("/native/file".into());
    let matching = listing(&names, "nord", Sources::ALL);
    assert_eq!(
        rows(&matching),
        [
            ("Nord", Scope::App),
            ("nord", Scope::Herdr),
            ("Nord Light", Scope::App),
        ]
    );
    // Herdr's library is listed by its own names, never by the app's.
    let herdr_only = listing(&names, "catppuccin", Sources::ALL);
    assert_eq!(herdr_only.len(), 2);
    assert!(herdr_only.iter().all(|choice| choice.scope == Scope::Herdr));
}

#[test]
fn listing_keeps_a_library_out_when_it_is_not_listed() {
    let names = app_names();
    let app = listing(
        &names,
        "",
        Sources {
            ghostty: true,
            herdr: false,
        },
    );
    assert_eq!(app.len(), names.len());
    assert!(app.iter().all(|choice| choice.scope == Scope::App));
    let herdr = listing(
        &names,
        "",
        Sources {
            ghostty: false,
            herdr: true,
        },
    );
    assert_eq!(herdr.len(), herdr_settings::THEME_NAMES.len());
    assert!(herdr.iter().all(|choice| choice.scope == Scope::Herdr));
}

#[test]
fn the_last_library_cannot_be_turned_off() {
    let mut sources = Sources::ALL;
    assert!(!sources.set(false, false));
    assert_eq!(sources, Sources::ALL);
    assert!(sources.set(true, false));
    assert_eq!(
        sources,
        Sources {
            ghostty: true,
            herdr: false
        }
    );
}

#[test]
fn the_choice_survives_a_search_that_still_lists_it() {
    let browsed = listing(&app_names(), "", Sources::ALL);
    let nord = Choice {
        scope: Scope::App,
        name: "Nord".into(),
    };
    assert_eq!(
        chosen(&browsed, Some(&nord)).map(|index| browsed[index].name.clone()),
        Some("Nord".into())
    );
    let searched = listing(&app_names(), "light", Sources::ALL);
    assert_eq!(
        chosen(&searched, Some(&nord)),
        Some(0),
        "a choice the query dropped falls back to the first row"
    );
    assert_eq!(chosen(&[], Some(&nord)), None);
    assert_eq!(chosen(&browsed, None), Some(0));
}

#[test]
fn merged_names_are_listed_once_in_name_order() {
    let mut names = vec!["Nord".to_owned(), "dracula".to_owned()];
    merged(
        &mut names,
        [
            "Nord Light".to_owned(),
            "Nord".to_owned(),
            "Dracula".to_owned(),
        ],
    );
    assert_eq!(names, ["Dracula", "dracula", "Nord", "Nord Light"]);
}

#[test]
fn following_the_system_pairs_the_theme_with_one_of_the_defaults() {
    let dark = Theme::builtin("Default").unwrap().background;
    let light = Theme::builtin(system::LIGHT_PAIR).unwrap().background;
    assert_eq!(
        system::toggled("Default", dark, true),
        format!("light:{},dark:Default", system::LIGHT_PAIR)
    );
    assert_eq!(
        system::toggled("Nord", light, false),
        format!("light:Nord,dark:{}", system::DARK_PAIR)
    );
    // Turning it off keeps the side the system shows.
    let pair = format!("light:Nord,dark:{}", system::DARK_PAIR);
    assert_eq!(system::toggled(&pair, light, true), "Nord");
    assert_eq!(system::toggled(&pair, light, false), system::DARK_PAIR);
}

#[test]
fn one_brightness_is_chosen_at_a_time() {
    let pair = format!("light:Nord,dark:{}", system::DARK_PAIR);
    assert_eq!(system::single(&pair, true, false).as_deref(), Some("Nord"));
    assert_eq!(
        system::single(&pair, false, true).as_deref(),
        Some(system::DARK_PAIR)
    );
    // A single theme of the brightness asked for is already the answer.
    assert_eq!(system::single("Nord", true, true), None);
    // The other brightness gives way to that brightness's default.
    assert_eq!(
        system::single("Nord", false, true).as_deref(),
        Some(system::DARK_PAIR)
    );
    assert_eq!(
        system::single("Nord", true, false).as_deref(),
        Some(system::LIGHT_PAIR)
    );
}

#[test]
fn a_pair_lists_both_of_its_sides_and_follow_herdr_lists_none() {
    assert_eq!(
        system::selected_names("light:Nord,dark:Dracula"),
        ["Nord", "Dracula"]
    );
    assert_eq!(system::selected_names("Nord"), ["Nord"]);
    assert_eq!(system::selected_names(FOLLOW_HERDR), Vec::<String>::new());
}

#[test]
fn every_id_the_browser_names_is_a_message() {
    for id in appearance::MESSAGE_IDS {
        assert!(spaceport_i18n::takes(id).is_some(), "{id} names no message");
    }
    assert_eq!(
        spaceport_i18n::takes("settings-theme-card-draft"),
        Some(&["source"][..]),
        "a card fills its library into the draft and saved lines"
    );
}

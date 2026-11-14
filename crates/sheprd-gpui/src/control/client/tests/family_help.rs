//! Help text is a contract with users and scripts: every family `--help`
//! lists has usage of its own, and none of the text changes by accident.
use super::*;
use std::collections::BTreeSet;

const LIST_END: &str = "\nRun `spaceport COMMAND --help` for one command's usage.\n\n";

/// The family names `spaceport --help` lists, in the order it prints them.
fn listed(overview: &str) -> Vec<&str> {
    let (_, rows) = overview.split_once("\n\n").unwrap();
    let (rows, _) = rows.split_once(LIST_END).unwrap();
    rows.lines()
        .map(|row| row.split_whitespace().next().unwrap())
        .collect()
}

/// The text every usage ends with.
fn common(overview: &str) -> &str {
    overview.split_once(LIST_END).unwrap().1
}

/// One family's usage without the shared text.
fn family_usage(name: &str, common: &str) -> String {
    let usage = help::usage(Some(name));
    usage
        .strip_suffix(&format!("\n\n{common}"))
        .unwrap_or_else(|| panic!("`{name}` usage does not end with the shared text"))
        .to_owned()
}

#[test]
fn every_listed_family_has_usage_starting_with_its_own_command() {
    let overview = help::usage(None);
    let names = listed(&overview);
    assert!(!names.is_empty());
    for name in names {
        assert!(
            crate::cli::control::is_family(&name.into()),
            "`{name}` is listed but not parsed"
        );
        let usage = family_usage(name, common(&overview));
        assert!(
            usage.starts_with(&format!("spaceport {name}")),
            "`{name}` usage starts with {:?}",
            usage.lines().next()
        );
    }
}

#[test]
fn family_names_are_unique() {
    let overview = help::usage(None);
    let names = listed(&overview);
    let unique: BTreeSet<_> = names.iter().collect();
    assert_eq!(unique.len(), names.len(), "{names:?}");
}

/// `family_help.txt` holds `spaceport --help` and then each family's usage
/// under a `=== NAME ===` line. Update it only with a deliberate text change.
#[test]
fn help_text_matches_the_recorded_text() {
    let overview = help::usage(None);
    let mut text = overview.clone();
    for name in listed(&overview) {
        text.push_str(&format!("\n\n=== {name} ===\n"));
        text.push_str(&family_usage(name, common(&overview)));
    }
    text.push('\n');
    assert_eq!(text, include_str!("family_help.txt"));
}

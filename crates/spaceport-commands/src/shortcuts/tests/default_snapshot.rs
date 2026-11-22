//! Pins every command's default shortcuts on macOS, Linux and Windows to a
//! checked-in fixture, so a mistyped or dropped default fails here and an
//! intended change shows up as a reviewed diff of that file.

use super::{
    super::{linux, macos, windows},
    Profile,
};
use crate::COMMANDS;
use std::{collections::BTreeSet, env, fs};

/// One line per command and platform, sorted: `name platform: keys`, with
/// the keys in binding order and `-` for none.
const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/default_shortcuts.txt"
);

/// Set to rewrite the fixture from the catalog instead of comparing.
const UPDATE: &str = "UPDATE_SHORTCUT_FIXTURE";

/// The fixture's text for the catalog as it is now.
fn current() -> String {
    let profiles: [(&str, Profile); 3] = [
        ("linux", linux::defaults),
        ("macos", macos::defaults),
        ("windows", windows::defaults),
    ];
    let mut lines: Vec<String> = COMMANDS
        .iter()
        .flat_map(|info| {
            profiles.iter().map(move |(platform, profile)| {
                let keys = match profile(info.command) {
                    [] => "-".to_owned(),
                    keys => keys.join(" "),
                };
                format!("{} {platform}: {keys}", info.name)
            })
        })
        .collect();
    lines.sort();
    lines.iter().map(|line| format!("{line}\n")).collect()
}

#[test]
fn default_shortcuts_match_the_fixture() {
    let current = current();
    if env::var_os(UPDATE).is_some() {
        fs::write(FIXTURE, &current).unwrap_or_else(|error| panic!("{FIXTURE}: {error}"));
        return;
    }
    let pinned = fs::read_to_string(FIXTURE).unwrap_or_else(|error| panic!("{FIXTURE}: {error}"));
    if pinned == current {
        return;
    }
    let pinned: BTreeSet<&str> = pinned.lines().collect();
    let now: BTreeSet<&str> = current.lines().collect();
    let removed = pinned.difference(&now).map(|line| format!("- {line}"));
    let added = now.difference(&pinned).map(|line| format!("+ {line}"));
    let diff: Vec<String> = removed.chain(added).collect();
    panic!(
        "The default shortcuts differ from {FIXTURE}:\n{}\n\n\
         If the change is intended, rewrite the fixture and review its diff:\n  \
         {UPDATE}=1 cargo test -p spaceport-commands --locked default_shortcuts_match_the_fixture",
        diff.join("\n"),
    );
}

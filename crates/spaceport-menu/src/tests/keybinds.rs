//! The Keyboard Shortcuts page's search and its keycaps.

use crate::keybinds::{self, Binding, Row, Section};

/// A section of `rows` under `heading`, from `(description, keys)` pairs.
fn section(heading: &str, rows: &[(&str, &[&str])]) -> Section {
    Section {
        heading: heading.into(),
        rows: rows
            .iter()
            .map(|(description, keys)| Row {
                description: (*description).into(),
                bindings: keys
                    .iter()
                    .map(|keys| Binding {
                        keys: (*keys).into(),
                        note: None,
                    })
                    .collect(),
                edit: None,
            })
            .collect(),
    }
}

#[test]
fn shortcut_search_matches_labels_keys_and_sections() {
    for query in ["", "pane close", "CMD+W", "cmd-w", "workspaces"] {
        assert!(keybinds::matches(
            query,
            "cmd-w",
            "Close Pane",
            "WORKSPACES & PANES"
        ));
    }
    assert!(!keybinds::matches(
        "zoom",
        "cmd-w",
        "Close Pane",
        "WORKSPACES & PANES"
    ));
    assert!(keybinds::matches(
        "cmd shift p",
        "cmd-shift-p",
        "Command Palette",
        "APPLICATION"
    ));
    assert!(!keybinds::matches(
        "cmd+p",
        "cmd-d",
        "Split Right",
        "WORKSPACES & PANES"
    ));
}

#[test]
fn keycaps_split_modifiers_from_the_key() {
    let caps = |keystroke| keybinds::render::keycaps(keystroke).collect::<Vec<_>>();
    assert_eq!(caps("cmd-shift-t"), ["Cmd", "Shift", "T"]);
    assert_eq!(caps("cmd--"), ["Cmd", "-"]);
    assert_eq!(caps("cmd-+"), ["Cmd", "+"]);
    assert_eq!(caps("f5"), ["F5"]);
    assert_eq!(caps("ctrl-b c"), ["Ctrl", "B", "C"]);
    assert_eq!(caps("ctrl-b -"), ["Ctrl", "B", "-"]);
    assert_eq!(caps("ctrl-b shift-tab"), ["Ctrl", "B", "Shift", "Tab"]);
}

#[test]
fn filtering_keeps_matching_rows_and_drops_a_heading_left_empty() {
    let mut sections = vec![
        section(
            "WORKSPACES",
            &[("Close Pane", &["cmd-w"]), ("Split Right", &["cmd-d"])],
        ),
        section("APPLICATION", &[("Command Palette", &["cmd-shift-p"])]),
    ];
    assert_eq!(keybinds::total(&sections), 3);
    assert_eq!(keybinds::filter(&mut sections, "palette"), 1);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].heading, "APPLICATION");
    assert_eq!(
        keybinds::total(&sections),
        1,
        "the count follows what the query kept"
    );
    assert_eq!(keybinds::filter(&mut sections, "no-such-command"), 0);
    assert!(sections.is_empty());
}

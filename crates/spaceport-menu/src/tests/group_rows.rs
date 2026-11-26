//! Where a group menu's rows land among its panel's children, once the
//! headings over them are children too.

use crate::group::render::{Entry, child_index};
use crate::rows::Row;

/// A chat row, a second chat row, a port, a close row and the split row: the
/// two headed runs, then two runs with no heading.
fn entries() -> Vec<Entry> {
    let row = |section, heading: Option<&str>| Entry {
        row: Row::default(),
        section,
        heading: heading.map(str::to_owned),
    };
    vec![
        row(0, Some("New Chat")),
        row(0, Some("New Chat")),
        row(1, Some("Listening")),
        row(2, None),
        row(3, None),
    ]
}

#[test]
fn a_headed_run_puts_a_child_before_its_first_row() {
    let entries = entries();
    assert_eq!(
        (0..entries.len())
            .map(|index| child_index(&entries, index))
            .collect::<Vec<_>>(),
        // Each row plus the headings the panel lays before it.
        [1, 2, 4, 5, 6]
    );
}

#[test]
fn a_run_with_no_heading_adds_no_child() {
    let entries: Vec<Entry> = (0..3)
        .map(|section| Entry {
            row: Row::default(),
            section,
            heading: None,
        })
        .collect();
    assert_eq!(
        (0..entries.len())
            .map(|index| child_index(&entries, index))
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
}

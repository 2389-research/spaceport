use crate::{
    picker::{Folding, Line, Pick},
    sessions::{Entry, Row, Spec, Target},
};
use spaceport_motion::LEAVE;
use std::time::Instant;

fn row(session: &str) -> Row {
    Row::Device {
        id: "ssh:build".to_owned(),
        session: session.to_owned(),
    }
}

fn entry(row: Row) -> Entry {
    Entry::Row {
        row,
        spec: Spec {
            label: "build".to_owned(),
            detail: "running".to_owned(),
            online: true,
            checked: false,
            enabled: true,
        },
    }
}

#[test]
fn only_a_row_is_something_to_choose() {
    let lines = [
        Line::All,
        Line::Device(0),
        Line::Session(entry(row("default"))),
        Line::Session(Entry::Muted {
            selector: "sessions-empty".to_owned(),
            text: "No sessions".to_owned(),
        }),
        Line::Session(Entry::Error("no answer".to_owned())),
        Line::Session(entry(Row::Add(Target::Local))),
        Line::AddDevice,
    ];
    let selectable: Vec<bool> = lines.iter().map(Line::selectable).collect();
    assert_eq!(
        selectable,
        [true, true, true, false, false, true, true],
        "notes explain the list; rows are chosen"
    );
}

#[test]
fn a_line_is_recognised_by_what_it_chooses_not_by_where_it_sits() {
    // The device rows above may arrive or go; the identity of a session row
    // does not move with them.
    assert_eq!(Pick::Session(row("default")), Pick::Session(row("default")));
    assert_ne!(Pick::Session(row("default")), Pick::Session(row("build")));
    assert_ne!(
        Pick::Session(row("default")),
        Pick::Device("ssh:build".to_owned())
    );
    assert_ne!(Pick::All, Pick::AddDevice);
}

#[test]
fn folded_sessions_close_from_where_they_were_and_then_go() {
    let start = Instant::now();
    let folding = Folding {
        device: "ssh:build".to_owned(),
        since: start,
        height: 120.,
        from: 0.6,
    };
    let shown = folding.shown(start).unwrap();
    assert!(
        (shown - 0.6).abs() < 0.001,
        "closes from where it was: {shown}"
    );
    let midway = folding.shown(start + LEAVE / 2).unwrap();
    assert!(midway < 0.6 && midway > 0., "{midway}");
    assert_eq!(folding.shown(start + LEAVE), None, "gone once LEAVE passes");
}

#[test]
fn a_fold_that_began_shut_shows_nothing_while_it_closes() {
    let start = Instant::now();
    let folding = Folding {
        device: "ssh:build".to_owned(),
        since: start,
        height: 0.,
        from: 0.,
    };
    assert_eq!(folding.shown(start), Some(0.));
    assert_eq!(folding.shown(start + LEAVE), None);
}

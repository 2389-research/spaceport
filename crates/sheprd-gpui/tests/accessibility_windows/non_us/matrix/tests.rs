// ABOUTME: Checks the fixed native matrix consumed by Rust and PowerShell.
// ABOUTME: Preserves positional digit targets, native key bounds, and console prerequisites.
use super::*;

#[test]
fn native_cases_have_unique_ids_and_bounded_physical_chords() {
    let mut ids = BTreeSet::new();
    for case in cases() {
        assert!(ids.insert(case.id));
        assert!((1..=4).contains(&case.chords.len()));
        for chord in case.chords {
            assert!((1..=4).contains(&chord.len()));
            assert!(chord.iter().all(|code| (1..=255).contains(code)));
            assert_eq!(chord.iter().collect::<BTreeSet<_>>().len(), chord.len());
        }
    }
}

#[test]
fn physical_digits_cover_each_one_based_tab_position() {
    let cases = cases();
    for number in 1..=9 {
        let matching = cases
            .iter()
            .filter(|case| case.outcome == Outcome::Digit { number })
            .collect::<Vec<_>>();
        let [case] = matching.as_slice() else {
            panic!("digit must have one native case");
        };
        assert_eq!(case.chords, [vec![162, 160, 48 + u32::from(number)]]);
    }
}

#[test]
fn console_attachment_precedes_every_pane_ownership_case() {
    let cases = cases();
    let console = cases
        .iter()
        .filter(|case| matches!(case.outcome, Outcome::PaneKey { .. }))
        .collect::<Vec<_>>();
    assert_eq!(console.first().unwrap().id, "console-attachment");
    for id in [
        "pane-f6",
        "pane-shift-f6",
        "pane-f11",
        "pane-ctrl-pageup",
        "pane-ctrl-pagedown",
    ] {
        assert!(console.iter().skip(1).any(|case| case.id == id));
    }
}

#[test]
fn windows_exceptions_match_the_real_platform_catalog() {
    use sheprd_commands::{COMMANDS, Command};
    let cases = cases();
    for (outcome, command, shortcut, chord) in [
        (
            Outcome::FontReset,
            Command::ResetFontSize,
            "ctrl-0",
            vec![162, 48],
        ),
        (
            Outcome::StatusBar,
            Command::ToggleStatusBar,
            "ctrl-alt-shift-b",
            vec![162, 164, 160, 66],
        ),
        (
            Outcome::Resume,
            Command::ResumePalette,
            "ctrl-alt-shift-r",
            vec![162, 164, 160, 82],
        ),
        (
            Outcome::Left,
            Command::FocusLeft,
            "ctrl-alt-shift-left",
            vec![162, 164, 160, 37],
        ),
        (
            Outcome::Right,
            Command::FocusRight,
            "ctrl-alt-shift-right",
            vec![162, 164, 160, 39],
        ),
        (
            Outcome::Up,
            Command::FocusUp,
            "ctrl-alt-shift-up",
            vec![162, 164, 160, 38],
        ),
        (
            Outcome::Down,
            Command::FocusDown,
            "ctrl-alt-shift-down",
            vec![162, 164, 160, 40],
        ),
    ] {
        let info = COMMANDS
            .iter()
            .find(|info| info.command == command)
            .unwrap();
        assert!(info.default_shortcuts().contains(&shortcut));
        let case = cases.iter().find(|case| case.outcome == outcome).unwrap();
        assert_eq!(case.chords, [chord]);
    }
}

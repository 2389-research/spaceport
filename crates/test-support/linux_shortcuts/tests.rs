// ABOUTME: Checks the real producer and consumer of native fixture notices.
// ABOUTME: Rejects unrelated, malformed, and unrecognized log messages before native input.
use super::*;

const NONCE: &str = "0123456789abcdef0123456789abcdef";

#[test]
fn fixture_notices_round_trip_for_every_compiled_case() {
    for value in [Notice::Mapped, Notice::Complete].into_iter().chain(
        CASES
            .iter()
            .flat_map(|case| [Notice::Ready(case.id), Notice::Passed(case.id)]),
    ) {
        let encoded = notice(NONCE, value);
        assert_eq!(parse_notice(&encoded, NONCE), Some(value));
    }
}

#[test]
fn other_sessions_and_unknown_cases_cannot_authorize_input() {
    for line in [
        "LINUX_KEYS fedcba9876543210 ready new-tab",
        "LINUX_KEYS 0123456789abcdef0123456789abcdef ready unknown",
        "LINUX_KEYS 0123456789abcdef0123456789abcdef ready new-tab extra",
        "terminal output LINUX_KEYS 0123456789abcdef0123456789abcdef ready new-tab",
    ] {
        assert_eq!(parse_notice(line, NONCE), None);
    }
    assert_eq!(
        parse_notice(&notice(NONCE, Notice::Mapped), "bad nonce"),
        None
    );
}

#[test]
fn native_window_preparation_requires_its_compiled_case_and_nonce() {
    for case in CASES {
        let line = notice(NONCE, Notice::FocusOriginal(case.id));
        let expected =
            (case.outcome == Outcome::CycleWindows).then_some(Notice::FocusOriginal(case.id));
        assert_eq!(parse_notice(&line, NONCE), expected);
        assert_eq!(parse_notice(&line, "fedcba9876543210"), None);
    }
}

#[test]
fn case_ids_are_unique_and_evdev_sequences_are_bounded() {
    let mut ids = std::collections::HashSet::new();
    for case in CASES {
        assert!(ids.insert(case.id), "duplicate case {}", case.id);
        assert!((1..=8).contains(&case.chords.len()));
        if let Outcome::TerminalBytes(bytes) = case.outcome {
            assert!((1..=64).contains(&bytes.len()));
        }
        for chord in case.chords {
            assert!((1..=8).contains(&chord.len()));
            assert!(chord.iter().all(|code| (1..=255).contains(code)));
            assert_eq!(
                chord.iter().collect::<std::collections::HashSet<_>>().len(),
                chord.len()
            );
        }
    }
}

#[test]
fn tab_cases_cover_each_shifted_digit_before_the_final_search() {
    for case in CASES {
        if let Outcome::Tab(number) = case.outcome {
            assert!((1..=9).contains(&number), "unsupported tab digit {number}");
        }
    }
    for number in 1..=9 {
        let matching = CASES
            .iter()
            .filter(|case| case.outcome == Outcome::Tab(number))
            .collect::<Vec<_>>();
        let [case] = matching.as_slice() else {
            panic!("expected one physical digit case for tab {number}");
        };
        assert_eq!(case.chords, &[&[29, 42, u32::from(number) + 1][..]]);
    }
    assert_eq!(CASES.last().unwrap().outcome, Outcome::PolishSearch);
}

#[test]
fn tab_targets_follow_workspace_order_and_reject_missing_positions() {
    let tabs = ["tab-14", "tab-3", "tab-77"].map(str::to_owned);
    assert_eq!(tab_at_position(&tabs, 1), Some("tab-14"));
    assert_eq!(tab_at_position(&tabs, 2), Some("tab-3"));
    assert_eq!(tab_at_position(&tabs, 3), Some("tab-77"));
    assert_eq!(tab_at_position(&tabs, 0), None);
    assert_eq!(tab_at_position(&tabs, 4), None);
    assert_eq!(tab_at_position(&[], 1), None);
}

#[test]
fn pane_cases_use_shifted_aliases_and_the_private_geometric_prefix() {
    for (id, chords) in [
        ("next-pane-alias", vec![vec![29, 56, 42, 27]]),
        ("previous-pane-alias", vec![vec![29, 56, 42, 26]]),
        ("history-back-alias", vec![vec![29, 42, 26]]),
        ("history-forward-alias", vec![vec![29, 42, 27]]),
        ("prefix-pane-left", vec![PREFIX.to_vec(), vec![35]]),
        ("prefix-pane-down", vec![PREFIX.to_vec(), vec![36]]),
        ("prefix-pane-up", vec![PREFIX.to_vec(), vec![37]]),
        ("prefix-pane-right", vec![PREFIX.to_vec(), vec![38]]),
    ] {
        let case = CASES.iter().find(|case| case.id == id).unwrap();
        assert_eq!(case.chords, chords);
    }
}

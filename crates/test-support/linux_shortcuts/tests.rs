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
fn case_ids_are_unique_and_evdev_sequences_are_bounded() {
    let mut ids = std::collections::HashSet::new();
    for case in CASES {
        assert!(ids.insert(case.id), "duplicate case {}", case.id);
        assert!((1..=8).contains(&case.chords.len()));
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

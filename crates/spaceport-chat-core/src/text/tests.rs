use super::*;
use spaceport_chat_protocol::Tokens;

fn plain(text: String) -> String {
    text.replace(['\u{2068}', '\u{2069}'], "")
}

/// Each unit starts where the one below it stops, and the parts left over
/// stay with it.
#[test]
fn a_duration_takes_its_largest_whole_unit() {
    assert_eq!(elapsed(0), Elapsed::Milliseconds(0));
    assert_eq!(elapsed(950), Elapsed::Milliseconds(950));
    assert_eq!(elapsed(999), Elapsed::Milliseconds(999));
    assert_eq!(elapsed(1_000), Elapsed::Seconds(1));
    assert_eq!(elapsed(12_000), Elapsed::Seconds(12));
    assert_eq!(elapsed(59_999), Elapsed::Seconds(59));
    assert_eq!(
        elapsed(60_000),
        Elapsed::Minutes {
            minutes: 1,
            seconds: 0
        }
    );
    assert_eq!(
        elapsed(184_000),
        Elapsed::Minutes {
            minutes: 3,
            seconds: 4
        }
    );
    assert_eq!(
        elapsed(3_599_999),
        Elapsed::Minutes {
            minutes: 59,
            seconds: 59
        }
    );
    assert_eq!(
        elapsed(3_600_000),
        Elapsed::Hours {
            hours: 1,
            minutes: 0
        }
    );
    assert_eq!(
        elapsed(3_720_000),
        Elapsed::Hours {
            hours: 1,
            minutes: 2
        }
    );
}

/// A duration reads in the unit it measured, and English reads as the
/// protocol's own `display` writes it.
#[test]
fn english_matches_the_protocol() {
    for ms in [950, 12_000, 184_000, 3_720_000] {
        assert_eq!(plain(duration(ms)), display::duration(ms));
    }
    let ends = [
        TurnEnd::Done,
        TurnEnd::Cancelled,
        TurnEnd::Limit,
        TurnEnd::Refused,
        TurnEnd::Failed,
        TurnEnd::Other,
    ];
    let tokens = [
        None,
        Some(Tokens {
            input: 3_200,
            output: 450,
            ..Tokens::default()
        }),
        Some(Tokens {
            input: 18_600,
            output: 1_200_000,
            thought: 90,
            ..Tokens::default()
        }),
    ];
    for end in ends {
        for tokens in &tokens {
            let summary = TurnSummary {
                duration_ms: 12_000,
                ended_ms: None,
                end,
                tokens: *tokens,
            };
            assert_eq!(plain(turn(&summary)), summary.describe());
        }
    }
}

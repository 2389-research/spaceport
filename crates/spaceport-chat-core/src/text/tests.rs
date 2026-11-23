use super::*;

/// Each unit starts where the one below it stops, and the parts left over
/// stay with it.
#[test]
fn a_duration_takes_its_largest_whole_unit() {
    assert_eq!(duration(0), Elapsed::Milliseconds(0));
    assert_eq!(duration(950), Elapsed::Milliseconds(950));
    assert_eq!(duration(999), Elapsed::Milliseconds(999));
    assert_eq!(duration(1_000), Elapsed::Seconds(1));
    assert_eq!(duration(12_000), Elapsed::Seconds(12));
    assert_eq!(duration(59_999), Elapsed::Seconds(59));
    assert_eq!(
        duration(60_000),
        Elapsed::Minutes {
            minutes: 1,
            seconds: 0
        }
    );
    assert_eq!(
        duration(184_000),
        Elapsed::Minutes {
            minutes: 3,
            seconds: 4
        }
    );
    assert_eq!(
        duration(3_599_999),
        Elapsed::Minutes {
            minutes: 59,
            seconds: 59
        }
    );
    assert_eq!(
        duration(3_600_000),
        Elapsed::Hours {
            hours: 1,
            minutes: 0
        }
    );
    assert_eq!(
        duration(3_720_000),
        Elapsed::Hours {
            hours: 1,
            minutes: 2
        }
    );
}

/// A turn keeps how it ended and the tokens reported, and measures its own
/// duration.
#[test]
fn a_turn_keeps_what_its_summary_says() {
    let tokens = Tokens {
        input: 3_200,
        output: 450,
        ..Tokens::default()
    };
    let summary = TurnSummary {
        duration_ms: 184_000,
        ended_ms: None,
        end: TurnEnd::Cancelled,
        tokens: Some(tokens),
    };
    assert_eq!(
        turn(&summary),
        Measured {
            elapsed: Elapsed::Minutes {
                minutes: 3,
                seconds: 4
            },
            end: TurnEnd::Cancelled,
            tokens: Some(tokens),
        }
    );
    let unmeasured = TurnSummary {
        tokens: None,
        ..summary
    };
    assert_eq!(
        turn(&unmeasured),
        Measured {
            elapsed: Elapsed::Minutes {
                minutes: 3,
                seconds: 4
            },
            end: TurnEnd::Cancelled,
            tokens: None,
        }
    );
}

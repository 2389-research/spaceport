// ABOUTME: Checks native provider fixture diagnostics against actual transcript outcomes.
// ABOUTME: Keeps recovered errors distinct from a final failed provider turn.
#![allow(clippy::unwrap_used)]
use super::completed_provider_turn;
use sheprd_chat_protocol::{
    Entry, EntryId, EntryKind, NoticeLevel, Transcript, TurnEnd, TurnSummary,
};

fn transcript_with(end: TurnEnd, error: Option<&str>) -> Transcript {
    let mut transcript = Transcript::default();
    if let Some(error) = error {
        transcript.entries.push(Entry {
            id: EntryId(1),
            kind: EntryKind::Notice {
                answers: Vec::new(),
                details: None,
                level: NoticeLevel::Error,
                text: error.into(),
            },
        });
    }
    transcript.entries.push(Entry {
        id: EntryId(2),
        kind: EntryKind::Turn(TurnSummary {
            duration_ms: 1,
            ended_ms: Some(2),
            end,
            tokens: None,
        }),
    });
    transcript
}

#[test]
fn failed_turn_reports_actual_provider_error_before_missing_evidence() {
    let diagnostic = "429: CONCURRENT_REQUEST_LIMIT_EXCEEDED";
    let transcript = transcript_with(TurnEnd::Failed, Some(diagnostic));
    let error = completed_provider_turn(&transcript).unwrap_err();
    assert!(error.to_string().contains(diagnostic));
}

#[test]
fn failed_turn_without_error_still_reports_the_authoritative_failure() {
    let transcript = transcript_with(TurnEnd::Failed, None);
    let error = completed_provider_turn(&transcript).unwrap_err();
    assert_eq!(
        error.to_string(),
        "actual provider turn failed without a diagnostic"
    );
}

#[test]
fn successful_final_turn_allows_an_earlier_recovered_error() {
    let transcript = transcript_with(TurnEnd::Done, Some("temporary admission error"));
    assert!(completed_provider_turn(&transcript).unwrap());
}

#[test]
fn partial_answer_without_final_summary_remains_pending() {
    let mut transcript = Transcript::default();
    transcript.entries.push(Entry {
        id: EntryId(1),
        kind: EntryKind::Agent {
            text: "NATIVE_RESULT=partial [source](spaceport-source:partial)".into(),
        },
    });
    assert!(!completed_provider_turn(&transcript).unwrap());
}

#[test]
fn partial_answer_with_unsuccessful_final_summary_cannot_pass() {
    for end in [
        TurnEnd::Cancelled,
        TurnEnd::Limit,
        TurnEnd::Refused,
        TurnEnd::Other,
    ] {
        let mut transcript = transcript_with(end, None);
        transcript.entries.insert(
            0,
            Entry {
                id: EntryId(0),
                kind: EntryKind::Agent {
                    text: "NATIVE_RESULT=partial [source](spaceport-source:partial)".into(),
                },
            },
        );
        assert!(
            !matches!(completed_provider_turn(&transcript), Ok(true)),
            "{end:?}"
        );
    }
}

//! The words a pane hands its prepared bodies are the words they write:
//! differently labelled ones reach each of the nine places those labels go.

use super::*;
use crate::chat::{
    labels::Labels,
    prepared::{Body, Part, Prepared, tool_header},
};
use spaceport_chat_core::prepared::{answers_text, tool_text};
use spaceport_chat_protocol::{MAX_QUESTION_ANSWERS, QuestionAnswer, ToolContent};

/// Labels nothing else would write, so a place that kept reading the language
/// itself stands out.
fn marked() -> Labels {
    Labels {
        answer_cut: "cut-note".into(),
        diff_after: "after-label".into(),
        diff_before: "before-label".into(),
        diff_new_file: "new-file-label".into(),
        question_answer: "answer-label".into(),
        question_number: |number| format!("question-{number}"),
        question_truncated: "truncated-label".into(),
        terminal: |id| format!("terminal-{id}"),
        tool_ask_user: "ask-user-label".into(),
        turn: |_| "turn-label".into(),
    }
}

#[test]
fn a_tool_header_and_its_output_carry_the_labels_given() {
    let labels = marked();
    assert_eq!(tool_header("request_user_input", &labels), "ask-user-label");
    assert_eq!(
        tool_header("request_user_input_async", &labels),
        "ask-user-label"
    );
    // Every other title is its own label.
    assert_eq!(tool_header("read", &labels), "read");
    assert_eq!(
        tool_text(
            &ToolContent::Diff {
                path: "a.txt".into(),
                old: Some("one".into()),
                new: "two".into(),
            },
            &labels,
        ),
        Some("a.txt\nbefore-label\none\nafter-label\ntwo".into())
    );
    assert_eq!(
        tool_text(
            &ToolContent::Diff {
                path: "b.txt".into(),
                old: None,
                new: "two".into(),
            },
            &labels,
        ),
        Some("b.txt\nnew-file-label\ntwo".into())
    );
    assert_eq!(
        tool_text(&ToolContent::Terminal { id: "t7".into() }, &labels),
        Some("terminal-t7".into())
    );
}

#[test]
fn answered_questions_carry_the_labels_given() {
    let labels = marked();
    let answers = [QuestionAnswer {
        question: "q".into(),
        answer: "a".into(),
    }];
    let text = answers_text(&answers, &labels);
    assert!(text.contains("**question-1**"), "{text}");
    assert!(text.contains("**answer-label**"), "{text}");

    let too_many: Vec<QuestionAnswer> = std::iter::repeat_with(|| QuestionAnswer {
        question: "q".into(),
        answer: "a".into(),
    })
    .take(MAX_QUESTION_ANSWERS + 1)
    .collect();
    assert!(
        answers_text(&too_many, &labels).ends_with("truncated-label"),
        "bounded answers end with the note the pane gave"
    );
}

#[test]
fn a_cut_answer_ends_with_the_label_given() {
    let labels = marked();
    let cut = format!("{}\n{}", "x".repeat(5_000), "a b c\n".repeat(2_100));
    let mut transcript = Transcript::new("Claude");
    transcript.entries = vec![agent(1, &cut)];
    let mut prepared = Prepared::default();
    prepared.update(&transcript, &[(EntryId(1), 1)], &labels);
    let body = prepared.body(EntryId(1)).expect("a prepared body");
    let Body::Message(parts) = &*body else {
        panic!("message");
    };
    let Some(Part::Text(lines)) = parts.last() else {
        panic!("plain lines");
    };
    assert_eq!(
        lines.last().map(|line| line.text.as_str()),
        Some("cut-note")
    );
}

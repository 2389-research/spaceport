// ABOUTME: Pins that an entry's plain text is drawn from the selection document's own segment.
// ABOUTME: Covers every entry kind, text past the display limit, folding, and a language switch.
use super::{
    overlay::{attach_with, open, poll},
    *,
};
use crate::{
    SheprdWindow,
    chat::{
        prepared::MESSAGE,
        selection::{Cell, Region},
    },
};
use gpui::{Entity, TestAppContext, VisualTestContext};
use sheprd_chat_protocol::{
    NoticeLevel, QuestionAnswer, Tool, ToolContent, ToolKind, ToolStatus, TurnEnd, TurnSummary,
};
use sheprd_client::ConnectTarget;
use sheprd_i18n::{Locale, Pseudo};

const PANE: &str = "w1:p1";

struct Shown<'a> {
    cx: &'a mut VisualTestContext,
    program: Program,
    _stream: Peer,
    view: Entity<SheprdWindow>,
}

/// The fixture window with `entries` in its chat, drawn.
fn shown(entries: Vec<Entry>, cx: &mut TestAppContext) -> Shown<'_> {
    crate::i18n::show(&Locale::FALLBACK);
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let stream = attach_with(&view, &program, snapshot(cursor(1, 0), entries), cx);
    Shown {
        cx,
        program,
        _stream: stream,
        view,
    }
}

impl Shown<'_> {
    fn draw(&mut self) {
        poll(&self.view, &self.program, self.cx);
    }

    /// Folds or opens `entry` as a click on its header does, and draws.
    fn toggle(&mut self, entry: u64) {
        self.view.update(self.cx, |view, _| {
            view.chats
                .pane_mut(PANE)
                .unwrap()
                .toggle_expanded(EntryId(entry));
        });
        self.draw();
    }

    /// The document's text at a place, when it holds any.
    fn segment(&mut self, entry: u64, region: Region, cell: Cell) -> Option<String> {
        self.view.read_with(self.cx, |view, _| {
            let document = view.chats.pane(PANE).unwrap().prepared.text();
            let key = document.key(EntryId(entry), region, cell)?;
            Some(document.segment(key)?.text().to_owned())
        })
    }

    /// The text drawn selectable at a place: `None` when the place has no
    /// segment, or the view drew its text plain.
    fn selectable(&mut self, entry: u64, region: Region, cell: Cell) -> Option<String> {
        self.view.read_with(self.cx, |view, _| {
            let pane = view.chats.pane(PANE).unwrap();
            let key = pane.prepared.text().key(EntryId(entry), region, cell)?;
            pane.text_geometry.drawn(key)
        })
    }

    /// Asserts the place draws `expected`, selectable, from its segment.
    #[track_caller]
    fn assert_text(&mut self, entry: u64, region: Region, cell: Cell, expected: &str) {
        assert_eq!(
            self.segment(entry, region, cell).as_deref(),
            Some(expected),
            "segment of entry {entry} at {region:?} {cell:?}"
        );
        assert_eq!(
            self.selectable(entry, region, cell).as_deref(),
            Some(expected),
            "drawn text of entry {entry} at {region:?} {cell:?}"
        );
    }

    /// Asserts the place has no segment and so draws nothing selectable.
    #[track_caller]
    fn assert_absent(&mut self, entry: u64, region: Region, cell: Cell) {
        assert_eq!(self.segment(entry, region, cell), None);
        assert_eq!(self.selectable(entry, region, cell), None);
    }
}

fn entry(id: u64, kind: EntryKind) -> Entry {
    Entry {
        id: EntryId(id),
        kind,
    }
}

fn notice(id: u64, text: &str, details: Option<&str>) -> Entry {
    entry(
        id,
        EntryKind::Notice {
            answers: Vec::new(),
            details: details.map(str::to_owned),
            level: NoticeLevel::Info,
            text: text.into(),
        },
    )
}

fn thought(id: u64, text: &str) -> Entry {
    entry(id, EntryKind::Thought { text: text.into() })
}

fn tool(id: u64, title: &str, locations: &[&str], content: Vec<ToolContent>) -> Entry {
    entry(
        id,
        EntryKind::Tool(Tool {
            call_id: format!("call-{id}"),
            content,
            kind: ToolKind::Edit,
            locations: locations.iter().map(|&location| location.into()).collect(),
            parent: None,
            started_ms: None,
            status: ToolStatus::Completed,
            title: title.into(),
        }),
    )
}

/// Lines of plain words past the display limit of one entry's text.
fn over_long() -> String {
    let text = "plain words on a line\n".repeat(MESSAGE.body / 22 + 1_000);
    assert!(text.len() > MESSAGE.body);
    text
}

/// Asserts the place draws exactly its segment, which stops at the limit.
#[track_caller]
fn assert_clipped_and_selectable(shown: &mut Shown<'_>, entry: u64, region: Region, cell: Cell) {
    let segment = shown
        .segment(entry, region, cell)
        .expect("the document holds the text");
    assert!(!segment.is_empty());
    assert!(segment.len() <= MESSAGE.body, "{}", segment.len());
    let drawn = shown.selectable(entry, region, cell);
    assert!(
        drawn.as_deref() == Some(segment.as_str()),
        "drawn selectable: {:?} bytes; segment: {} bytes",
        drawn.map(|drawn| drawn.len()),
        segment.len()
    );
}

/// Asserts an over-long message is drawn as bounded lines, and that each
/// line in view is drawn selectable from its own segment. The list follows
/// the message's end, so its last lines are the ones in view.
#[track_caller]
fn assert_bounded_lines_selectable(shown: &mut Shown<'_>, entry: u64) {
    let body = Region::Body(0);
    let rows = (0..)
        .take_while(|row| shown.segment(entry, body, Cell::row(*row)).is_some())
        .count();
    // Every kept line, and the note that says the rest is not shown.
    assert_eq!(rows, MESSAGE.lines + 1);
    assert_eq!(shown.segment(entry, body, Cell::row(rows)), None);
    let drawn: Vec<_> = (0..rows)
        .filter_map(|row| Some((row, shown.selectable(entry, body, Cell::row(row))?)))
        .collect();
    assert!(drawn.iter().any(|(row, _)| *row == rows - 1), "{drawn:?}");
    for (row, drawn) in drawn {
        assert_eq!(shown.segment(entry, body, Cell::row(row)), Some(drawn));
    }
}

#[gpui::test]
fn an_over_long_agent_message_is_selectable(cx: &mut TestAppContext) {
    let mut shown = shown(vec![agent(1, &over_long())], cx);
    assert_bounded_lines_selectable(&mut shown, 1);
}

#[gpui::test]
fn an_over_long_user_message_is_selectable(cx: &mut TestAppContext) {
    let user = entry(
        1,
        EntryKind::User {
            text: over_long(),
            files: 0,
        },
    );
    let mut shown = shown(vec![user], cx);
    assert_bounded_lines_selectable(&mut shown, 1);
}

#[gpui::test]
fn an_over_long_expanded_thought_is_selectable(cx: &mut TestAppContext) {
    let mut shown = shown(vec![thought(1, &over_long())], cx);
    shown.assert_text(1, Region::Body(0), Cell::default(), "plain words on a line");
    shown.toggle(1);
    assert_clipped_and_selectable(&mut shown, 1, Region::Body(0), Cell::default());
}

#[gpui::test]
fn an_over_long_notice_is_selectable(cx: &mut TestAppContext) {
    let mut shown = shown(vec![notice(1, &over_long(), None)], cx);
    assert_clipped_and_selectable(&mut shown, 1, Region::Body(0), Cell::default());
}

#[gpui::test]
fn an_over_long_notice_detail_is_selectable(cx: &mut TestAppContext) {
    let mut shown = shown(vec![notice(1, "Imported", Some(&over_long()))], cx);
    shown.toggle(1);
    assert_clipped_and_selectable(&mut shown, 1, Region::Details, Cell::default());
}

#[gpui::test]
fn short_messages_thoughts_and_turns_draw_their_segments(cx: &mut TestAppContext) {
    let turn = TurnSummary {
        duration_ms: 65_000,
        ended_ms: None,
        end: TurnEnd::Done,
        tokens: None,
    };
    let user = entry(
        1,
        EntryKind::User {
            text: "Fix it".into(),
            files: 0,
        },
    );
    let mut shown = shown(
        vec![
            user,
            agent(2, "Done."),
            thought(3, "\nFirst idea\nSecond idea"),
            entry(4, EntryKind::Turn(turn.clone())),
        ],
        cx,
    );
    let body = Region::Body(0);
    shown.assert_text(1, body, Cell::default(), "Fix it");
    shown.assert_text(2, body, Cell::default(), "Done.");
    shown.assert_text(3, body, Cell::default(), "First idea");
    shown.assert_text(4, body, Cell::default(), &text::turn(&turn));
}

#[gpui::test]
fn folding_a_thought_changes_its_drawn_text_and_segment_together(cx: &mut TestAppContext) {
    let text = "First idea\nSecond idea";
    let mut shown = shown(vec![thought(1, text)], cx);
    shown.assert_text(1, Region::Body(0), Cell::default(), "First idea");
    shown.toggle(1);
    shown.assert_text(1, Region::Body(0), Cell::default(), text);
    shown.toggle(1);
    shown.assert_text(1, Region::Body(0), Cell::default(), "First idea");
}

#[gpui::test]
fn notices_draw_their_segments_folded_and_open(cx: &mut TestAppContext) {
    let mut shown = shown(
        vec![
            notice(1, "Context compacted", None),
            notice(2, "Imported session", Some("<record>source</record>")),
        ],
        cx,
    );
    shown.assert_text(1, Region::Body(0), Cell::default(), "Context compacted");
    shown.assert_text(2, Region::Body(0), Cell::default(), "Imported session");
    shown.assert_absent(2, Region::Details, Cell::default());
    shown.toggle(2);
    shown.assert_text(2, Region::Body(0), Cell::default(), "Imported session");
    shown.assert_text(
        2,
        Region::Details,
        Cell::default(),
        "<record>source</record>",
    );
    shown.toggle(2);
    shown.assert_absent(2, Region::Details, Cell::default());
}

#[gpui::test]
fn answered_questions_draw_their_label_and_keep_their_record_header(cx: &mut TestAppContext) {
    let answered = entry(
        1,
        EntryKind::Notice {
            answers: vec![QuestionAnswer {
                question: "Which file?".into(),
                answer: "main.rs".into(),
            }],
            details: Some("<answers>source</answers>".into()),
            level: NoticeLevel::Info,
            text: "Questions answered".into(),
        },
    );
    let mut shown = shown(vec![answered], cx);
    shown.assert_text(1, Region::Label, Cell::default(), "Questions answered");
    // The record's header is a label of the view's own. The answers hold
    // `Body(0)`, so the header must never draw their first paragraph.
    let header = shown.cx.debug_bounds("chat-notice-1").unwrap();
    let answers = shown.cx.debug_bounds("chat-answers-1").unwrap();
    assert!(header.size.height < answers.size.height / 2.);
    shown.toggle(1);
    shown.assert_text(
        1,
        Region::Details,
        Cell::default(),
        "<answers>source</answers>",
    );
}

fn edit_call() -> Entry {
    tool(
        1,
        "Edit two files",
        &["src/main.rs:3", "src/lib.rs"],
        vec![
            ToolContent::Text {
                text: "first output".into(),
            },
            ToolContent::Image {
                // Never decoded here: only its place among the blocks matters.
                image: sheprd_chat_protocol::Image::new("image/png", "AAAA".into()).unwrap(),
            },
            ToolContent::Diff {
                new: "new line".into(),
                old: Some("old line".into()),
                path: "src/main.rs".into(),
            },
        ],
    )
}

#[gpui::test]
fn a_tool_call_draws_its_segments_folded_and_open(cx: &mut TestAppContext) {
    let mut shown = shown(vec![edit_call()], cx);
    shown.assert_text(1, Region::Label, Cell::default(), "Edit two files");
    shown.assert_absent(1, Region::ToolTitle, Cell::default());
    shown.assert_absent(1, Region::ToolContent, Cell::block(0));
    assert!(shown.cx.debug_bounds("chat-tool-content-1-0").is_none());

    shown.toggle(1);
    shown.assert_text(1, Region::Label, Cell::default(), "Edit two files");
    shown.assert_text(1, Region::ToolTitle, Cell::default(), "Edit two files");
    shown.assert_text(1, Region::ToolLocations, Cell::row(0), "src/main.rs:3");
    shown.assert_text(1, Region::ToolLocations, Cell::row(1), "src/lib.rs");
    shown.assert_text(1, Region::ToolContent, Cell::block(0), "first output");
    // A picture is not text: its block has no segment, and the blocks after
    // it keep the index of their content item.
    shown.assert_absent(1, Region::ToolContent, Cell::block(1));
    shown.assert_text(
        1,
        Region::ToolContent,
        Cell::block(2),
        "src/main.rs\nBefore:\nold line\nAfter:\nnew line",
    );
    assert!(shown.cx.debug_bounds("chat-tool-details-1").is_some());
    assert!(shown.cx.debug_bounds("chat-tool-content-1-0").is_some());
    assert!(shown.cx.debug_bounds("chat-tool-content-1-1").is_none());
    assert!(shown.cx.debug_bounds("chat-tool-content-1-2").is_some());

    shown.toggle(1);
    shown.assert_absent(1, Region::ToolContent, Cell::block(2));
    assert!(shown.cx.debug_bounds("chat-tool-details-1").is_none());
}

#[gpui::test]
fn a_language_switch_draws_the_new_languages_segments(cx: &mut TestAppContext) {
    let mut shown = shown(vec![edit_call()], cx);
    shown.toggle(1);
    let diff = (Region::ToolContent, Cell::block(2));
    let english = shown.segment(1, diff.0, diff.1).unwrap();
    assert!(english.contains("Before:"), "{english}");

    crate::i18n::show(&Pseudo::Accented.locale());
    shown.draw();
    let accented = shown.segment(1, diff.0, diff.1).unwrap();
    assert_ne!(accented, english);
    assert!(!accented.contains("Before:"), "{accented}");
    assert_eq!(
        shown.selectable(1, diff.0, diff.1).as_deref(),
        Some(accented.as_str())
    );
    // Text the agent wrote is the same in every language.
    shown.assert_text(1, Region::ToolContent, Cell::block(0), "first output");
    crate::i18n::show(&Locale::FALLBACK);
}

#[gpui::test]
fn text_the_document_does_not_hold_is_still_drawn(cx: &mut TestAppContext) {
    let user = entry(
        1,
        EntryKind::User {
            text: "Fix it".into(),
            files: 0,
        },
    );
    let mut shown = shown(vec![user], cx);
    // The layout whose rows can be found by entry.
    shown.view.update(shown.cx, |view, _| {
        view.config.chat.layout = crate::config::ChatLayoutMode::Bubbles;
    });
    shown.draw();
    shown.assert_text(1, Region::Body(0), Cell::default(), "Fix it");
    let prepared = shown.cx.debug_bounds("chat-bubble-1").unwrap();
    // No update path leaves the document behind the transcript; were one to,
    // the row shows its text plain rather than nothing.
    shown.view.update(shown.cx, |view, _| {
        view.chats.pane_mut(PANE).unwrap().prepared.clear();
    });
    shown.cx.update(|window, cx| {
        crate::sidebar::layout_tests::full_draw(window, cx).clear(cx);
    });
    shown.assert_absent(1, Region::Body(0), Cell::default());
    let plain = shown.cx.debug_bounds("chat-bubble-1").unwrap();
    assert_eq!(plain.size, prepared.size);
}

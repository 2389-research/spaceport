//! What a message shows for things loaded only when asked: pages from HTML
//! fences and HTML files, pictures on the web, and the captions of pictures
//! herdr-acp adds after the message.

use super::{
    overlay::{attach_with, open, poll},
    visual::message,
    *,
};
use crate::SheprdWindow;
use crate::chat::visual::VisualId;
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};
use sheprd_chat_protocol::{Tool, ToolKind, ToolStatus};
use sheprd_client::ConnectTarget;

#[test]
fn a_closed_html_fence_becomes_a_page_and_an_open_one_stays_code() {
    let (_, shape) = message("Here it is:\n```html\n<h1>Hi</h1>\n```\nEnjoy.");
    assert_eq!(shape, ["text:Here it is:", "page", "text:Enjoy."]);
    let (_, shape) = message("Here it is:\n```html\n<h1>Hi</h1>\n");
    assert_eq!(shape, ["text:Here it is:"]);
    let long = format!(
        "```html\n{}\n```",
        "x".repeat(crate::chat::prepared::MAX_PAGE + 1)
    );
    let (_, shape) = message(&long);
    assert!(!shape.contains(&"page".to_owned()), "too long to preview");
}

#[test]
fn picture_lines_load_on_request_or_name_what_follows() {
    let (prepared, shape) = message(
        "A chart:\n![chart](out/chart.png)\n![logo](https://example.com/logo.png)\n\
         See ![inline](a.png) here.\n![data](data:image/png;base64,AAAA)\n\
         ![odd](ftp://example.com/a.png)",
    );
    assert_eq!(
        shape,
        [
            "text:A chart:",
            "picture:chart",
            "remote:https://example.com/logo.png",
            "text:See ![inline](a.png) here. ![data](data:image/png;base64,AAAA) ![odd](ftp://example.com/a.png)",
        ]
    );
    let remote = VisualId::of("https://example.com/logo.png");
    assert!(!prepared.visuals.wants(remote), "nothing loads unasked");
}

/// The pane the fixture's agent runs in, which the chat stands in for.
fn chat_pane(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, _| {
        view.live.snapshot.as_ref().unwrap().agents[0]
            .pane_id
            .clone()
    })
}

fn said(text: &str) -> ServerMessage {
    let mut transcript = Transcript::new("Claude");
    transcript.status = Status::Idle;
    transcript.entries = vec![agent(1, text)];
    ServerMessage::Snapshot {
        cursor: cursor(1, 0),
        transcript: Box::new(transcript),
    }
}

#[gpui::test]
fn a_web_picture_loads_only_once_asked(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    // Nothing listens there, so asking fails at once without the network.
    let url = "http://127.0.0.1:9/chart.png";
    let _stream = attach_with(&view, &program, said(&format!("![chart]({url})")), cx);
    let id = VisualId::of(url);
    let pane = chat_pane(&view, cx);
    let wants = |cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| {
            view.chats.pane(&pane).unwrap().prepared.visuals.wants(id)
        })
    };
    assert!(!wants(cx));
    let load = cx.debug_bounds("chat-load-picture").unwrap();
    cx.simulate_click(load.center(), Modifiers::default());
    assert!(wants(cx));
    poll(&view, &program, cx);
    assert!(cx.debug_bounds("chat-load-picture").is_none());
}

#[gpui::test]
fn requested_html_opens_a_copied_file_without_creating_an_embedded_page(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, said("```html\n<p>preview</p>\n```"), cx);
    assert!(cx.opened_url().is_none());
    let show = cx.debug_bounds("chat-show-page").unwrap();
    cx.simulate_click(show.center(), Modifiers::default());
    wait_until(|| {
        cx.run_until_parked();
        cx.opened_url().is_some()
    });
    let opened = url::Url::parse(&cx.opened_url().unwrap())
        .unwrap()
        .to_file_path()
        .unwrap();
    assert_eq!(std::fs::read_to_string(opened).unwrap(), "<p>preview</p>\n");
    poll(&view, &program, cx);
    assert!(cx.debug_bounds("chat-page").is_none());
}

#[gpui::test]
fn an_html_file_a_tool_wrote_offers_a_preview(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut transcript = Transcript::new("Claude");
    transcript.status = Status::Idle;
    let page = if cfg!(windows) {
        r"C:\work\index.html"
    } else {
        "/work/index.html"
    };
    transcript.entries = vec![Entry {
        id: EntryId(1),
        kind: EntryKind::Tool(Tool {
            call_id: "write".into(),
            content: Vec::new(),
            kind: ToolKind::Edit,
            locations: vec![page.into()],
            parent: None,
            started_ms: None,
            status: ToolStatus::Completed,
            title: "Write index.html".into(),
        }),
    }];
    let snapshot = ServerMessage::Snapshot {
        cursor: cursor(1, 0),
        transcript: Box::new(transcript),
    };
    let _stream = attach_with(&view, &program, snapshot, cx);
    assert!(cx.debug_bounds("chat-show-page").is_some());
}

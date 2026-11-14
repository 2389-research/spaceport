// ABOUTME: Exercises a real Ask source click through the window event boundary.
// ABOUTME: Uses broker-issued fixture evidence and reports readiness instead of a false closed window.
use super::*;
use gpui::{Modifiers, px, size};
use sheprd_chat_protocol::{Entry, EntryId, EntryKind, Tool, ToolContent, Transcript};

#[gpui::test]
fn source_click_runs_after_the_current_window_update(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| crate::ask::fixture(None, cx));
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        view.live.status = ConnectionStatus::Connected;
        view.config.sidebar_style.activity_bar = true;
    });
    let target = cx.cx.update(|cx| {
        let Response::Observation {
            reply: reply @ Reply::Sessions { .. },
        } = serve(
            call(Operation::Sessions {
                offset: 0,
                limit: 1,
                status: None,
            }),
            cx,
        )
        else {
            panic!("real observation reply");
        };
        let Reply::Sessions { sessions, .. } = &reply else {
            unreachable!();
        };
        let target = sessions[0].source.target.clone();
        let evidence = serde_json::to_string(&reply).unwrap();
        let mut transcript = Transcript::new("Pi");
        transcript.entries = vec![
            Entry {
                id: EntryId(1),
                kind: EntryKind::User {
                    text: "Show this observed source".into(),
                    files: 0,
                },
            },
            Entry {
                id: EntryId(2),
                kind: EntryKind::Tool(Tool {
                    call_id: "fixture-list".into(),
                    content: vec![ToolContent::Text { text: evidence }],
                    kind: Default::default(),
                    locations: vec![],
                    parent: None,
                    started_ms: None,
                    status: Default::default(),
                    title: "spaceport_list_sessions".into(),
                }),
            },
            Entry {
                id: EntryId(3),
                kind: EntryKind::Agent {
                    text: format!("[Source](spaceport-source:{})", target.0),
                },
            },
        ];
        crate::ask::fixture(Some(transcript), cx);
        target
    });
    cx.simulate_resize(size(px(1000.), px(720.)));
    cx.update(|window, cx| {
        window.activate_window();
        view.update(cx, |view, cx| {
            view.show_activity_panel(crate::sidebar::ActivityPanel::Ask, window, cx)
        });
    });
    for _ in 0..2 {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
    let Some(bounds) = cx.debug_bounds("ask-source-3-0") else {
        panic!("actual answer source chip");
    };
    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();
    cx.cx.update(|cx| {
        let status = crate::ask::status(cx).unwrap();
        assert_eq!(
            status.error.as_deref(),
            Some(crate::i18n::fl!("ask-source-busy").as_str()),
            "source {} is valid; this fixture has no ready terminal surface",
            target.0
        );
    });
}

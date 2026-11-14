// ABOUTME: Exercises pointer selection through virtualized Ask transcript rows.
// ABOUTME: Uses explicit frame times and native text geometry instead of sleeps.
use super::*;
use gpui::{Modifiers, MouseButton, VisualTestContext, point, size};
use std::time::{Duration, Instant};

fn reading(cx: &mut TestAppContext) -> (gpui::Entity<View>, &mut VisualTestContext) {
    let (view, cx) =
        cx.add_window_view(|_, cx| View::new(Default::default(), Default::default(), cx));
    view.update(cx, |view, cx| {
        let mut transcript = Transcript::new("Pi");
        transcript.status = Status::Idle;
        transcript.entries = (1..=80)
            .map(|id| Entry {
                id: EntryId(id),
                kind: EntryKind::Agent {
                    text: format!("Row {id:02} contains selectable café text."),
                },
            })
            .collect();
        view.owner.update(cx, |owner, _| {
            owner.configured = true;
            owner.set_transcript(transcript);
        });
        view.prepare(cx);
        view.list.scroll_to(ListOffset::default());
        view.glide.user_scrolled();
    });
    cx.simulate_resize(size(px(680.), px(460.)));
    cx.run_until_parked();
    cx.update(|window, cx| {
        window.activate_window();
        window.draw(cx).clear(cx);
    });
    view.update(cx, |view, cx| {
        view.list.scroll_to(ListOffset::default());
        view.glide.user_scrolled();
        cx.notify();
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    (view, cx)
}

#[gpui::test]
fn stationary_drag_selects_rows_beyond_the_original_viewport(cx: &mut TestAppContext) {
    let (view, cx) = reading(cx);
    let Some(line) = cx.debug_bounds("ask-line-1-0") else {
        panic!("a mounted transcript line");
    };
    let from = point(line.left() + px(2.), line.center().y);
    let viewport = view.read_with(cx, |view, _| view.list.viewport_bounds());
    let Some(page) = cx.debug_bounds("ask-spaceport") else {
        panic!("Ask page");
    };
    let to = point(
        from.x,
        (viewport.bottom() + px(40.)).min(page.bottom() - px(2.)),
    );
    assert!(
        to.y > viewport.bottom(),
        "drag must leave the transcript viewport"
    );
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::default());
    let initial = view.read_with(cx, |view, _| {
        view.selection
            .text(view.prepared.text())
            .unwrap_or_default()
    });
    assert!(
        !initial.is_empty(),
        "mouse drag must select actual mounted text"
    );
    let now = Instant::now();
    for frame in 0..70 {
        view.update(cx, |view, _| {
            view.tick_selection_drag(now + Duration::from_millis(frame * 50));
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }
    let selected = view.read_with(cx, |view, _| {
        view.selection
            .text(view.prepared.text())
            .unwrap_or_default()
    });
    assert!(
        selected.len() > initial.len(),
        "stationary pointer must extend selection past mounted rows"
    );
    assert!(
        selected.contains("Row 12"),
        "selection must reach a row outside the initial viewport: {selected:?}"
    );
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    let stopped = view.read_with(cx, |view, _| view.list.logical_scroll_top());
    view.update(cx, |view, _| {
        view.tick_selection_drag(now + Duration::from_secs(10));
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.list.logical_scroll_top()),
        stopped,
        "release must stop scrolling"
    );
}

#[gpui::test]
fn drag_timer_stops_after_focus_loss_and_window_close(cx: &mut TestAppContext) {
    let (view, cx) = reading(cx);
    let Some(line) = cx.debug_bounds("ask-line-1-0") else {
        panic!("a mounted transcript line");
    };
    let from = point(line.left() + px(2.), line.center().y);
    let viewport = view.read_with(cx, |view, _| view.list.viewport_bounds());
    let Some(page) = cx.debug_bounds("ask-spaceport") else {
        panic!("Ask page");
    };
    let to = point(
        from.x,
        (viewport.bottom() + px(40.)).min(page.bottom() - px(2.)),
    );
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::default());
    let initial = view.read_with(cx, |view, _| view.list.logical_scroll_top());
    assert!(view.read_with(cx, |view, _| view.drag_task.is_some()));
    cx.executor().advance_clock(Duration::from_millis(20));
    cx.run_until_parked();
    let scrolled = view.read_with(cx, |view, _| view.list.logical_scroll_top());
    assert_ne!(
        scrolled, initial,
        "the owned timer must scroll a stationary held pointer"
    );
    cx.deactivate_window();
    cx.executor().advance_clock(Duration::from_millis(20));
    cx.run_until_parked();
    assert!(view.read_with(cx, |view, _| view.drag_task.is_none()));
    assert!(!view.read_with(cx, |view, _| view.selection.is_selecting()));
    let stopped = view.read_with(cx, |view, _| view.list.logical_scroll_top());
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    assert_eq!(
        view.read_with(cx, |view, _| view.list.logical_scroll_top()),
        stopped
    );
    cx.update(|window, _| window.activate_window());
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::default());
    assert!(view.read_with(cx, |view, _| view.drag_task.is_some()));
    cx.update(|window, _| window.remove_window());
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    assert!(view.read_with(cx, |view, _| view.drag_task.is_none()));
    assert!(!view.read_with(cx, |view, _| view.selection.is_selecting()));
}

#[gpui::test]
fn replacement_history_cancels_drag_and_rejects_old_text_geometry(cx: &mut TestAppContext) {
    use crate::chat::selection::{Cell, Region};
    let (view, cx) = reading(cx);
    let Some(line) = cx.debug_bounds("ask-line-1-0") else {
        panic!("mounted first line");
    };
    let from = point(line.left() + px(2.), line.center().y);
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    assert!(view.read_with(cx, |view, _| view.drag_task.is_some()));
    view.update(cx, |view, cx| {
        let old = view
            .prepared
            .text()
            .key(EntryId(1), Region::Body(0), Cell::row(0));
        let transcript = Transcript {
            status: Status::Idle,
            entries: vec![Entry {
                id: EntryId(1),
                kind: EntryKind::Agent {
                    text: "Replacement history".into(),
                },
            }],
            ..Transcript::new("Pi")
        };
        view.owner
            .update(cx, |owner, _| owner.set_transcript(transcript));
        view.prepare(cx);
        assert!(view.drag_task.is_none());
        assert!(!view.selection.is_selecting());
        assert!(old.is_some_and(|key| view.prepared.text().segment(key).is_none()));
    });
    let stopped = view.read_with(cx, |view, _| view.list.logical_scroll_top());
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    assert_eq!(
        view.read_with(cx, |view, _| view.list.logical_scroll_top()),
        stopped
    );
}

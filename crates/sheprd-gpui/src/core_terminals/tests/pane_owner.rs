// ABOUTME: Checks that a Core tab's pane tree has one owner, the app's tab store.
// ABOUTME: Covers two windows on one workspace and the store's readers in one window.

use crate::{
    SheprdWindow,
    browser::{CoreGroupKey, Location, Store, TabId},
    core_terminals::{
        self, Error, Start,
        panes::{Axis, Panes},
    },
    sidebar::layout_tests::fixture_core_window,
};
use gpui::{TestAppContext, WindowHandle};
use sheprd_backend::TerminalId;
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

fn id(name: &str) -> TerminalId {
    TerminalId::new(name).unwrap()
}

fn no_core(_: Start) -> Result<sheprd_core_client::CoreBackend, Error> {
    Err(Error::NotInstalled)
}

/// A window showing the Core group, joined to no real Core.
fn window_on(key: &CoreGroupKey, cx: &mut TestAppContext) -> WindowHandle<SheprdWindow> {
    let handle = cx.add_window(fixture_core_window);
    handle
        .update(cx, |view, _, cx| {
            view.core_terminals.hub.set_join_for_test(no_core);
            assert!(view.show_core_group(key, cx));
        })
        .unwrap();
    handle
}

/// One Core group with one tab of a single pane, `t1`.
fn group_with_tab(cx: &mut TestAppContext) -> (CoreGroupKey, TabId) {
    let group = Uuid::from_u128(71);
    let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
    let tab = cx.update(|cx| {
        core_terminals::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"Core group"}
                ]}),
                version: 3,
            },
        );
        Store::update(cx, |store| {
            store.open_core(
                &key,
                Some(Location::CoreTerminal {
                    panes: Panes::single(id("t1")),
                    observe: false,
                }),
            )
        })
        .unwrap()
    });
    (key, tab)
}

fn saved_terminals(tab: TabId, cx: &mut TestAppContext) -> Vec<String> {
    cx.update(
        |cx| match &cx.global::<Store>().get(tab).unwrap().location {
            Some(Location::CoreTerminal { panes, .. }) => panes
                .terminals()
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            other => panic!("a Core tab, not {other:?}"),
        },
    )
}

#[gpui::test]
fn a_split_in_one_window_survives_an_edit_in_another(cx: &mut TestAppContext) {
    let (key, tab) = group_with_tab(cx);
    let first = window_on(&key, cx);
    let second = window_on(&key, cx);
    // Both windows show the workspace before either edits it.
    for window in [first, second] {
        window
            .update(cx, |view, window, cx| view.poll_core_terminals(window, cx))
            .unwrap();
    }
    first
        .update(cx, |view, window, cx| {
            view.place_split(tab, &id("t1"), Axis::Row, id("t2"), window, cx);
            view.poll_core_terminals(window, cx);
        })
        .unwrap();
    second
        .update(cx, |view, window, cx| {
            view.poll_core_terminals(window, cx);
            view.place_split(tab, &id("t1"), Axis::Column, id("t3"), window, cx);
            view.poll_core_terminals(window, cx);
        })
        .unwrap();
    let mut saved = saved_terminals(tab, cx);
    saved.sort_unstable();
    assert_eq!(saved, ["t1", "t2", "t3"]);
}

#[gpui::test]
fn the_stores_readers_see_a_split_at_once(cx: &mut TestAppContext) {
    let (key, tab) = group_with_tab(cx);
    let window = window_on(&key, cx);
    window
        .update(cx, |view, window, cx| {
            view.poll_core_terminals(window, cx);
            // No tick runs between the split and the read below.
            view.place_split(tab, &id("t1"), Axis::Row, id("t2"), window, cx);
        })
        .unwrap();
    // The sidebar rows and `tab list` both read this tree.
    assert_eq!(saved_terminals(tab, cx), ["t1", "t2"]);
}

fn core_panes(tab: TabId, cx: &mut TestAppContext) -> Panes {
    cx.update(
        |cx| match &cx.global::<Store>().get(tab).unwrap().location {
            Some(Location::CoreTerminal { panes, .. }) => panes.clone(),
            other => panic!("a Core tab, not {other:?}"),
        },
    )
}

#[gpui::test]
fn a_dragged_split_reaches_the_store_once_the_drag_ends(cx: &mut TestAppContext) {
    use crate::core_terminals::{panes::Share, split::SplitDrag};
    let (key, tab) = group_with_tab(cx);
    let window = window_on(&key, cx);
    window
        .update(cx, |view, window, cx| {
            view.poll_core_terminals(window, cx);
            view.place_split(tab, &id("t1"), Axis::Row, id("t2"), window, cx);
        })
        .unwrap();
    let before = core_panes(tab, cx);
    let drag = SplitDrag::for_test(tab, Vec::new(), Axis::Row);
    window
        .update(cx, |view, _, _| {
            assert!(view.set_core_share(&drag, Share::at(0.25)));
            assert!(!view.set_core_share(&drag, Share::at(0.25)));
            assert!(view.set_core_share(&drag, Share::at(0.3)));
        })
        .unwrap();
    // Each step of the drag stays in the window: nothing is saved.
    assert_eq!(core_panes(tab, cx), before);
    // The tick after the drop hands the tab where the split was left.
    window
        .update(cx, |view, window, cx| view.poll_core_terminals(window, cx))
        .unwrap();
    let mut dropped = before;
    assert!(dropped.set_share(&[], Share::at(0.3)));
    assert_eq!(core_panes(tab, cx), dropped);
}

#[gpui::test]
fn a_pane_closed_in_another_window_leaves_the_pane_in_use_and_the_zoom(cx: &mut TestAppContext) {
    use crate::controls::Command;
    use crate::core_terminals::{CoreTerminalView, worker::Link};
    use gpui::AppContext as _;
    let (key, tab) = group_with_tab(cx);
    let first = window_on(&key, cx);
    let second = window_on(&key, cx);
    first
        .update(cx, |view, window, cx| {
            view.poll_core_terminals(window, cx);
            view.place_split(tab, &id("t1"), Axis::Row, id("t2"), window, cx);
        })
        .unwrap();
    // The second window uses and zooms t2, which holds its keyboard.
    let (kept, closed) = second
        .update(cx, |view, window, cx| {
            view.poll_core_terminals(window, cx);
            let [kept, closed] = ["t1", "t2"].map(|name| {
                let pane = cx.new(|cx| {
                    CoreTerminalView::new(
                        id(name),
                        Link::for_test(),
                        view.core_look(),
                        view.painter.clone(),
                        false,
                        cx,
                    )
                });
                view.core_terminals
                    .views
                    .insert((tab, id(name)), pane.clone());
                pane
            });
            view.show_browser_tab(tab, window, cx);
            window.focus(&closed.read(cx).focus_handle().clone(), cx);
            (kept, closed)
        })
        .unwrap();
    second
        .update(cx, |view, window, cx| {
            // The tick follows the keyboard to t2.
            view.poll_core_terminals(window, cx);
            assert!(view.core_pane_command(Command::Zoom, window, cx));
            let shown = &view.core_terminals.tabs[&tab];
            assert_eq!((&shown.active, &shown.zoomed), (&id("t2"), &Some(id("t2"))));
        })
        .unwrap();
    first
        .update(cx, |view, window, cx| {
            assert!(view.remove_core_pane(tab, &id("t2"), window, cx));
        })
        .unwrap();
    assert_eq!(saved_terminals(tab, cx), ["t1"]);
    second
        .update(cx, |view, window, cx| {
            view.poll_core_terminals(window, cx);
            let shown = &view.core_terminals.tabs[&tab];
            assert_eq!((&shown.active, &shown.zoomed), (&id("t1"), &None));
            assert!(!view.core_terminals.views.contains_key(&(tab, id("t2"))));
            assert!(kept.read(cx).focus_handle().is_focused(window));
        })
        .unwrap();
    drop(closed);
}

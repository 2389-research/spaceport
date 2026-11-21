// ABOUTME: Exercises selected Core projections through the actual profile group consumer.
// ABOUTME: Uses emulator output and headless canvas layout without claiming native presentation.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{
    browser::{CoreGroupKey, Location, Shown, Store},
    core_terminals::{
        CoreTerminalView, Error, Start,
        panes::{Axis, Panes},
        worker::Link,
    },
    sidebar::layout_tests::{fixture_core_window, fixture_window},
    working_profile::report::{CoreProjection, Group, MARKER},
};
use gpui::{AppContext, Entity, IntoElement, TestAppContext, VisualTestContext, point, px, size};
use sheprd_backend::{Exit, FrameData, Size, SurfaceUpdate, TerminalId};
use sheprd_core_client::wire::Doc;
use sheprd_terminal::{Damage, Emulator};
use std::sync::Arc;
use uuid::Uuid;

mod paint_cache;
mod paint_work;

fn no_core(_: Start) -> Result<sheprd_core_client::CoreBackend, Error> {
    Err(Error::NotInstalled)
}

fn frame(text: &str) -> FrameData {
    let mut emulator = Emulator::new(Size::new(80, 24).unwrap(), 0);
    emulator.feed(text.as_bytes());
    let Damage::Full(frame) = emulator.take_damage() else {
        panic!("a new emulator must produce its complete screen");
    };
    frame
}

struct Selected {
    owner: Entity<SheprdWindow>,
    key: CoreGroupKey,
    tab: TabId,
    terminal: TerminalId,
    view: Entity<CoreTerminalView>,
}

fn selected(cx: &mut TestAppContext, observe: bool) -> (Selected, &mut VisualTestContext) {
    let (owner, cx) = cx.add_window_view(fixture_core_window);
    let (key, tab, terminal, view) = owner.update_in(cx, |owner, window, cx| {
        let group = Uuid::from_u128(71);
        let key = CoreGroupKey::new("profile-hub", None, "spaceport", group).unwrap();
        crate::core_terminals::set_document_for_test_as(
            cx,
            "profile-hub",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"Profile Core group"}
                ]}),
                version: 3,
            },
        );
        let terminal = TerminalId::new("selected-core-terminal").unwrap();
        let tab = Store::update(cx, |store| {
            store.open_core(
                &key,
                Some(Location::CoreTerminal {
                    panes: Panes::single(terminal.clone()),
                    observe,
                }),
            )
        })
        .unwrap();
        owner.core_terminals.hub.set_join_for_test(no_core);
        assert!(owner.show_core_group(&key, cx));
        let view = cx.new(|cx| {
            let mut view = CoreTerminalView::new(
                terminal.clone(),
                Link::for_test(),
                owner.core_look(),
                owner.painter.clone(),
                observe,
                cx,
            );
            view.screen.apply(SurfaceUpdate::Frame(frame(MARKER)));
            view
        });
        owner
            .core_terminals
            .views
            .insert((tab, terminal.clone()), view.clone());
        owner.show_browser_tab(tab, window, cx);
        (key, tab, terminal, view)
    });
    (
        Selected {
            owner,
            key,
            tab,
            terminal,
            view,
        },
        cx,
    )
}

fn observed(selected: &Selected, cx: &mut VisualTestContext) -> Core {
    selected.owner.read_with(cx, |owner, cx| {
        assert!(
            owner
                .drawn_group_slots()
                .iter()
                .any(|slot| { owner.group_shown(slot.id, cx) == Shown::Page(selected.tab) })
        );
        let groups = owner.working_profile_groups(cx);
        assert!(!groups.is_empty(), "the selected group must be observed");
        assert!(groups.iter().all(|group| !group.shown_terminal));
        groups
            .into_iter()
            .find_map(|group| group.core)
            .unwrap_or_else(|| {
                panic!("the selected single-pane Core view needs a projected observation")
            })
    })
}

#[gpui::test]
fn selected_core_projection_is_not_a_paint_receipt(cx: &mut TestAppContext) {
    let (selected, cx) = selected(cx, false);
    let projection = observed(&selected, cx);
    assert_eq!(projection.tab, u64::from(selected.tab));
    assert_eq!(projection.terminal, selected.terminal.as_str());
    assert_eq!(projection.core_id, None);
    assert!(!projection.connected);
    assert!(!projection.observe);
    assert!(!projection.ended);
    assert!(!projection.failed);
    assert_eq!(projection.last_layout, None, "the canvas has never drawn");
    let frame = projection.projected.as_ref().unwrap();
    assert_eq!((frame.columns, frame.rows), (80, 24));
    assert!(frame.nonblank_cells >= MARKER.len());
    assert!(frame.projected_marker);
    let value = serde_json::to_value(&projection).unwrap();
    assert_eq!(value["projected"]["projected_marker"], true);
    assert!(value.get("presented").is_none());
    assert!(value["projected"].get("marker_visible").is_none());
    assert!(!value.to_string().contains(MARKER), "do not dump cell text");
    let decoded: Core = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), value);
}

#[gpui::test]
fn selected_tab_does_not_take_another_tabs_view_of_the_same_terminal(cx: &mut TestAppContext) {
    let (selected, cx) = selected(cx, false);
    selected.owner.update(cx, |owner, cx| {
        let hidden_tab = Store::update(cx, |store| {
            store.open_core(
                &selected.key,
                Some(Location::CoreTerminal {
                    panes: Panes::single(selected.terminal.clone()),
                    observe: true,
                }),
            )
        })
        .unwrap();
        assert_ne!(hidden_tab, selected.tab, "the fixture needs distinct tabs");
        let hidden = cx.new(|cx| {
            let mut view = CoreTerminalView::new(
                selected.terminal.clone(),
                Link::for_test(),
                owner.core_look(),
                owner.painter.clone(),
                true,
                cx,
            );
            view.screen
                .apply(SurfaceUpdate::Frame(frame("hidden content")));
            view
        });
        owner
            .core_terminals
            .views
            .insert((hidden_tab, selected.terminal.clone()), hidden);
    });
    let projection = observed(&selected, cx);
    assert_eq!(projection.tab, u64::from(selected.tab));
    assert!(!projection.observe);
    assert!(projection.projected.unwrap().projected_marker);
}

#[gpui::test]
fn projected_core_reports_actual_view_role_and_terminal_end(cx: &mut TestAppContext) {
    let (selected, cx) = selected(cx, true);
    selected.view.update(cx, |view, _| {
        view.screen
            .apply(SurfaceUpdate::Ended(Exit { code: Some(0) }));
    });
    let projection = observed(&selected, cx);
    assert!(projection.observe);
    assert!(projection.ended);
    assert!(!projection.failed);
    assert!(projection.projected.unwrap().projected_marker);
}

#[gpui::test]
fn projected_core_reports_mailbox_failure_without_private_error_text(cx: &mut TestAppContext) {
    let (selected, cx) = selected(cx, false);
    selected.view.update(cx, |view, cx| {
        view.link.failure("private fixture error".into());
        assert!(view.poll(cx));
    });
    let projection = observed(&selected, cx);
    assert!(projection.failed);
    assert!(!projection.connected);
    assert!(projection.projected.is_none());
    assert!(
        !serde_json::to_string(&projection)
            .unwrap()
            .contains("private fixture error")
    );
}

#[cfg(unix)]
#[gpui::test]
fn projected_core_reports_the_observed_hub_identity_and_connection_loss(cx: &mut TestAppContext) {
    let (selected, cx) = selected(cx, false);
    let (backend, peer) = crate::core_terminals::tests::scripted_core::connect("profile-hub");
    selected.owner.update(cx, |owner, _| {
        owner
            .core_terminals
            .hub
            .set_backend_for_test(backend.clone());
    });
    let projection = observed(&selected, cx);
    assert_eq!(projection.core_id.as_deref(), Some("profile-hub"));
    assert!(projection.connected);
    crate::core_terminals::tests::scripted_core::lose(&backend, peer);
    selected.owner.read_with(cx, |owner, cx| {
        assert_eq!(owner.selected_core_group(), Some(&selected.key));
        assert!(!owner.core_group_available(&selected.key, cx));
        let slots = owner.drawn_group_slots();
        assert!(!slots.is_empty(), "the fixture retains its group layout");
        assert!(
            slots
                .iter()
                .all(|slot| owner.group_shown(slot.id, cx) == Shown::Empty)
        );
        let groups = owner.working_profile_groups(cx);
        assert!(!groups.is_empty(), "observe the hidden group's exclusion");
        assert!(groups.iter().all(|group| group.core.is_none()));
        // A lost Core hides its group before the next hub or view poll.
        let projection = owner.profile_core_tab(selected.tab, cx).unwrap();
        assert_eq!(projection.core_id.as_deref(), Some("profile-hub"));
        assert!(!projection.connected, "do not wait for a later view poll");
    });
}

#[gpui::test]
fn projected_core_keeps_last_layout_separate_from_later_cells(cx: &mut TestAppContext) {
    let (selected, cx) = selected(cx, false);
    selected.view.update(cx, |view, _| {
        view.screen
            .apply(SurfaceUpdate::Frame(frame("earlier content")));
    });
    cx.draw(point(px(20.), px(30.)), size(px(800.), px(600.)), |_, _| {
        selected.view.clone().into_any_element()
    });
    let bounds = selected
        .view
        .read_with(cx, |view, _| view.bounds().unwrap());
    selected.view.update(cx, |view, _| {
        view.screen.apply(SurfaceUpdate::Frame(frame(MARKER)));
    });
    let source = selected
        .view
        .read_with(cx, |view, _| view.screen.frame.clone().unwrap());
    let owners = Arc::strong_count(&source);
    let projection = observed(&selected, cx);
    assert_eq!(
        projection.last_layout,
        Some([
            f32::from(bounds.left()),
            f32::from(bounds.top()),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        ])
    );
    assert!(projection.projected.unwrap().projected_marker);
    selected.view.read_with(cx, |view, _| {
        assert!(Arc::ptr_eq(view.screen.frame.as_ref().unwrap(), &source));
        assert!(
            view.sleeping.is_none(),
            "observation must not fill the veil cache"
        );
    });
    assert_eq!(Arc::strong_count(&source), owners, "borrow projected cells");
}

#[gpui::test]
fn profile_single_pane_limit_does_not_change_the_product_split(cx: &mut TestAppContext) {
    let (selected, cx) = selected(cx, false);
    cx.update(|_, cx| {
        Store::update(cx, |store| {
            store
                .edit_panes(selected.tab, |panes| {
                    panes.split(
                        &selected.terminal,
                        Axis::Row,
                        TerminalId::new("second-product-pane").unwrap(),
                    )
                })
                .unwrap()
                .unwrap();
        });
    });
    selected.owner.read_with(cx, |owner, cx| {
        assert!(
            owner
                .working_profile_groups(cx)
                .iter()
                .all(|group| group.core.is_none())
        );
        let Some(Location::CoreTerminal { panes, .. }) =
            &cx.global::<Store>().get(selected.tab).unwrap().location
        else {
            panic!("the product tab must remain a Core terminal");
        };
        assert!(panes.contains(&selected.terminal));
        assert!(panes.contains(&TerminalId::new("second-product-pane").unwrap()));
    });
}

#[gpui::test]
fn ordinary_terminal_group_serialization_omits_core(cx: &mut TestAppContext) {
    let (owner, cx) = cx.add_window_view(fixture_window);
    owner.read_with(cx, |owner, cx| {
        let groups = owner.working_profile_groups(cx);
        assert!(!groups.is_empty(), "the TUI control needs a drawn group");
        for group in groups {
            assert!(group.core.is_none());
            let value = serde_json::to_value(&group).unwrap();
            assert!(value.get("core").is_none());
            let decoded: Group = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), value);
        }
    });
}

#[test]
fn projected_marker_schema_round_trips_without_a_presented_claim() {
    let projected = CoreProjection {
        columns: 80,
        rows: 24,
        nonblank_cells: MARKER.len(),
        projected_marker: true,
    };
    let value = serde_json::to_value(projected).unwrap();
    assert_eq!(value["projected_marker"], true);
    assert!(value.get("marker_visible").is_none());
    let decoded: CoreProjection = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), value);
}

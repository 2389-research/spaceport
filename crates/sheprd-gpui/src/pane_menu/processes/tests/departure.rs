// ABOUTME: Supplies process paint fixtures without starting a monitor or signaling processes.
// ABOUTME: Keeps departure geometry separate from worker and identity execution tests.
use super::*;
use crate::pane_menu::tests::departure::{draw, preview};

pub(in crate::pane_menu) fn fixture() -> PaneProcesses {
    PaneProcesses {
        watch: None,
        listing: Some(Listing {
            rows: vec![row(10, 0), row(20, 1), row(30, 2)],
            hidden: 0,
            cpu: 4.5,
            memory: 9 << 20,
        }),
        chosen: [Identity {
            pid: 20,
            started: 1,
        }]
        .into(),
        cursor: Some(1),
        scroll: ScrollHandle::new(),
        error: None,
        report: None,
        ending: false,
    }
}

#[gpui::test]
fn dismissing_the_process_list_releases_its_watch_before_departure_finishes(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        crate::pane_menu::departure_fixture(Page::PaneProcesses, window, cx)
    });
    let (watch, updates, commands) = Watch::fake();
    view.update(cx, |view, _| {
        view.menu
            .pane
            .as_mut()
            .unwrap()
            .processes
            .as_mut()
            .unwrap()
            .watch = Some(watch);
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("pane-process-1").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.dismiss_menu(window, cx);
            assert!(view.menu.pane.is_none());
        });
    });
    assert!(matches!(
        updates.try_send(listing(&[(10, 0), (20, 1)])),
        Err(std::sync::mpsc::TrySendError::Disconnected(_))
    ));
    assert!(matches!(
        commands.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Disconnected)
    ));
}

#[gpui::test]
fn frozen_process_choices_and_consent_cannot_act_on_replacement_selection(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx, Page::PaneProcesses);
    let source = preview.read_with(cx, |preview, _| preview.source.clone());
    let list = source.update(cx, |source, cx| source.pane_menu_paint(cx).unwrap());
    let confirmation = source.update(cx, |source, cx| {
        source.menu.page = Some(Page::KillProcesses);
        source.pane_menu_paint(cx).unwrap()
    });
    let (watch, _updates, commands) = Watch::fake();
    source.update(cx, |source, _| {
        let processes = source
            .menu
            .pane
            .as_mut()
            .unwrap()
            .processes
            .as_mut()
            .unwrap();
        processes.chosen = [Identity {
            pid: 30,
            started: 1,
        }]
        .into();
        processes.watch = Some(watch);
    });
    for (paint, page, selectors) in [
        (
            list,
            Page::PaneProcesses,
            &["pane-process-1", "pane-processes-end"][..],
        ),
        (
            confirmation,
            Page::KillProcesses,
            &["pane-processes-cancel", "pane-processes-confirm"][..],
        ),
    ] {
        source.update(cx, |source, _| source.menu.page = Some(page));
        preview.update(cx, |preview, cx| {
            preview.paint = Some(paint);
            cx.notify();
        });
        draw(cx);
        assert!(cx.debug_bounds("menu-leaving-shield").is_none());
        for selector in selectors {
            let bounds = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(bounds.center(), Modifiers::default());
            source.read_with(cx, |source, _| {
                assert_eq!(source.menu.page, Some(page));
                let processes = source
                    .menu
                    .pane
                    .as_ref()
                    .unwrap()
                    .processes
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    processes.chosen,
                    [Identity {
                        pid: 30,
                        started: 1
                    }]
                    .into()
                );
                assert!(!processes.ending);
                assert!(processes.error.is_none());
            });
            assert!(matches!(
                commands.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Empty)
            ));
        }
    }
}

#[gpui::test]
fn frozen_process_rows_keep_the_actual_scroll_after_monitor_owner_reset(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx, Page::PaneProcesses);
    preview.update(cx, |preview, cx| {
        preview.source.update(cx, |source, _| {
            let processes = source
                .menu
                .pane
                .as_mut()
                .unwrap()
                .processes
                .as_mut()
                .unwrap();
            processes.listing.as_mut().unwrap().rows = (10..90)
                .map(|pid| row(pid, usize::from(pid != 10)))
                .collect();
        });
        cx.notify();
    });
    draw(cx);
    let viewport = cx.debug_bounds("pane-processes-list").unwrap();
    let before = cx.debug_bounds("pane-process-0").unwrap();
    cx.simulate_event(ScrollEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-120.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx.debug_bounds("pane-process-0").unwrap();
    assert!(scrolled.top() < before.top());
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(preview.source.update(cx, |source, cx| {
                let paint = source.pane_menu_paint(cx).unwrap();
                source.dismiss_menu(window, cx);
                assert!(source.menu.pane.is_none());
                paint
            }));
            cx.notify();
        })
    });
    draw(cx);
    assert_eq!(cx.debug_bounds("pane-process-0"), Some(scrolled));
    assert_eq!(cx.debug_bounds("pane-processes-list"), Some(viewport));
}

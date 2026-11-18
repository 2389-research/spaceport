// ABOUTME: Exercises measured picker clipping and passive session controls without a shield.
// ABOUTME: Uses prepared fixtures and invalid replacement drafts without discovery or mutation.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{fonts::StyledFont, menu::devices::DevicesPaint};
use gpui::{Entity, IntoElement, Render, TestAppContext, Window, div, point, prelude::*, px};
use sheprd_client::{ConnectTarget, LocalSession, SessionState};

mod catalog;
mod controls;

struct Preview {
    source: Entity<SheprdWindow>,
    paint: Option<DevicesPaint>,
    modal: bool,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let font = self.source.read(cx).config.ui.clone();
        div()
            .w(px(360.))
            .h(px(650.))
            .text_font(&font)
            .text_size(px(font.size))
            .line_height(px(font.line_height()))
            .child(self.source.update(cx, |view, cx| match &self.paint {
                Some(paint) if self.modal => view.render_devices_session_paint(paint, cx).unwrap(),
                Some(paint) => view.render_devices_paint(paint, cx),
                None => view.render_devices(cx),
            }))
    }
}

fn fixture(window: &mut Window, cx: &mut Context<SheprdWindow>) -> SheprdWindow {
    let mut view = fixture_window(window, cx);
    view.endpoints[0].connection.target = ConnectTarget::Socket("/synthetic/no-daemon.sock".into());
    view.sessions.entries = (0..6)
        .map(|index| LocalSession {
            name: format!("fixture-session-{index}"),
            state: SessionState::Running,
            socket: format!("/synthetic/fixture-{index}.sock").into(),
        })
        .collect();
    view.menu.page = Some(Page::Devices);
    view.menu.anchor = point(px(0.), px(650.));
    view.menu.unfolded_device = Some(view.endpoints[0].id.clone());
    view
}

#[gpui::test]
fn partial_opening_and_folding_keep_the_measured_clip_after_reset(cx: &mut TestAppContext) {
    for folding in [false, true] {
        let (root, cx) = cx.add_window_view(|window, cx| Preview {
            source: cx.new(|cx| fixture(window, cx)),
            paint: None,
            modal: false,
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let source = root.read_with(cx, |root, _| root.source.clone());
        let full = source.read_with(cx, |view, _| view.menu.unfolded_height.get());
        assert!(full > px(0.));
        source.update(cx, |view, _| {
            let later = crate::motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.live = true;
            view.menu.unfolded_since = Some(later);
            view.menu.unfold_from = 0.5;
            if folding {
                view.fold_device(0);
                view.menu.folding.as_mut().unwrap().since = later;
            }
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let before = source.read_with(cx, |view, _| {
            let mut bounds = view.menu.devices_scroll.bounds_for_item(3).unwrap();
            bounds.origin += view.menu.devices_scroll.offset();
            bounds
        });
        // Layout snaps the explicit half-height to device pixels before capture.
        assert!(before.size.height > px(0.) && before.size.height < full);
        cx.update(|window, cx| {
            root.update(cx, |root, cx| {
                root.paint = root.source.update(cx, |view, cx| {
                    let paint = view.devices_paint(cx).unwrap();
                    view.dismiss_menu(window, cx);
                    view.sessions.entries.clear();
                    view.menu.folding = None;
                    Some(paint)
                });
            });
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            cx.debug_bounds("devices-block-3"),
            Some(before),
            "folding={folding}"
        );
    }
}

struct Dropped(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[gpui::test]
fn departure_does_not_take_or_cancel_an_accepted_session_task(cx: &mut TestAppContext) {
    let (root, cx) = cx.add_window_view(|window, cx| Preview {
        source: cx.new(|cx| fixture(window, cx)),
        paint: None,
        modal: false,
    });
    let dropped = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let source = root.read_with(cx, |root, _| root.source.clone());
    source.update(cx, |view, cx| {
        let hold = Dropped(dropped.clone());
        view.sessions.mutation = Some(cx.spawn(async move |_, _| {
            let _hold = hold;
            std::future::pending::<()>().await;
        }));
    });
    cx.run_until_parked();
    assert_eq!(dropped.load(std::sync::atomic::Ordering::SeqCst), 0);
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        root.update(cx, |root, cx| {
            root.paint = root.source.update(cx, |view, cx| {
                let paint = view.devices_paint(cx);
                view.dismiss_menu(window, cx);
                assert!(view.sessions.mutation.is_some());
                paint
            });
            root.paint = None;
        });
    });
    cx.run_until_parked();
    assert_eq!(dropped.load(std::sync::atomic::Ordering::SeqCst), 0);
    source.update(cx, |view, _| view.sessions.mutation = None);
    cx.run_until_parked();
    assert_eq!(dropped.load(std::sync::atomic::Ordering::SeqCst), 1);
}

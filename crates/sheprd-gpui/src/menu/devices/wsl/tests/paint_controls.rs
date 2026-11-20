// ABOUTME: Exercises passive setup controls without a parent input shield.
// ABOUTME: Keeps bad submissions local and checks disposable input/task release.
use super::*;
use crate::fonts::StyledFont;
use gpui::{Entity, IntoElement, Render, Window, div, prelude::*, px};

struct Frozen {
    source: Entity<SheprdWindow>,
    paint: super::super::super::DeviceSetupPaint,
}

impl Render for Frozen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let font = self.source.read(cx).config.ui.clone();
        div()
            .w(px(640.))
            .h(px(620.))
            .flex()
            .flex_col()
            .text_font(&font)
            .text_size(px(font.size))
            .line_height(px(font.line_height()))
            .child(self.source.update(cx, |view, cx| {
                view.render_device_setup_paint(&self.paint, false, cx)
            }))
    }
}

fn frozen(page: Page, cx: &mut TestAppContext) -> (Entity<Frozen>, &mut gpui::VisualTestContext) {
    let (root, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| departure::fixture(page, window, cx));
        let paint = source.update(cx, |view, cx| {
            let paint = view.device_setup_paint(cx).unwrap();
            view.dismiss_menu(window, cx);
            departure::install(view, page, window, cx);
            if let Some(setup) = &mut view.menu.wsl_setup {
                setup.selected = None;
            }
            paint
        });
        Frozen { source, paint }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    (root, cx)
}

#[gpui::test]
fn frozen_core_submit_and_close_cannot_affect_replacement_form(cx: &mut TestAppContext) {
    let (root, cx) = frozen(Page::AddDevice, cx);
    for selector in ["device-setup-submit", "device-setup-close"] {
        let bounds = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(bounds.center(), Modifiers::default());
    }
    root.read_with(cx, |root, cx| {
        let view = root.source.read(cx);
        assert_eq!(view.menu.page, Some(Page::AddDevice));
        assert!(view.menu.device_setup.as_ref().unwrap().error.is_none());
    });
}

#[gpui::test]
fn frozen_wsl_submit_and_close_cannot_affect_replacement_form(cx: &mut TestAppContext) {
    let (root, cx) = frozen(Page::AddWsl, cx);
    for selector in ["wsl-setup-submit", "wsl-setup-close"] {
        let bounds = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(bounds.center(), Modifiers::default());
    }
    root.read_with(cx, |root, cx| {
        let view = root.source.read(cx);
        assert_eq!(view.menu.page, Some(Page::AddWsl));
        let setup = view.menu.wsl_setup.as_ref().unwrap();
        assert!(setup.error.is_none());
        assert!(setup.selected.is_none());
        assert_eq!(setup.step, Step::Form);
    });
}

#[gpui::test]
fn frozen_distribution_rows_cannot_change_replacement_selection(cx: &mut TestAppContext) {
    let (root, cx) = frozen(Page::AddWsl, cx);
    let bounds = cx.debug_bounds("wsl-distro-0").unwrap();
    cx.simulate_click(bounds.center(), Modifiers::default());
    root.read_with(cx, |root, cx| {
        assert!(
            root.source
                .read(cx)
                .menu
                .wsl_setup
                .as_ref()
                .unwrap()
                .selected
                .is_none()
        )
    });
}

#[cfg(windows)]
#[gpui::test]
fn frozen_source_rail_cannot_switch_replacement_forms(cx: &mut TestAppContext) {
    let (root, cx) = frozen(Page::AddDevice, cx);
    let source = root.read_with(cx, |root, _| root.source.clone());
    source.update(cx, |view, cx| {
        // Keep switching inside prepared forms, even if the frozen callback regresses.
        open_form(view, &["Private-Alpha"], cx);
        view.menu.page = Some(Page::AddDevice);
    });
    let row = cx.debug_bounds("add-device-source-wsl").unwrap();
    cx.simulate_click(row.center(), Modifiers::default());
    root.read_with(cx, |root, cx| {
        let view = root.source.read(cx);
        assert_eq!(view.menu.page, Some(Page::AddDevice));
        assert!(view.menu.device_setup.is_some());
        assert!(view.menu.wsl_setup.as_ref().unwrap().task.is_none());
    });
}

#[gpui::test]
fn setup_fields_release_and_restore_window_focus_on_dismissal(cx: &mut TestAppContext) {
    for page in [Page::AddDevice, Page::AddWsl] {
        let (view, cx) = cx.add_window_view(|window, cx| departure::fixture(page, window, cx));
        let weak: Vec<_> = view.read_with(cx, |view, _| match page {
            Page::AddDevice => view
                .menu
                .device_setup
                .as_ref()
                .unwrap()
                .fields
                .iter()
                .map(Entity::downgrade)
                .collect(),
            _ => vec![view.menu.wsl_setup.as_ref().unwrap().session.downgrade()],
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            view.update(cx, |view, cx| {
                view.menu.appear.live = true;
                view.dismiss_menu(window, cx);
            });
            assert!(view.read(cx).focus.is_focused(window));
            window.draw(cx).clear(cx);
        });
        assert!(weak.iter().all(|field| field.upgrade().is_none()));
    }
}

struct Dropped(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[gpui::test]
fn departing_wsl_form_does_not_retain_its_pending_observer(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| departure::fixture(Page::AddWsl, window, cx));
    let dropped = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    view.update(cx, |view, cx| {
        let hold = Dropped(dropped.clone());
        view.menu.wsl_setup.as_mut().unwrap().task = Some(cx.spawn(async move |_, _| {
            let _hold = hold;
            std::future::pending::<()>().await;
        }));
    });
    cx.run_until_parked();
    assert_eq!(dropped.load(std::sync::atomic::Ordering::SeqCst), 0);
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
        });
    });
    cx.run_until_parked();
    assert_eq!(dropped.load(std::sync::atomic::Ordering::SeqCst), 1);
}

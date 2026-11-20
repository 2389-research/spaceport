// ABOUTME: Builds private device-theme fixtures without catalog discovery or saved-file writes.
// ABOUTME: Shares exact host identity and input ownership for departure verification.
#![allow(clippy::unwrap_used)]
use super::super::Saved;
use super::*;
use core::prelude::v1::test;

pub(in crate::menu) fn fixture(
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> SheprdWindow {
    let mut view = crate::sidebar::layout_tests::fixture_window(window, cx);
    install(&mut view, "first", window, cx);
    view
}

fn install(
    view: &mut SheprdWindow,
    identity: &str,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) {
    view.open_menu(window, cx);
    let search = cx.new(SearchInput::new);
    search.update(cx, |input, cx| {
        input.set_appearance(view.config.ui.clone(), view.theme.clone(), cx);
        window.focus(&input.focus, cx);
    });
    let mut picker = DeviceThemePicker {
        search,
        names: Theme::BUILTIN_NAMES
            .iter()
            .map(|name| (*name).to_owned())
            .collect(),
        filtered: Vec::new(),
        selected: 0,
        scroll: UniformListScrollHandle::new(),
        error: None,
        preview: None,
        preview_load: None,
        save_load: None,
        discovery: None,
        _subscription: Subscription::new(|| {}),
    };
    picker.filter("");
    view.menu.host = Some(HostMenu {
        id: format!("ssh:fixture-{identity}"),
        saved: Saved::Ssh {
            profile: format!("fixture-{identity}"),
        },
        label: format!("Fixture {identity}"),
        target: format!("fixture@{identity}.invalid"),
        session: "private-fixture".into(),
        selected: None,
        input: None,
        renaming: false,
        error: None,
        theme: Some(picker),
        tags: None,
        scroll: ScrollHandle::new(),
    });
    view.menu.page = Some(Page::DeviceTheme);
}

#[gpui::test]
fn dismissing_device_theme_restores_focus_and_releases_its_search(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture);
    let search = view.read_with(cx, |view, _| {
        view.menu
            .host
            .as_ref()
            .unwrap()
            .theme
            .as_ref()
            .unwrap()
            .search
            .downgrade()
    });
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert!(search.upgrade().unwrap().read(cx).focus.is_focused(window));
        view.update(cx, |view, cx| view.dismiss_menu(window, cx));
        assert!(view.read(cx).focus.is_focused(window));
        window.draw(cx).clear(cx);
    });
    assert!(search.upgrade().is_none());
    view.read_with(cx, |view, _| assert!(view.menu.host.is_none()));
}

struct Frozen {
    source: Entity<SheprdWindow>,
    paint: Paint,
}

impl Render for Frozen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(500.))
            .h(px(560.))
            .child(self.source.update(cx, |view, cx| {
                view.render_device_theme_paint(&self.paint, cx)
            }))
    }
}

#[gpui::test]
fn frozen_device_theme_controls_cannot_act_on_a_replacement_host(cx: &mut TestAppContext) {
    let (frozen, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture(window, cx));
        let paint = source.update(cx, |view, cx| {
            let paint = view.device_theme_paint(cx).unwrap();
            view.dismiss_menu(window, cx);
            install(view, "replacement", window, cx);
            view.menu
                .host
                .as_mut()
                .unwrap()
                .theme
                .as_mut()
                .unwrap()
                .selected = 1;
            paint
        });
        Frozen { source, paint }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let app = cx.debug_bounds("device-theme-row-0").unwrap();
    cx.simulate_mouse_move(app.center(), None, Modifiers::default());
    // Same as app already matches the saved value, even if a broken callback runs.
    // This keeps a regression inside the fixture instead of writing personal config.
    cx.simulate_click(app.center(), Modifiers::default());
    let close = cx.debug_bounds("device-theme-close").unwrap();
    cx.simulate_click(close.center(), Modifiers::default());
    frozen.read_with(cx, |frozen, cx| {
        let view = frozen.source.read(cx);
        let host = view.menu.host.as_ref().unwrap();
        assert_eq!(view.menu.page, Some(Page::DeviceTheme));
        assert_eq!(host.id, "ssh:fixture-replacement");
        assert_eq!(host.target, "fixture@replacement.invalid");
        assert_eq!(host.session, "private-fixture");
        assert_eq!(host.theme.as_ref().unwrap().selected, 1);
    });
}

struct Dropped(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[gpui::test]
fn departing_device_theme_does_not_retain_reader_tasks_or_subscription(cx: &mut TestAppContext) {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let (view, cx) = cx.add_window_view(fixture);
    let counters = view.update(cx, |view, cx| {
        let mut tasks = Vec::new();
        let mut counters = Vec::new();
        for _ in 0..3 {
            let count = Arc::new(AtomicUsize::new(0));
            let held = Dropped(count.clone());
            tasks.push(cx.spawn(async move |_, _| {
                let _held = held;
                std::future::pending::<()>().await;
            }));
            counters.push(count);
        }
        let picker = view.menu.host.as_mut().unwrap().theme.as_mut().unwrap();
        picker.discovery = tasks.pop();
        picker.save_load = tasks.pop();
        picker.preview_load = tasks.pop();
        let subscription = Arc::new(AtomicUsize::new(0));
        let release = subscription.clone();
        picker._subscription = Subscription::new(move || {
            release.fetch_add(1, Ordering::SeqCst);
        });
        counters.push(subscription);
        counters
    });
    cx.run_until_parked();
    assert!(
        counters
            .iter()
            .all(|count| count.load(Ordering::SeqCst) == 0)
    );
    cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));
    cx.run_until_parked();
    assert!(
        counters
            .iter()
            .all(|count| count.load(Ordering::SeqCst) == 1)
    );
}

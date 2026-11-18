// ABOUTME: Runs actual WSL removal against a private catalog in an isolated process.
// ABOUTME: Checks stale confirmation buttons without risking personal saved distributions.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::menu::devices::DeviceSetupPaint;
use gpui::{Context, IntoElement, Modifiers, Render, Window, div};
use std::{
    fs,
    path::PathBuf,
    process::Child,
    thread,
    time::{Duration, Instant},
};

const ROOT: &str = "SPACEPORT_WSL_PAINT_ROOT";
const ALPHA: &str = "Private-Alpha";
const BETA: &str = "Private-Beta";

#[test]
fn removal_paint_preserves_private_catalog_targets_and_confirmation_body() {
    let sandbox = crate::test_sandbox::Sandbox::new();
    let mut command = sandbox.command(std::env::current_exe().unwrap(), "wsl-paint.log");
    command.env(ROOT, &sandbox.dir).args([
        "--exact",
        "menu::appear::tests::device_setup::isolation::private_wsl_catalog_fixture",
        "--ignored",
        "--nocapture",
    ]);
    let mut child = Fixture(command.spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "isolated WSL paint fixture timed out"
        );
        thread::sleep(Duration::from_millis(10));
    };
    assert!(
        status.success(),
        "isolated WSL paint fixture failed: {}",
        fs::read_to_string(sandbox.dir.join("wsl-paint.log")).unwrap()
    );
}

struct Fixture(Child);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[gpui::test]
#[ignore = "private catalog fixture, invoked by the nonignored parent test"]
fn private_wsl_catalog_fixture(cx: &mut TestAppContext) {
    let root = std::env::var_os(ROOT);
    assert!(root.is_some(), "run through the isolated catalog parent");
    let root = PathBuf::from(root.unwrap());
    assert_eq!(
        std::env::var_os("HOME").map(PathBuf::from),
        Some(root.clone())
    );
    assert_eq!(
        std::env::var_os("XDG_STATE_HOME").map(PathBuf::from),
        Some(root.join("state"))
    );
    sheprd_client::add_wsl_host(false, ALPHA, "default").unwrap();
    sheprd_client::add_wsl_host(false, BETA, "work").unwrap();
    sheprd_client::store_wsl_selection(false, Some(BETA)).unwrap();
    stale_buttons_preserve_replacement(cx);
    confirmed_removal_keeps_its_body_and_exact_target(cx);
}

struct Frozen {
    source: Entity<SheprdWindow>,
    paint: DeviceSetupPaint,
}

impl Render for Frozen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().w(px(360.)).child(self.source.update(cx, |view, cx| {
            view.render_device_setup_paint(&self.paint, false, cx)
        }))
    }
}

fn stale_buttons_preserve_replacement(cx: &mut TestAppContext) {
    let (root, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| device_setup_departure_fixture(Page::RemoveWsl, window, cx));
        let paint = source.update(cx, |view, cx| {
            let paint = view.device_setup_paint(cx).unwrap();
            view.dismiss_menu(window, cx);
            view.open_remove_wsl(&format!("wsl:{BETA}"), Point::default(), window, cx);
            paint
        });
        Frozen { source, paint }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    for selector in ["remove-wsl-submit", "remove-wsl-cancel"] {
        let button = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(button.center(), Modifiers::default());
    }
    cx.run_until_parked();
    root.read_with(cx, |root, cx| {
        let view = root.source.read(cx);
        assert_eq!(view.menu.page, Some(Page::RemoveWsl));
        assert_eq!(view.menu.wsl_remove.as_deref(), Some(BETA));
        assert!(view.menu.removing_devices.is_empty());
    });
    let saved = sheprd_client::load_wsl_hosts(false).unwrap();
    assert!(saved.hosts.iter().any(|host| host.distro == ALPHA));
    assert!(saved.hosts.iter().any(|host| host.distro == BETA));
    assert_eq!(saved.selected.as_deref(), Some(BETA));
}

fn confirmed_removal_keeps_its_body_and_exact_target(cx: &mut TestAppContext) {
    let (view, cx) = cx
        .add_window_view(|window, cx| device_setup_departure_fixture(Page::RemoveWsl, window, cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let before = cx.debug_bounds("remove-wsl-submit").unwrap();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.remove_wsl_key(&crate::actions::edit_key("enter"), window, cx);
        })
    });
    hold_departure(&view, cx);
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(cx.debug_bounds("remove-wsl-submit"), Some(before));
    let saved = sheprd_client::load_wsl_hosts(false).unwrap();
    assert!(!saved.hosts.iter().any(|host| host.distro == ALPHA));
    assert!(saved.hosts.iter().any(|host| host.distro == BETA));
    assert_eq!(saved.selected.as_deref(), Some(BETA));
}

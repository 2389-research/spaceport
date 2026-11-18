// ABOUTME: Exercises local history controls against isolated runtime and view files.
// ABOUTME: Prevents the runtime switch from writing Spaceport's private view settings.
use super::*;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    thread,
    time::{Duration, Instant},
};

use crate::test_sandbox as sandbox;

const PROBE: &str = "SPACEPORT_LOCAL_HISTORY_PROBE";
const CLOSE: &str = "SPACEPORT_LOCAL_HISTORY_CLOSE";
const VIEW: &str =
    "[keys]\nprefix = 'ctrl+a'\n[theme]\nname = 'dracula'\n[experimental]\npane_history = false\n";
const RUNTIME: &str =
    "# runtime fixture\n[theme]\nname = 'nord'\n[experimental]\npane_history = false\n";

#[test]
fn local_pane_history_changes_the_runtime_config_only() {
    run_history_fixture(false);
}

#[test]
fn local_pane_history_finishes_when_settings_close() {
    run_history_fixture(true);
}

fn run_history_fixture(close: bool) {
    let sandbox = sandbox::Sandbox::new();
    let view = sandbox.dir.join("config/spaceport/view.toml");
    fs::create_dir_all(view.parent().unwrap()).unwrap();
    fs::write(&view, VIEW).unwrap();
    let runtime = sandbox.dir.join("runtime.toml");
    fs::write(&runtime, RUNTIME).unwrap();
    let mut command = sandbox.command(std::env::current_exe().unwrap(), "history.log");
    command
        .env(PROBE, &view)
        .env(CLOSE, if close { "1" } else { "0" })
        .env("HERDR_CONFIG_PATH", &runtime)
        .args([
            "--exact",
            "settings_window::controls::tests::local_history::local_history_fixture",
            "--nocapture",
        ]);
    let status = wait_for_fixture(command);
    assert!(
        status.success(),
        "local history fixture failed: {}",
        fs::read_to_string(sandbox.dir.join("history.log")).unwrap()
    );
    assert_eq!(
        fs::read_to_string(view).unwrap(),
        VIEW,
        "the runtime switch changed private view settings"
    );
    assert_eq!(
        fs::read_to_string(runtime).unwrap(),
        RUNTIME.replace("pane_history = false", "pane_history = true")
    );
}

fn wait_for_fixture(mut command: Command) -> std::process::ExitStatus {
    let mut child = Fixture(command.spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            panic!("local history fixture timed out");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

struct Fixture(std::process::Child);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[gpui::test]
fn local_history_fixture(cx: &mut TestAppContext) {
    let Some(path) = std::env::var_os(PROBE).map(PathBuf::from) else {
        return;
    };
    let (view, cx) = cx.add_window_view(skill_fixture);
    view.update(cx, |view, cx| {
        view.section = Section::Tabs;
        view.host_history.read = crate::settings_window::host_history::HostHistory::default().read;
        view.shared = Some(crate::herdr_settings::Settings::load_fixture(path).unwrap());
        view.sync_host_history(false, cx);
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_resize(size(px(960.), px(2200.)));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    let toggle = cx.debug_bounds("tabs-pane-history-toggle").unwrap();
    cx.simulate_click(toggle.center(), Default::default());
    if std::env::var_os(CLOSE).is_some_and(|value| value == "1") {
        cx.update(|window, _| window.remove_window());
        drop(view);
        cx.run_until_parked();
        return;
    }
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.shared.as_ref().unwrap().theme_name, "dracula");
    });
}

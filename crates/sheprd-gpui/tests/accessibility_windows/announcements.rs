// ABOUTME: Qualifies real warning and toast announcements through native UIA events.
// ABOUTME: Uses normal configuration reloads, QA previews, and Zen in a private sandbox.
use super::{CAPTURE_LIMIT, GuiTarget, Running, fs, thread};
use std::{
    io::Read as _,
    time::{Duration, Instant},
};

const CONFIG: &str = include_str!("../support/announcement-config.toml");
const WARNING: &str = "Load GUI config:";
const TOAST: &str = "QA preview: a custom notification message.";

impl Running {
    fn start_live_observer(&mut self) {
        for (name, source) in [
            ("uia_live.cs", include_str!("../support/uia_live.cs")),
            ("uia_live.ps1", include_str!("../support/uia_live.ps1")),
        ] {
            fs::write(self.sandbox.dir.join(name), source).unwrap();
        }
        let script = self.sandbox.dir.join("uia_live.ps1");
        self.observer = Some(self.powershell(&script, "live-events.log").spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            self.assert_observer_running();
            if self.sandbox.dir.join("live-events-ready").exists() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "native UIA registration timed out:\n{}",
                self.live_log()
            );
            thread::sleep(Duration::from_millis(25));
        }
        assert!(self.live_log().contains("UIA_LIVE_READY"));
    }

    fn assert_observer_running(&mut self) {
        assert!(
            self.gui.as_mut().unwrap().try_wait().unwrap().is_none(),
            "owned GUI exited"
        );
        assert!(
            self.observer
                .as_mut()
                .unwrap()
                .try_wait()
                .unwrap()
                .is_none(),
            "native observer exited:\n{}",
            self.live_log()
        );
    }

    fn live_log(&self) -> String {
        let mut bytes = Vec::new();
        fs::File::open(self.sandbox.dir.join("live-events.log"))
            .unwrap()
            .take((CAPTURE_LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(
            bytes.len() <= CAPTURE_LIMIT,
            "native UIA capture exceeds its limit"
        );
        // The producer may still be writing the final UTF-8 JSON record.
        let complete = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |at| at + 1);
        String::from_utf8(bytes[..complete].to_vec()).unwrap()
    }

    fn wait_live(&mut self, names: &[&str], after: usize) {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            self.assert_observer_running();
            let log = self.live_log();
            for line in log[after..]
                .lines()
                .filter_map(|line| line.strip_prefix("UIA_LIVE "))
            {
                let record: String = serde_json::from_str(line).unwrap();
                if names.iter().all(|name| record.contains(name)) {
                    return;
                }
            }
            assert!(
                Instant::now() < deadline,
                "missing native live-region event for {names:?}:\n{log}"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn finish_live_observer(&mut self) {
        fs::write(self.sandbox.dir.join("live-events-stop"), "stop").unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(status) = self.observer.as_mut().unwrap().try_wait().unwrap() {
                let log = self.live_log();
                assert!(status.success(), "native observer failed:\n{log}");
                assert!(log.contains("UIA_LIVE_REMOVED"), "{log}");
                eprintln!("{log}");
                break;
            }
            assert!(
                Instant::now() < deadline,
                "native UIA removal timed out:\n{}",
                self.live_log()
            );
            thread::sleep(Duration::from_millis(25));
        }
    }
}

#[test]
#[ignore = "requires an active Windows desktop and HERDR_TEST_BINARY; observes only a private GUI"]
fn native_warnings_and_toasts_deliver_named_live_region_events() {
    let mut running = Running::prepared();
    let config = running
        .sandbox
        .dir
        .join("config/spaceport/config.local.toml");
    fs::write(&config, CONFIG).unwrap();
    running.start_daemon();
    running.start_gui(GuiTarget::Socket);
    running.wait_tree("private hidden-status window", |tree| {
        tree.contains("Document 'Terminal'")
            && tree.contains("MenuItem 'QA'")
            && !tree.contains("StatusBar 'Status bar'")
    });
    running.start_live_observer();

    fs::write(
        &config,
        include_str!("../support/announcement-hidden-invalid.toml"),
    )
    .unwrap();
    running.control(&["config", "reload"]);
    running.wait_tree("visible configuration warning", |tree| {
        tree.contains(WARNING) && tree.contains("native_hidden")
    });
    running.wait_live(&[WARNING, "native_hidden"], 0);

    fs::write(&config, CONFIG).unwrap();
    running.control(&["config", "reload"]);
    running.wait_tree("cleared configuration warning", |tree| {
        !tree.contains(WARNING)
    });
    running.uia_action("-Expand", "QA");
    running.wait_tree("custom preview action", |tree| {
        tree.contains("MenuItem 'Show Custom toast'")
    });
    running.uia_action("-MenuItem", "Show Custom toast");
    running.wait_tree("visible custom toast", |tree| tree.contains(TOAST));
    running.wait_live(&[TOAST], 0);

    running.control(&["run", "toggle_zen_mode"]);
    running.wait_tree("private Zen layout", |tree| {
        tree.contains("Document 'Terminal'")
            && !tree.contains("MenuBar 'Menu bar'")
            && !tree.contains("StatusBar 'Status bar'")
    });
    let after = running.live_log().len();
    fs::write(
        &config,
        include_str!("../support/announcement-zen-invalid.toml"),
    )
    .unwrap();
    running.control(&["config", "reload"]);
    running.wait_tree("visible Zen warning", |tree| {
        tree.contains(WARNING) && tree.contains("native_zen")
    });
    running.wait_live(&[WARNING, "native_zen"], after);
    running.finish_live_observer();
}

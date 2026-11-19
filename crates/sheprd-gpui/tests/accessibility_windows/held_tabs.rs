// ABOUTME: Checks native held shortcuts against private Herdr tabs after visible acknowledgement.
// ABOUTME: Requires observed native repeat flags before accepting unchanged tab counts.
use super::{Duration, GuiTarget, Instant, Running, fs, thread};

#[path = "../support/state_path.rs"]
mod state_path;

impl Running {
    fn held_events(&self) -> usize {
        let path = sheprd_diagnostics::log_file(&state_path::app_state_path(
            &self.sandbox.dir.join("state"),
        ));
        let bytes = fs::read(path).unwrap_or_default();
        let namespace = std::path::Path::new(env!("CARGO_BIN_EXE_spaceport"))
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        bytes
            .split_inclusive(|byte| *byte == b'\n')
            .filter(|line| line.ends_with(b"\n"))
            .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
            .filter(|record| {
                record["namespace"] == namespace
                    && record["message"] == "Native shortcut input"
                    && record["fields"]["is_held"] == true
            })
            .count()
    }

    fn hold_tab_shortcut(&mut self, action: &str) {
        let before = self.held_events();
        let limit = Duration::from_secs(25);
        let deadline = (std::time::SystemTime::now() + limit)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();
        let capture = self.sandbox.dir.join("uia-held.log");
        let mut command =
            self.powershell(&self.sandbox.dir.join("uia_held_tabs.ps1"), "uia-held.log");
        command.args(["-Action", action, "-DeadlineEpochMs", &deadline]);
        let output = self.command_output(command, &capture, limit).unwrap();
        assert!(
            output.contains("HELD_TABS sent=5 released=true"),
            "{output}"
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let observed = self.held_events();
            assert!(
                observed <= before + 5,
                "unexpected held events in private fixture"
            );
            if observed == before + 5 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "native held input did not arrive: {observed} after {before}"
            );
            thread::sleep(Duration::from_millis(25));
        }
        eprintln!("HELD_TABS native {action} received=5");
    }
}

fn tabs(tree: &str, expected: &[&str]) -> bool {
    let mut lines = tree.lines();
    let Some(strip) = lines.find(|line| line.trim_start().starts_with("Tab 'Tabs' ")) else {
        return false;
    };
    let indent = strip.len() - strip.trim_start().len();
    let labels: Vec<_> = lines
        .take_while(|line| line.len() - line.trim_start().len() > indent)
        .filter(|line| line.trim_start().starts_with("TabItem '"))
        .collect();
    labels.len() == expected.len()
        && expected.iter().all(|label| {
            labels
                .iter()
                .any(|line| line.contains(&format!("TabItem '{label}'")))
        })
}

#[test]
#[ignore = "requires active Windows desktop and HERDR_TEST_BINARY; sends held input only to its private window"]
fn native_tab_shortcuts_ignore_held_tail_after_visible_acknowledgement() {
    let mut running = Running::prepared();
    fs::write(
        running.sandbox.dir.join("uia_held_tabs.ps1"),
        include_str!("../support/uia_held_tabs.ps1"),
    )
    .unwrap();
    fs::write(
        running.sandbox.dir.join("config/spaceport/view.toml"),
        "[ui]\nprompt_new_tab_name = false\n",
    )
    .unwrap();
    fs::write(
        running
            .sandbox
            .dir
            .join("config/spaceport/config.local.toml"),
        "language = \"en-US\"\nconfirm_close_tab = false\n",
    )
    .unwrap();
    running.start_daemon();
    running.start_gui(GuiTarget::ShortcutObserver);
    running.wait_tree("one private starting tab", |tree| tabs(tree, &["1"]));
    for labels in [&["1", "2"][..], &["1", "2", "3"][..]] {
        running.hold_tab_shortcut("New");
        running.wait_tree("exactly one new tab per fresh press", |tree| {
            tabs(tree, labels)
        });
    }
    for labels in [&["1", "2"][..], &["1"][..]] {
        running.hold_tab_shortcut("Close");
        running.wait_tree("exactly one closed tab per fresh press", |tree| {
            tabs(tree, labels)
        });
    }
}

// ABOUTME: Qualifies Windows shortcuts with real scan codes, UIA views, and daemon identities.
// ABOUTME: Keeps every window, layout scope, helper, and terminal inside the existing sandbox.
use super::{Duration, Running, identities, snapshot};
use crate::GuiTarget;
use serde::Deserialize;
use sheprd_client::protocol::ClientShellSnapshot;
use std::{collections::BTreeSet, fs, sync::Arc, thread, time::Instant};

#[path = "matrix/console.rs"]
mod console;
#[cfg(test)]
#[path = "matrix/tests.rs"]
mod tests;

#[derive(Debug, Deserialize)]
struct Case {
    chords: Vec<Vec<u32>>,
    id: String,
    outcome: Outcome,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    // keep-sorted start
    Back,
    CycleWindows,
    Digit { number: u8 },
    Down,
    FontLarger,
    FontReset,
    FontSmaller,
    Forward,
    Fullscreen,
    Keybinds,
    Left,
    NextPane,
    NextTab,
    Palette,
    PaneKey { key: String, modifiers: Vec<String> },
    PreviousPane,
    PreviousTab,
    Resume,
    Right,
    Settings,
    Sidebar,
    SplitEditor,
    StatusBar,
    Up,
    Zen,
    // keep-sorted end
}

fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!("../../support/windows_shortcut_cases.json")).unwrap()
}

struct Observer {
    latest: Arc<ClientShellSnapshot>,
}

impl Observer {
    fn attach(running: &Running) -> Self {
        Self {
            latest: snapshot(running),
        }
    }

    fn wait(
        &mut self,
        running: &mut Running,
        label: &str,
        accept: impl Fn(&ClientShellSnapshot) -> bool,
    ) -> Arc<ClientShellSnapshot> {
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            // Herdr focus is connection-local. A fresh passive client seeds its
            // location from the default updated by the GUI's own API request.
            // Projection revisions cannot be compared across these connections.
            self.latest = snapshot(running);
            if accept(&self.latest) {
                return self.latest.clone();
            }
            if Instant::now() >= deadline {
                diagnostics(running, label);
                panic!(
                    "{label}: acknowledgement missing; tabs={} panes={}",
                    self.latest.tabs.len(),
                    self.latest.panes.len()
                );
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn focus_tab(&mut self, running: &mut Running, number: usize) -> Arc<ClientShellSnapshot> {
        let workspace = self.latest.focused_workspace_id.as_ref().unwrap();
        let target = self
            .latest
            .tabs
            .iter()
            .filter(|tab| &tab.workspace_id == workspace)
            .nth(number.checked_sub(1).unwrap())
            .unwrap()
            .tab_id
            .clone();
        running.control(&["run", &format!("focus_tab_{number}")]);
        let focused = self.wait(running, "preparation tab focus", |snapshot| {
            snapshot.focused_tab_id.as_ref() == Some(&target)
        });
        selected_tab(running, &focused);
        focused
    }

    fn tabs(&mut self, running: &mut Running, count: usize) {
        while self.latest.tabs.len() < count {
            let before = self.latest.tabs.len();
            running.control(&["run", "new_tab"]);
            self.wait(running, "one real prepared tab creation", |snapshot| {
                assert!(
                    snapshot.tabs.len() <= before + 1,
                    "preparation created extra tabs"
                );
                snapshot.tabs.len() == before + 1
            });
        }
        assert_eq!(self.latest.tabs.len(), count);
    }
}

fn fixture() -> Running {
    for (name, expected) in [
        ("GITHUB_ACTIONS", "true"),
        ("RUNNER_ENVIRONMENT", "github-hosted"),
        ("RUNNER_OS", "Windows"),
    ] {
        assert_eq!(
            std::env::var(name).as_deref(),
            Ok(expected),
            "native layout matrix needs a disposable Windows runner"
        );
    }
    let mut running = Running::prepared();
    fs::write(
        running.sandbox.dir.join("config/spaceport/view.toml"),
        "[ui]\nprompt_new_tab_name = false\nprompt_new_workspace_name = false\n",
    )
    .unwrap();
    fs::write(
        running
            .sandbox
            .dir
            .join("config/spaceport/config.local.toml"),
        "language = \"en-US\"\n",
    )
    .unwrap();
    for (name, source) in [
        (
            "console_shortcuts.ps1",
            include_str!("../../support/console_shortcuts.ps1"),
        ),
        (
            "native_altgr.cs",
            include_str!("../../support/native_altgr.cs"),
        ),
        (
            "native_profile_id.cs",
            include_str!("../../support/native_profile_id.cs"),
        ),
        (
            "uia_shortcut_matrix.ps1",
            include_str!("../../support/uia_shortcut_matrix.ps1"),
        ),
        (
            "windows_shortcut_cases.json",
            include_str!("../../support/windows_shortcut_cases.json"),
        ),
    ] {
        fs::write(running.sandbox.dir.join(name), source).unwrap();
    }
    running.start_daemon();
    running.start_gui(GuiTarget::Socket);
    running.wait_tree("private connected terminal", |tree| {
        tree.contains("Document 'Terminal'") && tree.contains("TabItem '1'")
    });
    running.read("{ESC}");
    console::prepare_input(&mut running, None);
    running
}

fn diagnostics(running: &mut Running, label: &str) {
    use std::io::Read as _;
    eprintln!("WINDOWS_KEYS failure_stage={label}");
    let capture = running.sandbox.dir.join("shortcut-diagnostic.log");
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_shortcut_matrix.ps1"),
        "shortcut-diagnostic.log",
    );
    command.args(["-CaseId", "console-attachment", "-Diagnostic"]);
    match running.command_output(command, &capture, Duration::from_secs(5)) {
        Ok(output) => eprintln!("{output}"),
        Err(error) => eprintln!("WINDOWS_KEYS diagnostic_failure={error}"),
    }
    for log in ["gui.log", "daemon.log"] {
        if let Ok(file) = fs::File::open(running.sandbox.dir.join(log)) {
            let mut bytes = Vec::new();
            if file.take(16_384).read_to_end(&mut bytes).is_ok() {
                eprintln!(
                    "WINDOWS_KEYS private_log={log}\n{}",
                    String::from_utf8_lossy(&bytes)
                );
            }
        }
    }
}

fn native(running: &mut Running, case: &Case) {
    let before = window_state(running);
    native_with_font(running, case, None, None, None, false);
    if case.outcome != Outcome::Fullscreen {
        assert_eq!(
            window_state(running),
            before,
            "native case changed window IDs or fullscreen state"
        );
    }
}

fn native_with_font(
    running: &mut Running,
    case: &Case,
    marker: Option<&str>,
    width: Option<f64>,
    minimum_before: Option<f64>,
    measure: bool,
) -> String {
    let capture = running.sandbox.dir.join("windows-keys.log");
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_shortcut_matrix.ps1"),
        "windows-keys.log",
    );
    let limit = Duration::from_secs(40);
    let deadline = (std::time::SystemTime::now() + limit)
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    command
        .env("SPACEPORT_DISPOSABLE_LAYOUT_TEST", "github-hosted-windows")
        .args(["-CaseId", &case.id, "-DeadlineEpochMs", &deadline]);
    if let Some(marker) = marker {
        command.args(["-FontMarker", marker]);
    }
    if let Some(width) = width {
        command.args(["-ExpectedWidth", &width.to_string()]);
    }
    if let Some(minimum) = minimum_before {
        command.args(["-MinimumBefore", &minimum.to_string()]);
    }
    if measure {
        command.arg("-Measure");
    }
    let output = match running.command_output(command, &capture, limit) {
        Ok(output) => output,
        Err(error) => {
            diagnostics(running, &format!("native helper {}", case.id));
            panic!("native helper failed for {}: {error}", case.id);
        }
    };
    if !measure {
        assert!(output.contains(&format!("WINDOWS_KEYS native PASS id={} klid=00000415 keys_released=true layouts_restored=true", case.id)), "{output}");
        eprintln!("{output}");
    }
    output
}

fn same_scene(before: &ClientShellSnapshot, after: &ClientShellSnapshot) {
    assert_eq!(
        identities(before),
        identities(after),
        "native input changed the owned runtime scene"
    );
    assert_eq!(before.focused_workspace_id, after.focused_workspace_id);
    assert_eq!(
        before
            .tabs
            .iter()
            .map(|tab| (&tab.workspace_id, &tab.tab_id, tab.number))
            .collect::<Vec<_>>(),
        after
            .tabs
            .iter()
            .map(|tab| (&tab.workspace_id, &tab.tab_id, tab.number))
            .collect::<Vec<_>>(),
        "native case reordered the daemon tabs"
    );
    assert_eq!(
        before
            .panes
            .iter()
            .map(|pane| (&pane.workspace_id, &pane.tab_id, &pane.pane_id))
            .collect::<Vec<_>>(),
        after
            .panes
            .iter()
            .map(|pane| (&pane.workspace_id, &pane.tab_id, &pane.pane_id))
            .collect::<Vec<_>>(),
        "native case changed pane membership or order"
    );
}

fn same_focus(before: &ClientShellSnapshot, after: &ClientShellSnapshot) {
    same_scene(before, after);
    assert_eq!(before.focused_tab_id, after.focused_tab_id);
    assert_eq!(before.focused_pane_id, after.focused_pane_id);
}

fn selected_tab(running: &mut Running, snapshot: &ClientShellSnapshot) {
    let tab = snapshot
        .tabs
        .iter()
        .find(|tab| Some(&tab.tab_id) == snapshot.focused_tab_id.as_ref())
        .unwrap();
    assert!(
        !tab.custom_label,
        "fixture expects default numbered tab labels"
    );
    let selected = format!("TabItem '{}' [selected=True]", tab.number);
    running.wait_tree("native selected tab acknowledgement", |tree| {
        tree.contains(&selected)
    });
}

#[test]
#[ignore = "requires disposable Windows desktop and explicit HERDR_TEST_BINARY; uses actual Polish scan codes"]
fn native_shifted_digits_select_exact_positional_tab_ids() {
    let cases = cases();
    let mut running = fixture();
    let mut observer = Observer::attach(&running);
    let required = cases
        .iter()
        .filter_map(|case| match case.outcome {
            Outcome::Digit { number } => Some(usize::from(number)),
            _ => None,
        })
        .max()
        .unwrap();
    observer.tabs(&mut running, required);
    for case in cases
        .iter()
        .filter(|case| matches!(case.outcome, Outcome::Digit { .. }))
    {
        let Outcome::Digit { number } = case.outcome else {
            unreachable!()
        };
        let before = observer.focus_tab(&mut running, if number == 1 { 2 } else { 1 });
        let workspace = before.focused_workspace_id.as_ref().unwrap();
        let target = before
            .tabs
            .iter()
            .filter(|tab| &tab.workspace_id == workspace)
            .nth(usize::from(number) - 1)
            .unwrap()
            .tab_id
            .clone();
        assert_ne!(before.focused_tab_id.as_ref(), Some(&target));
        native(&mut running, case);
        let after = observer.wait(&mut running, "native positional tab focus", |snapshot| {
            snapshot.focused_tab_id.as_ref() == Some(&target)
        });
        same_scene(&before, &after);
        selected_tab(&mut running, &after);
        eprintln!("WINDOWS_KEYS positional id={} target={target:.80}", case.id);
    }
}

#[test]
#[ignore = "requires disposable Windows desktop and explicit HERDR_TEST_BINARY; checks actual punctuation outcomes"]
fn native_punctuation_reaches_history_panes_popups_groups_and_windows() {
    let cases = cases();
    let find = |outcome| cases.iter().find(|case| case.outcome == outcome).unwrap();
    let mut running = fixture();
    let mut observer = Observer::attach(&running);
    observer.tabs(&mut running, 3);
    observer.focus_tab(&mut running, 1);
    let middle = observer
        .focus_tab(&mut running, 2)
        .focused_pane_id
        .clone()
        .unwrap();
    let before = observer.focus_tab(&mut running, 3);
    let last = before.focused_pane_id.clone().unwrap();
    native(&mut running, find(Outcome::Back));
    let back = observer.wait(&mut running, "native back history", |snapshot| {
        snapshot.focused_pane_id.as_ref() == Some(&middle)
    });
    same_scene(&before, &back);
    selected_tab(&mut running, &back);
    native(&mut running, find(Outcome::Forward));
    let forward = observer.wait(&mut running, "native forward history", |snapshot| {
        snapshot.focused_pane_id.as_ref() == Some(&last)
    });
    same_scene(&before, &forward);
    selected_tab(&mut running, &forward);
    split(&mut running, &mut observer, "split_right");
    let split = split(&mut running, &mut observer, "split_right");
    let tab = split.focused_tab_id.as_ref().unwrap();
    let panes = split
        .panes
        .iter()
        .filter(|pane| &pane.tab_id == tab)
        .map(|pane| pane.pane_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(panes.len(), 3);
    for outcome in [Outcome::NextPane, Outcome::PreviousPane] {
        let before = observer.latest.clone();
        let current = panes
            .iter()
            .position(|pane| Some(pane) == before.focused_pane_id.as_ref())
            .unwrap();
        let offset = if outcome == Outcome::NextPane {
            1
        } else {
            panes.len() - 1
        };
        let target = panes[(current + offset) % panes.len()].clone();
        native(&mut running, find(outcome));
        let after = observer.wait(&mut running, "native pane cycle", |snapshot| {
            snapshot.focused_pane_id.as_ref() == Some(&target)
        });
        same_scene(&before, &after);
    }
    for outcome in [Outcome::Keybinds, Outcome::Settings, Outcome::SplitEditor] {
        let before = snapshot(&running);
        native(&mut running, find(outcome));
        same_focus(&before, &snapshot(&running));
    }
    let initial = window_ids(&mut running);
    assert_eq!(initial.len(), 1);
    running.control(&["run", "new_window"]);
    let deadline = Instant::now() + Duration::from_secs(12);
    let before = loop {
        let current = window_ids(&mut running);
        assert!(
            current.len() <= initial.len() + 1,
            "preparation opened extra windows"
        );
        if current.len() == initial.len() + 1 {
            assert!(initial.is_subset(&current));
            break current;
        }
        assert!(
            Instant::now() < deadline,
            "second owned window did not open"
        );
        thread::sleep(Duration::from_millis(20));
    };
    let scene = snapshot(&running);
    native(&mut running, find(Outcome::CycleWindows));
    assert_eq!(window_ids(&mut running), before);
    assert_eq!(identities(&scene), identities(&snapshot(&running)));
}

fn split(
    running: &mut Running,
    observer: &mut Observer,
    command: &str,
) -> Arc<ClientShellSnapshot> {
    let before: BTreeSet<_> = observer
        .latest
        .panes
        .iter()
        .map(|pane| pane.pane_id.clone())
        .collect();
    running.control(&["run", command]);
    let after = observer.wait(running, "one acknowledged pane split", |snapshot| {
        assert!(
            snapshot.panes.len() <= before.len() + 1,
            "split created extra panes"
        );
        snapshot.panes.len() == before.len() + 1
    });
    let ids: BTreeSet<_> = after
        .panes
        .iter()
        .map(|pane| pane.pane_id.clone())
        .collect();
    assert!(before.is_subset(&ids));
    let created = ids.difference(&before).next().unwrap();
    assert_eq!(
        after.focused_pane_id.as_ref(),
        Some(created),
        "split did not focus its new pane"
    );
    after
}

#[test]
#[ignore = "requires disposable Windows desktop and explicit HERDR_TEST_BINARY; checks retained directional defaults"]
fn native_retained_arrow_defaults_focus_exact_adjacent_panes() {
    let cases = cases();
    let mut running = fixture();
    let mut observer = Observer::attach(&running);
    let left = observer.latest.focused_pane_id.clone().unwrap();
    let right = split(&mut running, &mut observer, "split_right")
        .focused_pane_id
        .clone()
        .unwrap();
    for (outcome, target) in [(Outcome::Left, left), (Outcome::Right, right.clone())] {
        let before = observer.latest.clone();
        assert_ne!(before.focused_pane_id.as_ref(), Some(&target));
        native(
            &mut running,
            cases.iter().find(|case| case.outcome == outcome).unwrap(),
        );
        let after = observer.wait(&mut running, "native horizontal pane focus", |snapshot| {
            snapshot.focused_pane_id.as_ref() == Some(&target)
        });
        same_scene(&before, &after);
    }
    let down = split(&mut running, &mut observer, "split_down")
        .focused_pane_id
        .clone()
        .unwrap();
    for (outcome, target) in [(Outcome::Up, right), (Outcome::Down, down)] {
        let before = observer.latest.clone();
        assert_ne!(before.focused_pane_id.as_ref(), Some(&target));
        native(
            &mut running,
            cases.iter().find(|case| case.outcome == outcome).unwrap(),
        );
        let after = observer.wait(&mut running, "native vertical pane focus", |snapshot| {
            snapshot.focused_pane_id.as_ref() == Some(&target)
        });
        same_scene(&before, &after);
    }
}

fn window_ids(running: &mut Running) -> BTreeSet<u64> {
    window_state(running)
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

fn window_state(running: &mut Running) -> BTreeSet<(u64, bool)> {
    let output: serde_json::Value =
        serde_json::from_str(&running.control_output(&["window", "list", "--json"])).unwrap();
    output["windows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|window| {
            (
                window["id"].as_u64().unwrap(),
                window["fullscreen"].as_bool().unwrap(),
            )
        })
        .collect()
}

#[test]
#[ignore = "requires disposable Windows desktop and explicit HERDR_TEST_BINARY; checks actual revised defaults"]
fn native_revised_defaults_change_their_actual_targets() {
    let cases = cases();
    let find = |outcome| cases.iter().find(|case| case.outcome == outcome).unwrap();
    let mut running = fixture();
    let mut observer = Observer::attach(&running);
    observer.tabs(&mut running, 2);
    let first = observer.focus_tab(&mut running, 1);
    let second = first
        .tabs
        .iter()
        .find(|tab| Some(&tab.tab_id) != first.focused_tab_id.as_ref())
        .unwrap()
        .tab_id
        .clone();
    native(&mut running, find(Outcome::NextTab));
    let after = observer.wait(&mut running, "native next tab", |snapshot| {
        snapshot.focused_tab_id.as_ref() == Some(&second)
    });
    same_scene(&first, &after);
    selected_tab(&mut running, &after);
    native(&mut running, find(Outcome::PreviousTab));
    let after = observer.wait(&mut running, "native previous tab", |snapshot| {
        snapshot.focused_tab_id == first.focused_tab_id
    });
    same_scene(&first, &after);
    selected_tab(&mut running, &after);
    for expected in [true, false] {
        native(&mut running, find(Outcome::Fullscreen));
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            let windows: serde_json::Value =
                serde_json::from_str(&running.control_output(&["window", "list", "--json"]))
                    .unwrap();
            if windows["windows"][0]["fullscreen"].as_bool() == Some(expected) {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "native fullscreen acknowledgement missing"
            );
        }
    }
    for outcome in [
        Outcome::Zen,
        Outcome::StatusBar,
        Outcome::Sidebar,
        Outcome::Palette,
        Outcome::Resume,
    ] {
        native(&mut running, find(outcome));
        same_focus(&first, &snapshot(&running));
    }
    console::fonts(&mut running, &cases);
    same_focus(&first, &snapshot(&running));
}

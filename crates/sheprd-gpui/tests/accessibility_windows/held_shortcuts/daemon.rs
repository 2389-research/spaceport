// ABOUTME: Qualifies held split and custom shortcuts after real daemon effects and app fences.
// ABOUTME: Keeps helper input, CLI children, markers, and queue receipts inside one private fixture.
use super::{
    Duration, GuiTarget, Instant, Read, Running, fs, shortcut_prepared, start_shortcut, thread,
    window_ids,
};
use crate::{CAPTURE_LIMIT, daemon_binary};
use serde_json::Value;
use std::collections::BTreeSet;
use std::os::windows::process::CommandExt as _;

#[derive(Clone, Copy)]
enum Action {
    Custom,
    Split,
}

impl Action {
    fn name(self) -> &'static str {
        match self {
            Self::Custom => "Custom",
            Self::Split => "Split",
        }
    }

    fn field(self) -> &'static str {
        match self {
            Self::Custom => "custom_command",
            Self::Split => "split_right",
        }
    }

    fn input(self, record: &Value, held: bool) -> bool {
        record["message"] == "Native shortcut input"
            && record["fields"][self.field()] == true
            && record["fields"]["is_held"] == held
    }
}

fn ready_after(records: &[Value], index: usize) -> bool {
    records[index + 1..].iter().any(|record| {
        record["message"] == "Native shortcut readiness"
            && record["fields"]["surface_stream"] == true
            && record["fields"]["input_ready"] == true
    })
}

fn custom_queued(records: &[Value]) -> usize {
    records
        .iter()
        .filter(|record| {
            record["message"] == "Native shortcut queued"
                && record["fields"]["custom_command"] == true
        })
        .count()
}

fn helper_output(running: &Running) -> String {
    let mut bytes = Vec::new();
    fs::File::open(running.sandbox.dir.join("uia-held.log"))
        .unwrap()
        .take((CAPTURE_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .unwrap();
    assert!(bytes.len() <= CAPTURE_LIMIT);
    String::from_utf8(bytes).unwrap()
}

fn daemon_result(running: &mut Running, args: &[&str]) -> Value {
    let capture = running.sandbox.dir.join("held-daemon-api.log");
    let mut command = running
        .sandbox
        .command(daemon_binary(), "held-daemon-api.log");
    // Queries run while keys are held; a console window must not take focus.
    command.creation_flags(sheprd_client::CREATE_NO_WINDOW);
    command.args(args);
    let output = running
        .command_output(command, &capture, Duration::from_secs(3))
        .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert!(value["error"].is_null(), "{value}");
    value["result"].clone()
}

fn starting_tab(running: &mut Running) -> Value {
    assert_eq!(window_ids(running).len(), 1);
    let result = daemon_result(running, &["tab", "list"]);
    assert_eq!(result["type"], "tab_list");
    let tabs = result["tabs"].as_array().unwrap();
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0]["pane_count"], 1);
    tabs[0].clone()
}

fn pane_count(running: &mut Running, tab: &str) -> usize {
    let result = daemon_result(running, &["tab", "get", tab]);
    assert_eq!(result["type"], "tab_info");
    assert_eq!(result["tab"]["tab_id"], tab);
    usize::try_from(result["tab"]["pane_count"].as_u64().unwrap()).unwrap()
}

fn panes(running: &mut Running, workspace: &str, tab: &str) -> BTreeSet<String> {
    let result = daemon_result(running, &["pane", "list", "--workspace", workspace]);
    assert_eq!(result["type"], "pane_list");
    let ids: Vec<_> = result["panes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|pane| pane["tab_id"] == tab)
        .map(|pane| pane["pane_id"].as_str().unwrap().to_owned())
        .collect();
    let unique: BTreeSet<_> = ids.iter().cloned().collect();
    assert_eq!(unique.len(), ids.len());
    unique
}

fn hold_after_effect(
    running: &mut Running,
    action: Action,
    mut effect: impl FnMut(&mut Running) -> bool,
) {
    assert!(running.observer.is_none());
    for name in ["held-first.down", "held-effect.ack", "held-release.request"] {
        match fs::remove_file(running.sandbox.dir.join(name)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("cannot clear owned handshake: {error}"),
        }
    }
    let baseline = running.shortcut_records();
    let offset = baseline.len();
    let queued = custom_queued(&baseline);
    let limit = Duration::from_secs(25);
    let deadline = Instant::now() + limit;
    let absolute = (std::time::SystemTime::now() + limit)
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_held_shortcuts.ps1"),
        "uia-held.log",
    );
    command.args(["-Action", action.name(), "-DeadlineEpochMs", &absolute]);
    running.observer = Some(command.spawn().unwrap());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        observe_burst(running, action, offset, queued, deadline, &mut effect);
    }));
    if let Err(payload) = result {
        release_before_unwind(running, deadline);
        std::panic::resume_unwind(payload);
    }
}

fn observe_burst(
    running: &mut Running,
    action: Action,
    offset: usize,
    queued: usize,
    deadline: Instant,
    effect: &mut impl FnMut(&mut Running) -> bool,
) {
    loop {
        if let Some(status) = running.observer.as_mut().unwrap().try_wait().unwrap() {
            panic!(
                "native helper exited before acknowledgement: {status}\n{}",
                helper_output(running)
            );
        }
        let records = running.shortcut_records();
        let first = records
            .iter()
            .enumerate()
            .skip(offset)
            .find(|(_, record)| action.input(record, false))
            .map(|(index, _)| index);
        if running.sandbox.dir.join("held-first.down").exists()
            && first.is_some_and(|first| ready_after(&records, first))
            && effect(running)
        {
            fs::write(running.sandbox.dir.join("held-effect.ack"), b"1").unwrap();
            break;
        }
        assert!(
            Instant::now() < deadline,
            "native first effect or coherent app fence missing"
        );
        thread::sleep(Duration::from_millis(10));
    }
    loop {
        if let Some(status) = running.observer.as_mut().unwrap().try_wait().unwrap() {
            let output = helper_output(running);
            assert!(
                status.success() && output.contains("HELD_SHORTCUT sent=5 released=true"),
                "{status}\n{output}"
            );
            running.observer.take();
            break;
        }
        assert!(
            Instant::now() < deadline,
            "native held helper exceeded its parent deadline"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let receipt_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let records = running.shortcut_records();
        let held: Vec<_> = records
            .iter()
            .enumerate()
            .skip(offset)
            .filter(|(_, record)| action.input(record, true))
            .collect();
        assert!(held.len() <= 5, "unexpected extra native held events");
        if held.len() == 5 && ready_after(&records, held[4].0) {
            assert!(
                held.iter()
                    .all(|(_, record)| record["fields"]["input_ready"] == true)
            );
            assert_eq!(
                records[offset..]
                    .iter()
                    .filter(|record| action.input(record, false))
                    .count(),
                1
            );
            assert_eq!(
                custom_queued(&records),
                queued + usize::from(matches!(action, Action::Custom))
            );
            break;
        }
        assert!(
            Instant::now() < receipt_deadline,
            "native held records or final app fence missing"
        );
        thread::sleep(Duration::from_millis(10));
    }
    eprintln!(
        "HELD_DAEMON native {} received=5 input_ready=true fenced=true",
        action.name()
    );
}

fn release_before_unwind(running: &mut Running, deadline: Instant) {
    if running.observer.is_none() {
        return;
    }
    // Keep Running alive until finally or the independent watchdog can release keys.
    let requested = fs::write(running.sandbox.dir.join("held-release.request"), b"1");
    let mut wait_error = None;
    loop {
        match running.observer.as_mut().unwrap().try_wait() {
            Ok(Some(_)) => {
                running.observer.take();
                return;
            }
            Ok(None) => {}
            Err(error) => wait_error = Some(error),
        }
        if Instant::now() >= deadline {
            // The watchdog expires five seconds before this original parent deadline.
            eprintln!(
                "HELD_HELPER reached parent cleanup deadline: release={requested:?} wait={wait_error:?}"
            );
            crate::stop_children([&mut running.observer]);
            running.observer.take();
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore = "requires active Windows desktop and HERDR_TEST_BINARY; splits only its private tab"]
fn native_split_ignores_held_tail_after_real_daemon_and_surface_acknowledgement() {
    let mut running = shortcut_prepared();
    start_shortcut(&mut running, GuiTarget::ShortcutFenceObserver);
    let tab = starting_tab(&mut running);
    let tab_id = tab["tab_id"].as_str().unwrap();
    let workspace = tab["workspace_id"].as_str().unwrap();
    let mut previous = panes(&mut running, workspace, tab_id);
    assert_eq!(previous.len(), 1);
    for expected in [2, 3] {
        hold_after_effect(&mut running, Action::Split, |running| {
            let count = pane_count(running, tab_id);
            assert!(count <= expected);
            count == expected
        });
        assert_eq!(pane_count(&mut running, tab_id), expected);
        let current = panes(&mut running, workspace, tab_id);
        assert_eq!(current.len(), expected);
        assert!(previous.is_subset(&current));
        previous = current;
    }
    eprintln!("HELD_SPLIT native PASS: two fresh presses, ten held events, three preserved panes");
}

#[test]
#[ignore = "requires active Windows desktop and HERDR_TEST_BINARY; checks owned-key release after a deliberate parent failure"]
fn native_parent_failure_releases_held_keys_before_returning() {
    let mut running = shortcut_prepared();
    start_shortcut(&mut running, GuiTarget::ShortcutFenceObserver);
    let tab = starting_tab(&mut running);
    let tab_id = tab["tab_id"].as_str().unwrap();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        hold_after_effect(&mut running, Action::Split, |running| {
            if pane_count(running, tab_id) == 2 {
                panic!("expected native parent failure after the first real split");
            }
            false
        });
    }))
    .expect_err("the deliberate parent failure must occur");
    assert_eq!(
        failure.downcast_ref::<&str>(),
        Some(&"expected native parent failure after the first real split")
    );
    assert!(
        running.observer.is_none(),
        "the held helper must exit before the failure returns"
    );
    let output = running.shell_probe(r#"
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
Add-Type @"
using System.Runtime.InteropServices;
public static class ReleasedKeys {
  [DllImport("user32.dll")] public static extern short GetAsyncKeyState(int key);
}
"@
foreach ($key in @(0x11,0x10,0x44)) {
  if (([ReleasedKeys]::GetAsyncKeyState($key) -band 0x8000) -ne 0) { throw 'A fixture-owned key stayed down after parent failure' }
}
[Console]::WriteLine('HELD_KEYS released=true')
"#, Duration::from_secs(20)).unwrap();
    assert!(output.contains("HELD_KEYS released=true"), "{output}");
    assert_eq!(pane_count(&mut running, tab_id), 2);
    eprintln!(
        "HELD_PARENT_FAILURE native PASS: original failure retained, owned helper reaped, all fixture keys released"
    );
}

fn markers(running: &Running) -> usize {
    let entries: Vec<_> = fs::read_dir(running.sandbox.dir.join("agbt-counter"))
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_dir());
            let slot: usize = entry.file_name().to_str().unwrap().parse().unwrap();
            assert!((1..=16).contains(&slot));
            slot
        })
        .collect();
    assert!(entries.len() <= 16);
    entries.len()
}

#[test]
#[ignore = "requires active Windows desktop and HERDR_TEST_BINARY; invokes only a private directory-marker command"]
fn native_custom_command_ignores_held_tail_after_real_execution_and_surface_acknowledgement() {
    let mut running = shortcut_prepared();
    fs::create_dir(running.sandbox.dir.join("agbt-counter")).unwrap();
    let command = (1..=16)
        .map(|slot| format!(r#"mkdir "%TEMP%\agbt-counter\{slot}""#))
        .collect::<Vec<_>>()
        .join(" || ");
    fs::write(running.sandbox.dir.join("config.toml"), format!(
        "[[keys.command]]\nkey = \"ctrl+shift+f8\"\ntype = \"shell\"\ndescription = \"Private native repeat fixture\"\ncommand = '{command}'\n"
    )).unwrap();
    start_shortcut(&mut running, GuiTarget::ShortcutFenceObserver);
    starting_tab(&mut running);
    assert_eq!(markers(&running), 0);
    for expected in [1, 2] {
        hold_after_effect(&mut running, Action::Custom, |running| {
            let count = markers(running);
            assert!(count <= expected);
            count == expected
        });
        assert_eq!(markers(&running), expected);
    }
    assert_eq!(custom_queued(&running.shortcut_records()), 2);
    eprintln!(
        "HELD_CUSTOM native PASS: two fresh executions, ten held events, no extra queued invocation"
    );
}

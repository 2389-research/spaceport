// ABOUTME: Exercises recovery, restart and blocked allocation through three normal Spaceport launches.
// ABOUTME: Retains original bytes, private backup permissions and both widths of the painted warning.
use super::*;
use serde_json::{Value, json};
use sheprd_control_protocol::protocol::tabs::{TabInfo, TabKind};
use std::os::unix::fs::PermissionsExt;

const TABS: &str = "browser-tabs.json";
const LAYOUTS: &str = "editor-groups.json";
const SAVED_WORKSPACE: &str = "saved-state-fixture";
const REPAIR: [&str; 4] = [
    "saved tabs or layouts need repair",
    TABS,
    LAYOUTS,
    "restart spaceport before opening new tabs",
];

fn file_bytes(path: &Path) -> Vec<u8> {
    sheprd_state_file::read(path, 1024 * 1024).unwrap().unwrap()
}

struct Files {
    tabs: Vec<u8>,
    layouts: Vec<u8>,
}

impl Files {
    fn snapshot(test: &Qualification, folder: &Path, phase: &str) -> Self {
        let tabs = file_bytes(&folder.join(TABS));
        let layouts = file_bytes(&folder.join(LAYOUTS));
        let mut hashes = String::new();
        for (name, bytes) in [(TABS, &tabs), (LAYOUTS, &layouts)] {
            let file = format!("{phase}-{name}");
            test.record(&file, bytes);
            hashes.push_str(&format!("{}  {file}\n", hex(&Sha256::digest(bytes))));
        }
        test.record(&format!("{phase}.sha256"), hashes);
        Self { tabs, layouts }
    }
}

fn seed(folder: &Path, host: &str) {
    fs::create_dir_all(folder).unwrap();
    let row = |id: u64, kind: &str, title: &str| {
        json!({"id": id, "scope": host, "workspace_id": SAVED_WORKSPACE,
               "location": {"kind": kind}, "title": title})
    };
    let tabs = json!({"tabs": [row(2, "devices", "Good"), row(70, "future", "Future")], "next": 3});
    fs::write(folder.join(TABS), format!(" \n{tabs}\n")).unwrap();
    let group = json!({"hidden": [{"kind":"page", "id":300}], "share":1});
    let layout = json!({"scope":host, "workspace":SAVED_WORKSPACE,
                        "layout":{"groups":[group], "active":0}});
    fs::write(
        folder.join(LAYOUTS),
        json!({"layouts":[layout]}).to_string(),
    )
    .unwrap();
}

fn check_backup(test: &Qualification, folder: &Path, original: &[u8]) {
    let backups = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("browser-tabs.json.rejected-")
        })
        .collect::<Vec<_>>();
    let [backup] = backups.as_slice() else {
        panic!("expected one rejected tabs backup: {backups:?}")
    };
    let bytes = file_bytes(backup);
    let mode = fs::metadata(backup).unwrap().permissions().mode() & 0o7777;
    test.record("rejected-tabs.bin", &bytes);
    test.record(
        "backup.json",
        json!({
            "name": backup.file_name().unwrap().to_str().unwrap(),
            "permissions": format!("{mode:04o}"),
            "sha256": hex(&Sha256::digest(&bytes))
        })
        .to_string(),
    );
    assert_eq!(mode, 0o600);
    assert_eq!(bytes, original);
}

fn devices(id: u64, title: &str, workspace: &str, host: &str) -> TabInfo {
    TabInfo {
        id,
        kind: TabKind::Devices,
        title: title.into(),
        location: None,
        workspace: workspace.into(),
        host: host.into(),
        opened_by: None,
    }
}

fn saved_tabs(files: &Files) -> Vec<TabInfo> {
    let saved: Value = serde_json::from_slice(&files.tabs).unwrap();
    assert_eq!(saved["next"], 302);
    let mut tabs = Vec::new();
    for row in saved["tabs"].as_array().unwrap() {
        assert_eq!(row["location"], json!({"kind":"devices"}));
        assert!(row["origin"].is_null());
        tabs.push(devices(
            row["id"].as_u64().unwrap(),
            row["title"].as_str().unwrap(),
            row["workspace_id"].as_str().unwrap(),
            row["scope"].as_str().unwrap(),
        ));
    }
    tabs.sort_by_key(|tab| tab.id);
    tabs
}

fn listed(test: &Qualification, label: &str) -> Vec<TabInfo> {
    let mut rows: Vec<TabInfo> =
        serde_json::from_value(test.reply(&["tab", "list"], label, 0)).unwrap();
    rows.sort_by_key(|tab| tab.id);
    rows
}

fn saved_devices_layout(bytes: &[u8], host: &str, live: &str) -> bool {
    let layouts: Value = serde_json::from_slice(bytes).unwrap();
    layouts["layouts"].as_array().unwrap().iter().any(|row| {
        row["scope"] == host
            && row["workspace"] == live
            && row["layout"]["groups"]
                .as_array()
                .unwrap()
                .iter()
                .any(|group| group["pick"] == json!({"kind":"page", "id":301}))
    })
}

fn ambiguous_layout(bytes: &[u8], host: &str) -> String {
    let mut saved: Value = serde_json::from_slice(bytes).unwrap();
    let rows = saved["layouts"].as_array_mut().unwrap();
    let mut matching = rows
        .iter_mut()
        .filter(|row| row["scope"] == host && row["workspace"] == SAVED_WORKSPACE);
    let row = matching
        .next()
        .expect("the synthetic workspace layout survived");
    assert!(matching.next().is_none());
    // JSON values cannot represent duplicate keys. Insert this one raw pick
    // after serialization, retaining the daemon workspace and the saved floor.
    const MARKER: &str = "REPLACE_WITH_AMBIGUOUS_PICK";
    row["layout"] = json!({"active":0,"groups":[{"pick":MARKER,"share":1}]});
    let output = saved.to_string();
    let marker = json!(MARKER).to_string();
    assert_eq!(output.matches(&marker).count(), 1);
    output.replace(&marker, r#"{"kind":"page","id":300,"id":3}"#)
}

fn repair_score(text: &str) -> (bool, usize) {
    let text = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let contains = |part: &str| match part {
        TABS | LAYOUTS => {
            // Native OCR sometimes reads the extension's period as a comma.
            let misread = part.replace('.', ",");
            text.split_whitespace().any(|word| {
                let name = word.strip_suffix(',').unwrap_or(word);
                name == part || name == misread
            })
        }
        _ => text.contains(part),
    };
    let words = REPAIR
        .into_iter()
        .flat_map(str::split_whitespace)
        .filter(|&word| contains(word))
        .count();
    let readable = REPAIR.into_iter().all(contains);
    (readable, words)
}

#[test]
fn complete_native_repair_lines_are_readable_at_both_widths() {
    for lines in [
        "., Saved tabs or layouts need repair. Check browser-tabs,json and editor-groups.json, then

restart Spaceport before opening new tabs.",
        "‘Saved tabs or layouts need repair. Check

© browser-tabs,json and editor-groups,json, then

restart Spaceport before opening new tabs.",
    ] {
        assert!(repair_score(lines).0, "{lines}");
    }
}

#[test]
fn clipped_or_missing_repair_guidance_is_unreadable() {
    for lines in [
        "© Saved tabs or layouts need repair. Check browser-tabs.json and editor-groups.json, then r.",
        "{ © Saved tabs or layouts need repair. Check brows...",
        "Saved tabs or layouts need repair. Check browser-tabs,json, then restart Spaceport before opening new tabs.",
    ] {
        assert!(!repair_score(lines).0, "{lines}");
    }
}

#[test]
#[ignore = "requires pinned Linux x64 Herdr, private Xvfb, capture tools and a fresh evidence directory"]
fn native_saved_state_recovery() {
    let binary = daemon_binary();
    let mut test = prepare(&binary);
    let desktop = display::Display::start(&test.isolated.sandbox);
    let witness = test.start_server(&binary);
    test.record("herdr-welcome.json", witness.receipt());
    let live = &witness.workspace;
    assert_ne!(live, SAVED_WORKSPACE);
    let sandbox = test.isolated.sandbox.dir.clone();
    let folder = sandbox.join("state/spaceport");
    let scope = sheprd_browser::Scope::local(&test.isolated.sandbox.socket());
    let host = scope.as_str();
    let good = devices(2, "Good", SAVED_WORKSPACE, host);
    let created = devices(301, "Devices", live, host);

    seed(&folder, host);
    let original = Files::snapshot(&test, &folder, "seed");
    test.launch(&desktop, live, "a");
    assert_eq!(listed(&test, "a-restored"), std::slice::from_ref(&good));
    let opened = test.reply(&["devices", "open"], "a-open", 0);
    assert_eq!(opened, json!({"kind":"opened","id":301,"workspace":live}));
    let first_list = listed(&test, "a-open-tabs");
    assert_eq!(first_list, [good.clone(), created]);
    // The control reply precedes the window poll that queues its group layout.
    let deadline = Instant::now() + TIMEOUT;
    while !saved_devices_layout(&file_bytes(&folder.join(LAYOUTS)), host, live) {
        assert!(
            Instant::now() < deadline,
            "GUI A did not save its Devices layout"
        );
        thread::sleep(POLL);
    }
    test.quit("a");
    let first_files = Files::snapshot(&test, &folder, "after-a");
    assert_eq!(saved_tabs(&first_files), first_list);
    assert!(saved_devices_layout(&first_files.layouts, host, live));
    check_backup(&test, &folder, &original.tabs);

    test.launch(&desktop, live, "b");
    assert_eq!(listed(&test, "b-restored"), saved_tabs(&first_files));
    test.reply(&["tab", "close", "301"], "b-close", 0);
    assert_eq!(listed(&test, "b-survivor"), std::slice::from_ref(&good));
    test.quit("b");
    let second_files = Files::snapshot(&test, &folder, "after-b");
    assert_eq!(saved_tabs(&second_files), std::slice::from_ref(&good));

    fs::write(
        folder.join(LAYOUTS),
        ambiguous_layout(&second_files.layouts, host),
    )
    .unwrap();
    let before = Files::snapshot(&test, &folder, "before-c");
    assert_eq!(before.tabs, second_files.tabs);
    let xid = test.launch(&desktop, live, "c");
    let remaining = listed(&test, "c-restored");
    assert_eq!(remaining, [good]);
    let mut captures = Vec::new();
    for width in [900, 640] {
        let size = format!("{width}x600");
        test.reply(&["window", "size", &size], &format!("resize-{width}"), 0);
        test.await_state(
            "online",
            Some(live),
            Some(width),
            &format!("online-{width}"),
        );
        desktop.await_size(&test.isolated, &xid, width);
        let refused = test.reply(&["devices", "open"], &format!("refused-{width}"), 1);
        assert_eq!(refused["error"]["code"], "refused");
        assert!(
            refused["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Could not open the Devices overview; the window says why")
        );
        captures.push((width, desktop.capture(&test, &xid, width)));
    }
    assert_eq!(listed(&test, "c-after-refusals"), remaining);
    test.quit("c");
    let after = Files::snapshot(&test, &folder, "after-c");
    assert_eq!(after.tabs, before.tabs);
    assert_eq!(after.layouts, before.layouts);
    test.stop_server();
    witness.shutdown(&test.evidence.join("herdr-events.txt"));
    assert!(!sandbox.join("started").exists());
    let mut unreadable = Vec::new();
    for (width, frames) in captures {
        if !desktop.best_frame(&test, width, &frames, repair_score) {
            unreadable.push(width);
        }
    }
    desktop.stop(&test);
    let evidence = test.evidence.clone();
    drop(test);
    assert!(!sandbox.exists(), "private sandbox survived cleanup");
    fs::write(evidence.join("cleanup.txt"), "All three GUIs reaped. Herdr survived GUI C, received TERM, and exited. Startup trap empty. Xvfb reaped. Sandbox removed.\n").unwrap();
    assert!(
        unreadable.is_empty(),
        "repair warning, filenames or restart guidance unreadable at {unreadable:?}px; inspect repair-*-text.txt and PNGs"
    );
    fs::write(evidence.join("complete.txt"), "Saved-state recovery, restart and refusal verified. Both native warning frames contain the repair guidance.\n").unwrap();
}

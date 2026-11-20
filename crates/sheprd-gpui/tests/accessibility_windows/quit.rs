// ABOUTME: Checks native Quit while a real main-window popup owns keyboard focus.
// ABOUTME: Requires clean GUI exit while the private Herdr daemon and tabs survive.
use super::{Duration, GuiTarget, Instant, Running, daemon_binary, fs, thread};
use std::{collections::BTreeSet, os::windows::process::CommandExt as _};

fn tab_ids(running: &mut Running) -> BTreeSet<String> {
    let mut command = running.sandbox.command(daemon_binary(), "quit-tabs.log");
    command.args(["tab", "list"]);
    command.creation_flags(sheprd_client::CREATE_NO_WINDOW);
    let output = running
        .command_output(
            command,
            &running.sandbox.dir.join("quit-tabs.log"),
            Duration::from_secs(3),
        )
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(value["error"].is_null(), "{value}");
    assert_eq!(value["result"]["type"], "tab_list");
    value["result"]["tabs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tab| tab["tab_id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
#[ignore = "requires an active Windows desktop and HERDR_TEST_BINARY; quits only the owned GUI"]
fn native_quit_with_a_popup_exits_the_gui_and_preserves_daemon_tabs() {
    let mut running = Running::prepared();
    fs::write(
        running
            .sandbox
            .dir
            .join("config/spaceport/config.local.toml"),
        "language = \"en-US\"\n",
    )
    .unwrap();
    running.start_daemon();
    running.start_gui(GuiTarget::Socket);
    running.wait_tree("private terminal", |tree| {
        tree.contains("Document 'Terminal'") && !tree.contains("Button 'Reconnect now'")
    });
    let before = tab_ids(&mut running);
    assert!(!before.is_empty());
    running.invoke("Shortcuts");
    running.wait_tree("shortcut popup", |tree| {
        tree.contains("Edit 'Search shortcuts...'")
    });
    let mut command = running.powershell(&running.sandbox.dir.join("uia_tree.ps1"), "quit-key.log");
    command.args(["-Keys", "^+q", "-ExpectExit"]);
    let output = running
        .command_output(
            command,
            &running.sandbox.dir.join("quit-key.log"),
            Duration::from_secs(20),
        )
        .unwrap();
    assert!(
        output.contains("UIA_KEY expected_exit=true sent=true"),
        "{output}"
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = running.gui.as_mut().unwrap().try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "native Quit did not exit the popup's GUI"
        );
        thread::sleep(Duration::from_millis(25));
    };
    assert!(
        status.success(),
        "native Quit crashed instead of exiting: {status}"
    );
    assert!(
        running
            .daemon
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none()
    );
    assert_eq!(tab_ids(&mut running), before);
    eprintln!(
        "QUIT native PASS: popup keyboard Quit exits cleanly and preserves the private daemon and tab IDs"
    );
}

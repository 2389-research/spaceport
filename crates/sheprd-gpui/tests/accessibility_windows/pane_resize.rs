// ABOUTME: Qualifies Herdr pane resizing through actual native prefix and arrow keys.
// ABOUTME: Checks native text geometry and preserves both real daemon pane identities.
use super::{Duration, Instant, Running, fs, thread};
use std::collections::BTreeSet;

fn panes(running: &mut Running, workspace: &str) -> BTreeSet<String> {
    let result = running.daemon_result(&["pane", "list", "--workspace", workspace]);
    assert_eq!(result["type"], "pane_list");
    let panes = result["panes"].as_array().unwrap();
    let ids = panes
        .iter()
        .map(|pane| pane["pane_id"].as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), panes.len());
    ids
}

#[test]
#[ignore = "requires active Windows desktop and HERDR_TEST_BINARY; resizes only a private split"]
fn native_herdr_prefix_resize_moves_text_bounds_and_preserves_both_panes() {
    let mut running = Running::start();
    running.wait_tree("private resize terminal", |tree| {
        tree.contains("Document 'Terminal' [Value, Text]")
    });
    let tabs = running.daemon_result(&["tab", "list"]);
    assert_eq!(tabs["type"], "tab_list");
    let tabs = tabs["tabs"].as_array().unwrap();
    assert_eq!(tabs.len(), 1);
    let workspace = tabs[0]["workspace_id"].as_str().unwrap().to_owned();
    let initial = panes(&mut running, &workspace);
    assert_eq!(initial.len(), 1);
    running.control(&["run", "split_right"]);
    let deadline = Instant::now() + Duration::from_secs(10);
    let before = loop {
        let ids = panes(&mut running, &workspace);
        if ids.len() == 2 {
            break ids;
        }
        assert!(
            Instant::now() < deadline,
            "the private split did not reach the daemon"
        );
        thread::sleep(Duration::from_millis(25));
    };
    assert!(initial.is_subset(&before));
    for (name, source) in [
        (
            "uia_terminal.ps1",
            include_str!("../support/uia_terminal.ps1"),
        ),
        ("uia_live.cs", include_str!("../support/uia_live.cs")),
    ] {
        fs::write(running.sandbox.dir.join(name), source).unwrap();
    }
    let capture = running.sandbox.dir.join("resize-capture.log");
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_terminal.ps1"),
        "resize-capture.log",
    );
    command.args(["-Action", "Resize"]);
    let output = running
        .command_output(command, &capture, Duration::from_secs(30))
        .unwrap();
    assert!(output.contains("TERMINAL_RESIZE native=true prefix=true left=true right=true restored=true released=true"), "{output}");
    assert_eq!(
        panes(&mut running, &workspace),
        before,
        "keyboard resize changed pane ownership"
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
    eprintln!(
        "TERMINAL_RESIZE native PASS: real prefix mode, both arrows, changed and restored native text bounds, both daemon panes preserved"
    );
}

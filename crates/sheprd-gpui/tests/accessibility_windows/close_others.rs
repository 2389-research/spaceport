// ABOUTME: Verifies one real Close Others consent before either owned tab closes.
// ABOUTME: Preserves the selected tab, private checkout bytes, and daemon across cancellation.
use super::{Duration, GuiTarget, Running, daemon_binary, fs};

fn daemon_tabs(running: &mut Running, workspace: Option<&str>) -> Vec<serde_json::Value> {
    let capture = running.sandbox.dir.join("tab-list.log");
    let mut command = running.sandbox.command(daemon_binary(), "tab-list.log");
    command.args(["tab", "list"]);
    if let Some(workspace) = workspace {
        command.args(["--workspace", workspace]);
    }
    let output = running
        .command_output(command, &capture, Duration::from_secs(20))
        .unwrap();
    let response: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(response["result"]["type"], "tab_list");
    response["result"]["tabs"].as_array().unwrap().clone()
}

fn all_tabs(tree: &str) -> bool {
    ["1", "2", "3"]
        .iter()
        .all(|label| tree.contains(&format!("TabItem '{label}'")))
}

fn consent(tree: &str) -> bool {
    let tree = tree.replace(['\u{2068}', '\u{2069}'], "");
    tree.contains("Close 2 tabs?")
        && tree.contains("Text '1 / 2'")
        && tree.contains("This terminates 2 Herdr terminal panes")
        && tree.contains("closes 0 client views")
        && tree.contains("Button 'Close 2 tabs' [Invoke]")
        && tree.contains("Button 'Cancel' [Invoke]")
}

fn open_others(running: &mut Running) {
    running.uia_action("-GroupActionsFor", "3");
    running.wait_tree("the selected tab's group menu", |tree| {
        tree.lines()
            .any(|line| line.trim_start().starts_with("Menu '"))
    });
    // Split and Close Group follow Close Others; Up wraps from no selection.
    running.read("{UP}|{UP}|{UP}|{ENTER}");
    let tree = running.wait_tree("one complete Close Others consent", consent);
    assert!(all_tabs(&tree), "tabs closed before consent:\n{tree}");
    assert!(!tree.contains("Do not ask again"), "{tree}");
}

#[test]
#[ignore = "requires an active Windows desktop and HERDR_TEST_BINARY; closes only two private test-created tabs"]
fn native_close_others_cancels_every_target_then_confirms_once() {
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
        "language = \"en-US\"\nconfirm_close_tab = false\n",
    )
    .unwrap();
    running.start_daemon();
    running.start_gui(GuiTarget::Socket);
    let marker = running.sandbox.dir.join("keep.txt");
    fs::write(&marker, "preserve the private checkout\n").unwrap();
    running.wait_tree("initial private tab", |tree| tree.contains("TabItem '1'"));
    for label in ["2", "3"] {
        running.invoke("New tab");
        running.wait_tree("new private tab", |tree| {
            tree.contains(&format!("TabItem '{label}' [selected=True]"))
                && tree.contains("Document 'Terminal'")
        });
    }
    let before = daemon_tabs(&mut running, None);
    assert_eq!(before.len(), 3);
    let kept = before.iter().find(|tab| tab["number"] == 3).unwrap();
    let kept_id = kept["tab_id"].as_str().unwrap().to_owned();
    let workspace = kept["workspace_id"].as_str().unwrap().to_owned();
    let ids = |tabs: &[serde_json::Value]| {
        tabs.iter()
            .map(|tab| tab["tab_id"].as_str().unwrap().to_owned())
            .collect::<std::collections::BTreeSet<_>>()
    };
    running.control(&["run", "split_editor"]);
    running.wait_tree("an empty second client group", |tree| {
        all_tabs(tree) && tree.matches("Button 'Tab actions' [Invoke]").count() == 2
    });

    open_others(&mut running);
    running.read("{ENTER}");
    running.wait_tree("cancelled whole batch", |tree| {
        all_tabs(tree) && !tree.contains("Close 2 tabs?")
    });
    assert_eq!(
        ids(&daemon_tabs(&mut running, Some(&workspace))),
        ids(&before)
    );
    assert_eq!(
        fs::read_to_string(&marker).unwrap(),
        "preserve the private checkout\n"
    );

    open_others(&mut running);
    running.invoke("Close 2 tabs");
    running.wait_tree(
        "only the kept tab remains, without another prompt",
        |tree| {
            tree.contains("TabItem '1' [selected=True]")
                && !tree.contains("TabItem '3'")
                && !tree.contains("TabItem '2'")
                && !tree.contains("Close 2 tabs?")
                && !tree.contains("Close tab?")
        },
    );
    let after = daemon_tabs(&mut running, Some(&workspace));
    assert_eq!(after.len(), 1);
    assert_eq!(after[0]["tab_id"], kept_id);
    assert_eq!(after[0]["number"], 3);
    assert_eq!(after[0]["label"], "1");
    assert_eq!(
        fs::read_to_string(marker).unwrap(),
        "preserve the private checkout\n"
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
        "CLOSE_OTHERS native PASS: cancelled all targets, confirmed once, kept stable tab number 3"
    );
}

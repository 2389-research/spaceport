// ABOUTME: Closes only a test-created Herdr workspace through native confirmation.
// ABOUTME: Preserves the original workspace, checkout bytes, and owned daemon.
use super::{Running, fs};

const WORKSPACE: &str = "spaceport-close-fixture";

fn consent(tree: &str) -> bool {
    let text = tree.replace(['\u{2068}', '\u{2069}'], "");
    text.contains("Text 'Close workspace?'")
        && text.contains(&format!("Text '{WORKSPACE}'"))
        && text.contains("Text 'This closes 1 terminal panes across 1 workspaces")
        && text.contains("Button 'Close workspace' [Invoke]")
        && text.contains("Button 'Cancel' [Invoke]")
}

#[test]
#[ignore = "requires an active Windows desktop and HERDR_TEST_BINARY; closes only a private test-created workspace"]
fn native_workspace_consent_names_the_target_and_preserves_other_work() {
    let mut running = Running::start();
    let original = format!(
        "ListItem '{}'",
        running.sandbox.dir.file_name().unwrap().to_str().unwrap()
    );
    running.wait_tree("original workspace", |tree| {
        tree.contains(&original) && tree.contains("Document 'Terminal'")
    });
    let marker = running.sandbox.dir.join("keep.txt");
    fs::write(&marker, "keep the private checkout\n").unwrap();
    running.invoke("New workspace");
    running.wait_tree("workspace creation dialog", |tree| {
        tree.contains("Button 'Create'")
    });
    running.read(&format!("{WORKSPACE}{{ENTER}}"));
    let created = format!("ListItem '{WORKSPACE}'");
    running.wait_tree("created workspace", |tree| {
        tree.contains(&created) && tree.contains(&original) && tree.contains("Document 'Terminal'")
    });

    running.read("^b+d");
    let tree = running.wait_tree("named workspace consent", consent);
    assert!(!tree.contains("Do not ask again"), "{tree}");
    running.read("{ENTER}");
    running.wait_tree("cancelled workspace consent", |tree| {
        tree.contains(&created) && !tree.contains("Button 'Close workspace'")
    });
    assert!(
        running
            .daemon
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none()
    );
    assert_eq!(
        fs::read_to_string(&marker).unwrap(),
        "keep the private checkout\n"
    );

    running.read("^b+d");
    running.wait_tree("fresh workspace consent", consent);
    running.read("{TAB}|{ENTER}");
    running.wait_tree("closed only the created workspace", |tree| {
        !tree.contains(&created)
            && tree.contains(&original)
            && tree.contains("Document 'Terminal'")
            && !tree.contains("Button 'Close workspace'")
    });
    assert!(
        running
            .daemon
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none()
    );
    assert_eq!(
        fs::read_to_string(marker).unwrap(),
        "keep the private checkout\n"
    );
}

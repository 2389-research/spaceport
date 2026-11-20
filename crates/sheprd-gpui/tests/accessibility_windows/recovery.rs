// ABOUTME: Checks native recovery-button focus and real private-daemon retry.
// ABOUTME: Starts only the explicitly selected test daemon after missing-runtime advice appears.
use super::{GuiTarget, Running};

fn focused(output: &str, key: &str, button: &str) -> bool {
    output.contains(&format!("focus after {key} -> Button '{button}'"))
}

#[test]
#[ignore = "requires an active Windows desktop; uses an absent private socket"]
fn native_reconnect_button_participates_in_chrome_focus() {
    let mut running = Running::prepared();
    running.start_gui(GuiTarget::Socket);
    running.wait_tree("reconnect card", |tree| {
        tree.contains("Button 'Reconnect now'")
    });
    let output = running.read("^b{F6}|^b{F6}|^b{F6}|{TAB}|+{TAB}");
    assert!(focused(&output, "^b{F6}", "Reconnect now"), "{output}");
    assert!(focused(&output, "{TAB}", "Reconnect now"), "{output}");
    assert!(focused(&output, "+{TAB}", "Reconnect now"), "{output}");
    let refused = running
        .press_button("{ENTER}", "missing fixture button")
        .unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("The focused control is not the expected private button"),
        "{refused}"
    );
}

#[test]
#[ignore = "requires an active Windows desktop and HERDR_TEST_BINARY; starts only an owned private daemon"]
fn native_missing_herdr_retry_attaches_after_keyboard_activation() {
    for activation in ["{ENTER}", " "] {
        let mut running = Running::prepared();
        running.start_gui(GuiTarget::Local);
        running.wait_tree("missing Herdr advice", |tree| {
            tree.contains("Button 'Install Herdr'") && tree.contains("Button 'Try again'")
        });
        let output = running.read("^b{F6}|^b{F6}|^b{F6}|{TAB}|+{TAB}|{TAB}");
        assert!(focused(&output, "^b{F6}", "Install Herdr"), "{output}");
        assert!(focused(&output, "+{TAB}", "Install Herdr"), "{output}");
        assert!(focused(&output, "{TAB}", "Try again"), "{output}");
        assert!(
            output
                .lines()
                .filter_map(|line| line.split_once(" -> "))
                .next_back()
                .is_some_and(|(_, control)| control.starts_with("Button 'Try again'")),
            "{output}"
        );

        running.start_daemon();
        let before = running.read("");
        assert!(before.contains("Button 'Try again'"), "{before}");
        running.press_button(activation, "Try again").unwrap();
        let workspace = format!(
            "ListItem '{}'",
            running.sandbox.dir.file_name().unwrap().to_str().unwrap()
        );
        running.wait_tree("keyboard recovery to the private workspace", |tree| {
            tree.contains(&workspace)
                && tree.contains("Document 'Terminal'")
                && !tree.contains("Button 'Try again'")
                && !tree.contains("Button 'Install Herdr'")
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
    }
}

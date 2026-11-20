// ABOUTME: Qualifies Herdr terminal text and caret ranges through native UI Automation.
// ABOUTME: Keeps native output-change delivery separate from audible screen-reader speech.
use super::{Duration, Running, fs};

#[test]
#[ignore = "requires an active Windows desktop and HERDR_TEST_BINARY; reads only the private terminal"]
fn native_herdr_text_caret_line_bounds_and_output_change_events() {
    let mut running = Running::start();
    running.wait_tree("native terminal text provider", |tree| {
        tree.contains("Document 'Terminal' [Value, Text]")
    });
    for (name, source) in [
        (
            "uia_terminal.ps1",
            include_str!("../support/uia_terminal.ps1"),
        ),
        ("uia_live.cs", include_str!("../support/uia_live.cs")),
    ] {
        fs::write(running.sandbox.dir.join(name), source).unwrap();
    }
    let capture = running.sandbox.dir.join("terminal-capture.log");
    let command = running.powershell(
        &running.sandbox.dir.join("uia_terminal.ps1"),
        "terminal-capture.log",
    );
    let output = running
        .command_output(command, &capture, Duration::from_secs(30))
        .unwrap();
    for receipt in [
        "TERMINAL_A11Y text=true native_text_changed=true unselected_caret=true caret_line_bounds=true",
        "TERMINAL_A11Y observer_removed=true",
    ] {
        assert!(output.contains(receipt), "{output}");
    }
    eprintln!(
        "TERMINAL_A11Y native PASS: actual text, changed-output event, collapsed caret, contained line bounds, observer removal"
    );
}

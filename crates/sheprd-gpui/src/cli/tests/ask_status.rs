// ABOUTME: Keeps read-only app status inspection on the existing global CLI options.
// ABOUTME: Preserves exact window selection and JSON parsing without starting the GUI.
use super::*;

#[test]
fn ask_inspection_uses_existing_status_json_and_window_options() {
    let parsed = LaunchOptions::parse(["status", "--window", "7", "--json"]).unwrap();
    let LaunchMode::Control(invocation) = parsed.mode else {
        panic!("expected control command");
    };
    assert_eq!(invocation.command, control::ControlCommand::Status);
    assert!(invocation.json);
    assert_eq!(invocation.window, Some(7));
}

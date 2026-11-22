// ABOUTME: Verifies the real macOS keep-awake assertion names and scopes.
// ABOUTME: Checks only this test process's assertions and proves guard cleanup.
use super::*;
use std::process::Command;

#[test]
#[ignore = "creates real macOS power assertions"]
fn spaceport_power_assertions_follow_scope_and_release() {
    for (scope, reason, count) in [
        (AwakeScope::System, "Spaceport keeps the system awake", 1),
        (AwakeScope::Display, "Spaceport keeps the display awake", 2),
    ] {
        let guard = hold(scope).unwrap();
        let held = own_assertions();
        drop(guard);
        let released = own_assertions();

        let name = format!("named: \"{reason}\"");
        let named: Vec<_> = held.iter().filter(|line| line.contains(&name)).collect();
        assert_eq!(named.len(), count, "owned assertions: {held:?}");
        assert!(
            named
                .iter()
                .any(|line| line.contains("PreventUserIdleSystemSleep"))
        );
        assert_eq!(
            named
                .iter()
                .any(|line| line.contains("PreventUserIdleDisplaySleep")),
            scope == AwakeScope::Display,
            "owned assertions: {held:?}"
        );
        assert!(
            released.iter().all(|line| !line.contains(&name)),
            "owned assertions after release: {released:?}"
        );
    }
}

fn own_assertions() -> Vec<String> {
    let output = Command::new("pmset")
        .args(["-g", "assertions"])
        .output()
        .unwrap();
    assert!(output.status.success(), "pmset failed: {}", output.status);
    let prefix = format!("pid {}(", std::process::id());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter(|line| line.trim_start().starts_with(&prefix))
        .map(str::to_owned)
        .collect()
}

// ABOUTME: Keep the native input handoff's decisive state in bounded failure logs.
// ABOUTME: Distinguish wrong focus from a shell that has not reached its prompt.
use super::live;
use crate::smoke::{handoff_diagnostic, timeout_diagnostic};

#[test]
fn matching_focus_does_not_hide_an_unready_shell() {
    let diagnostic = handoff_diagnostic(&live("Loading profiles"), "w0", "t0");
    let logged: String = format!(
        "step=8 elapsed=20s {diagnostic} {}",
        "navigation".repeat(500)
    )
    .chars()
    .take(600)
    .collect();
    for decisive in [
        "workspace_match=true",
        "tab_match=true",
        "shell_ready=false",
        "cursor=Some(CursorState",
        "focused_workspace=w0",
        "focused_tab=t0",
        "expected_workspace=w0",
        "expected_tab=t0",
    ] {
        assert!(logged.contains(decisive), "{decisive} absent from {logged}");
    }
}

#[test]
fn ready_prompt_does_not_hide_wrong_focus() {
    let diagnostic = handoff_diagnostic(&live("LIVE> "), "other-workspace", "other-tab");
    assert!(diagnostic.contains("workspace_match=false"), "{diagnostic}");
    assert!(diagnostic.contains("tab_match=false"), "{diagnostic}");
    assert!(diagnostic.contains("shell_ready=true"), "{diagnostic}");
}

#[test]
fn full_terminal_screen_does_not_push_the_handoff_out_of_the_log_tail() {
    let screen = std::iter::repeat_n("terminal row", 40)
        .collect::<Vec<_>>()
        .join("\n");
    let diagnostic = timeout_diagnostic("step=8 shell_ready=false", &screen);
    assert!(
        diagnostic
            .lines()
            .rev()
            .take(40)
            .any(|line| line.contains("step=8 shell_ready=false")),
        "handoff diagnostic absent from the bounded log tail"
    );
}

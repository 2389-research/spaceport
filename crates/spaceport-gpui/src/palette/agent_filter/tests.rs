use super::*;

#[test]
fn alt_and_a_navigator_letter_choose_a_filter() {
    let parse = |text| from_keystroke(&crate::keymap::parse_keystroke(text).unwrap());
    for (text, expected) in [
        ("alt-a", AgentFilter::All),
        ("alt-b", AgentFilter::Blocked),
        ("alt-w", AgentFilter::Working),
        ("alt-i", AgentFilter::Idle),
        ("alt-d", AgentFilter::Done),
    ] {
        assert_eq!(parse(text), Some(expected), "{text}");
    }
    for text in [
        "b",
        "shift-b",
        "ctrl-b",
        "cmd-b",
        "alt-shift-b",
        "ctrl-alt-b",
        "alt-x",
        "alt-1",
    ] {
        assert_eq!(parse(text), None, "{text}");
    }
}

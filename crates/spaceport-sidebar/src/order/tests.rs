use super::*;

#[test]
fn steps_wrap_and_start_from_an_end() {
    let rows = [(0, "a"), (0, "b"), (1, "c")];
    for (command, current, expected) in [
        (Command::NextWorkspace, Some(0), Some(1)),
        (Command::NextWorkspace, Some(2), Some(0)),
        (Command::NextWorkspace, None, Some(0)),
        (Command::PreviousWorkspace, Some(0), Some(2)),
        (Command::PreviousWorkspace, Some(2), Some(1)),
        (Command::PreviousWorkspace, None, Some(2)),
        (Command::NextAgent, Some(1), Some(2)),
        (Command::PreviousAgent, None, Some(2)),
    ] {
        assert_eq!(step(&rows, current, 0, command), expected, "{command:?}");
    }
    for command in [Command::NextWorkspace, Command::PreviousAgent] {
        assert_eq!(step(&[], None, 0, command), None);
    }
    assert_eq!(step(&rows, Some(0), 0, Command::Tab), None);
}

#[test]
fn numbers_count_from_one() {
    let rows = [(0, "a"), (1, "b"), (0, "c"), (1, "d")];
    // A numbered workspace counts the selected host's rows only.
    assert_eq!(step(&rows, None, 0, Command::WorkspaceNumber(2)), Some(2));
    assert_eq!(step(&rows, None, 1, Command::WorkspaceNumber(1)), Some(1));
    assert_eq!(step(&rows, None, 1, Command::WorkspaceNumber(3)), None);
    assert_eq!(step(&rows, None, 0, Command::WorkspaceNumber(0)), None);
    // A numbered agent counts every host's.
    assert_eq!(step(&rows, None, 0, Command::AgentNumber(4)), Some(3));
    assert_eq!(step(&rows, None, 0, Command::AgentNumber(5)), None);
    assert_eq!(step(&rows, None, 0, Command::AgentNumber(0)), None);
}

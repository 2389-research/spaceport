#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use std::io::{BufRead, BufReader, Read};

/// The agent exits at once on TERM, but leaves a tool behind that ignores
/// it; stopping the chat must still end the tool.
#[test]
fn stopping_ends_a_tool_that_ignores_term_after_the_agent_exits() {
    let dir = tempfile::tempdir().unwrap();
    // The tool installs its TERM trap before it reports itself on the
    // inherited stdout, so the pid arrives only once it is ignoring TERM.
    let script = "(trap '' TERM; echo $$; exec sleep 30) & exec sleep 30";
    let spawned = spawn(
        &OsString::from("/bin/sh"),
        &[OsString::from("-c"), OsString::from(script)],
        &[],
        dir.path(),
    )
    .unwrap();
    let mut output = BufReader::new(spawned.output);
    let mut reported = String::new();
    assert!(
        output.read_line(&mut reported).unwrap() > 0,
        "the tool never started"
    );
    let tool = rustix::process::Pid::from_raw(reported.trim().parse().unwrap()).unwrap();

    spawned.process.stop();

    // `stop` reaps the agent, so the tool holds the pipe's last writer:
    // end of file is the tool's exit, which needs no deadline to observe.
    let mut rest = Vec::new();
    output.read_to_end(&mut rest).unwrap();
    assert!(
        rustix::process::test_kill_process(tool).is_err(),
        "the tool outlived its chat"
    );
}

#[cfg(unix)]
#[test]
fn the_agent_gets_the_environment_it_was_started_with() {
    let dir = tempfile::tempdir().unwrap();
    let mut spawned = spawn(
        &OsString::from("/bin/sh"),
        &[
            OsString::from("-c"),
            OsString::from("printf %s \"$HERDR_ACP_TEST\""),
        ],
        &[("HERDR_ACP_TEST", "1")],
        dir.path(),
    )
    .unwrap();
    let mut seen = String::new();
    spawned.output.read_to_string(&mut seen).unwrap();
    assert_eq!(seen, "1");
}

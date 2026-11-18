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
    let (started, started_rx) = std::sync::mpsc::channel();
    let (ended, ended_rx) = std::sync::mpsc::channel();
    let reader = thread::spawn(move || {
        let mut reported = String::new();
        let line = output
            .read_line(&mut reported)
            .ok()
            .filter(|read| *read > 0)
            .map(|_| reported);
        if started.send(line).is_err() {
            return;
        }
        let mut rest = Vec::new();
        let _ = ended.send(output.read_to_end(&mut rest).is_ok());
    });
    // The pipe's end is the signal; the timeout only bounds a failing run.
    let bound = Duration::from_secs(10);
    let reported = started_rx
        .recv_timeout(bound)
        .expect("the tool never started")
        .expect("the tool never reported itself");
    let tool = rustix::process::Pid::from_raw(reported.trim().parse().unwrap()).unwrap();

    spawned.process.stop();

    ended_rx
        .recv_timeout(bound)
        .expect("the tool outlived its chat");
    reader.join().unwrap();
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

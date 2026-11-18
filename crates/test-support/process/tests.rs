use super::*;
use std::{
    io::Read as _,
    process::{Child, Command, Stdio},
};

#[cfg(target_os = "linux")]
#[path = "tests/linux.rs"]
mod linux;

/// A child whose end of a pipe closes when it exits, reaped or not.
fn sleep() -> Child {
    Command::new("/bin/sleep")
        .arg("60")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}

#[test]
fn reports_running_as_alive_and_zombie_and_reaped_as_dead() {
    let mut child = sleep();
    let pid = child.id();
    let mut exited = child.stdout.take().unwrap();
    let running = is_dead(pid);

    child.kill().unwrap();
    assert!(!running.unwrap());
    // `sleep` writes nothing, so this blocks until the child's end of the pipe
    // closes at its exit: the zombie the next check reads.
    assert_eq!(
        exited.read(&mut [0u8; 1]).unwrap(),
        0,
        "killed child did not exit"
    );
    assert!(is_dead(pid).unwrap());

    child.wait().unwrap();
    assert!(is_dead(pid).unwrap());
}

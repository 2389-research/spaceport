// ABOUTME: Exercises completed shell response boundaries before exposing captured output.
// ABOUTME: Uses an isolated shell and explicit buffered frames to avoid pipe timing assumptions.
#![allow(clippy::unwrap_used)]
use super::*;

#[cfg(unix)]
#[test]
fn completed_oversized_output_breaks_the_session() {
    let home = tempfile::tempdir().unwrap();
    let mut command = Command::new("/bin/sh");
    command
        .arg("-s")
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut shell = Shell::start(command).unwrap();
    // The reader can deliver the final payload bytes and completion marker
    // together. Validate the entire completed payload before returning it.
    shell.buffer = vec![b'x'; LIMIT + 1];
    shell
        .buffer
        .extend_from_slice(format!("\n{} 0\n", shell.marker).as_bytes());
    assert!(matches!(
        shell.run("true", Duration::from_secs(5)),
        Err(Error::Size)
    ));
    assert!(matches!(
        shell.run("true", Duration::from_secs(5)),
        Err(Error::Unreachable)
    ));
}

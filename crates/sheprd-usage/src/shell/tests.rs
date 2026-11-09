// ABOUTME: Exercises completed shell response boundaries before exposing captured output.
// ABOUTME: Uses an isolated shell and explicit buffered frames to avoid pipe timing assumptions.
#![allow(clippy::unwrap_used)]
use super::*;

fn shell() -> (tempfile::TempDir, Shell) {
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
    let shell = Shell::start(command).unwrap();
    (home, shell)
}

#[test]
fn completed_oversized_output_breaks_the_session() {
    let (_home, mut shell) = shell();
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

#[test]
fn exact_limit_payload_accepts_each_completion_marker_split() {
    let (_home, initial) = shell();
    let completion_length = format!("\n{} 0\n", initial.marker).len();
    for split in 0..=completion_length {
        let (_home, mut shell) = shell();
        let completion = format!("\n{} 0\n", shell.marker);
        shell.buffer = vec![b'x'; LIMIT];
        shell
            .buffer
            .extend_from_slice(&completion.as_bytes()[..split]);
        let (sender, output) = mpsc::sync_channel(1);
        sender
            .send(Ok(completion.as_bytes()[split..].to_vec()))
            .unwrap();
        shell.output = output;
        let result = shell.run("true", Duration::from_secs(5));
        assert!(result.is_ok(), "split {split}: {:?}", result.as_ref().err());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.len(), LIMIT);
        assert!(result.stdout.bytes().all(|byte| byte == b'x'));
    }
}

#[test]
fn fragmented_oversized_payload_breaks_the_session() {
    let (_home, mut shell) = shell();
    shell.buffer = vec![b'x'; LIMIT + 1];
    shell.buffer.push(b'\n');
    assert!(matches!(
        shell.run("true", Duration::from_secs(5)),
        Err(Error::Size)
    ));
    assert!(matches!(
        shell.run("true", Duration::from_secs(5)),
        Err(Error::Unreachable)
    ));
}

#[test]
fn completion_status_has_bounded_overhead() {
    let (_home, mut shell) = shell();
    shell.buffer = vec![b'x'; LIMIT];
    shell
        .buffer
        .extend_from_slice(format!("\n{} {}\n", shell.marker, "1".repeat(12)).as_bytes());
    assert!(matches!(
        shell.run("true", Duration::from_secs(5)),
        Err(Error::Size)
    ));
    assert!(matches!(
        shell.run("true", Duration::from_secs(5)),
        Err(Error::Unreachable)
    ));
}

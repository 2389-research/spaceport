// ABOUTME: Exercises the screenshot transfer protocol with a real POSIX shell and filesystem.
// ABOUTME: Checks disconnects on either side of the host retention acknowledgement.
use super::*;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    process::Stdio,
};

fn valid_path(path: &str) -> bool {
    is_staged_file(path, NAME)
}

#[test]
fn confirmed_file_survives_disconnect_and_only_an_explicit_discard_removes_it() {
    let png = b"\x89PNG\r\n\x1a\n\x00\xff\nretain\n";
    for (retain, discard) in [(false, false), (true, false), (true, true)] {
        let mut child = spaceport_client::new_command("/bin/sh")
            .args(["-c", &command(NAME, png.len())])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(png).unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        let mut path = String::new();
        stdout.read_line(&mut path).unwrap();
        let path = path.trim_end();
        assert!(valid_path(path));
        let dir = std::path::Path::new(path).parent().unwrap();
        assert_eq!(fs::read(path).unwrap(), png);
        assert_eq!(fs::metadata(dir).unwrap().permissions().mode() & 0o077, 0);
        if retain {
            stdin.write_all(b"retain\n").unwrap();
            let mut ack = String::new();
            stdout.read_line(&mut ack).unwrap();
            assert_eq!(ack, "retained\n");
        }
        if discard {
            stdin.write_all(b"discard\n").unwrap();
        }
        drop(stdin);
        child.wait().unwrap();
        assert_eq!(dir.exists(), retain && !discard);
        if dir.exists() {
            fs::remove_file(path).unwrap();
            fs::remove_dir(dir).unwrap();
        }
    }
}

#[test]
fn partial_transfer_is_never_offered_for_paste() {
    let mut child = spaceport_client::new_command("/bin/sh")
        .args(["-c", &command(NAME, 123)])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"short").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn only_generated_absolute_screenshot_paths_can_reach_the_terminal() {
    let valid = "/tmp/herdr-upload.AbcD12345678/screenshot.png";
    assert!(valid_path(valid));
    // The host resolves its own temporary directory, which macOS spells through
    // the /private/tmp link, so the staging prefix is not a `/tmp` one.
    assert!(valid_path(
        "/private/tmp/herdr-upload.AbcD12345678/screenshot.png"
    ));
    for path in [
        valid.replace("AbcD12345678", "../123456789"),
        valid.replace("AbcD12345678", "abc;rm -rf ~"),
        valid.replace("screenshot.png", "other.png"),
        valid.replace("screenshot.png", "sub/screenshot.png"),
        valid.replace("AbcD12345678", "short"),
        valid.replace("AbcD12345678", "AbcD123456789"),
        format!("{valid}\nwhoami"),
        format!("{valid} extra"),
    ] {
        assert!(!valid_path(&path));
    }
}

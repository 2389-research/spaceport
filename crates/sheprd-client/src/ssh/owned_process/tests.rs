// ABOUTME: Checks guarded SSH command resolution and argument boundaries.
// ABOUTME: Uses executable files and actual child output without remote access.
use super::*;
use std::io::Read;

#[test]
fn a_missing_bare_executable_keeps_the_spawn_error() {
    let directory = tempfile::tempdir().unwrap();
    let mut command = Command::new("spaceport-missing-owned-ssh");
    command.env("PATH", directory.path());
    let result = OwnedChild::spawn(command, None, Stdio::null());
    assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::NotFound));
}

#[test]
fn command_path_and_arguments_keep_their_boundaries() {
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("fixture ssh");
    crate::test_executable::write(
        &script,
        "#!/bin/sh\nprintf '%s\\n' \"$1\" \"$OWNED_TEST_VALUE\"\n",
        0o700,
    )
    .unwrap();
    let mut command = Command::new("fixture ssh");
    command
        .current_dir(directory.path())
        .env("PATH", ".")
        .env("OWNED_TEST_VALUE", "private fixture")
        .arg("$(touch unwanted); ' quoted");
    let mut child = OwnedChild::spawn(command, None, Stdio::piped()).unwrap();
    let mut output = String::new();
    child
        .take_stdout()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    assert_eq!(output, "$(touch unwanted); ' quoted\nprivate fixture\n");
    child.finish();
    assert!(!directory.path().join("unwanted").exists());
}

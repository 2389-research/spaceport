// ABOUTME: Tests the draft transfer's basename boundary and its cancellation before network I/O.
// ABOUTME: Confirms a shell-significant filename survives private staging as one argument.
use super::*;

#[test]
fn file_names_are_one_bounded_component() {
    for name in [
        "",
        ".",
        "..",
        "../draft",
        "sub/file",
        "bad\nname",
        &"x".repeat(256),
    ] {
        assert!(!valid_filename(name), "{name:?}");
    }
    assert!(valid_filename("my agent's $report.pdf"));
}

#[test]
fn cancelled_file_never_opens_a_transport() {
    let transfer = MobileFileTransfer::new();
    transfer.cancel();
    assert!(matches!(
        ImageStager::default().stage_file(
            &ConnectedSsh::default(),
            "draft.txt".into(),
            vec![0, 255],
            transfer
        ),
        Err(MobileError::FileCancelled)
    ));
}

#[test]
fn shell_paste_names_exact_file() {
    let path = "/tmp/herdr-upload.Ab1234567890/my agent's $report.pdf";
    let staged = StagedImage {
        path: path.to_owned(),
        keep: Mutex::new(None),
    };
    let output = spaceport_client::new_command("/bin/sh")
        .args(["-c", &format!("printf '%s' {}", staged.paste_path())])
        .output()
        .unwrap();
    assert_eq!(output.stdout, path.as_bytes());
}

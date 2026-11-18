// ABOUTME: Exercises text-only acquisition against real private AppKit pasteboards.
// ABOUTME: Keeps file attachments and image data out of plain text probes.
use super::*;

#[test]
fn text_mode_reads_text_instead_of_a_copied_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().canonicalize().unwrap().join("report.txt");
    std::fs::write(&path, b"private fixture").unwrap();
    objc2::rc::autoreleasepool(|_| {
        let board = pasteboard(&[&[
            ("public.file-url", file_url(&path)),
            ("public.utf8-plain-text", b"report.txt".to_vec()),
            ("public.png", vec![1, 2, 3]),
        ]]);
        let item = read_pasteboard(&board, Mode::Text).unwrap();
        assert_eq!(
            item.and_then(|item| item.text()).as_deref(),
            Some("report.txt")
        );
        board.clearContents();
    });
}

#[test]
fn text_mode_ignores_an_image_only_pasteboard() {
    objc2::rc::autoreleasepool(|_| {
        let board = pasteboard(&[&[("public.png", vec![1, 2, 3])]]);
        assert!(read_pasteboard(&board, Mode::Text).unwrap().is_none());
        board.clearContents();
    });
}

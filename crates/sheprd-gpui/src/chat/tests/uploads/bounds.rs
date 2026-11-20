// ABOUTME: Verifies bounded SSH upload admission and whole-batch acceptance.
// ABOUTME: Keeps rejected copies on the existing original-host cleanup path.
use super::*;
use sheprd_chat_protocol::MAX_ATTACHMENTS;

#[gpui::test]
fn preparing_a_picture_does_not_block_another_files_upload(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    let dir = tempfile::tempdir().unwrap();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let pane = view.chats.panes_mut().next().unwrap();
            pane.target.route = Route::Ssh("fixture-host".into());
            let preparing = pane.begin_files(1);
            let id = pane.target.pane_id.clone();
            view.attach_chat_files(&id, vec![dir.path().join("missing.txt")], window, cx);
            let pane = view.chats.pane_mut(&id).unwrap();
            assert_eq!(pane.uploading(), 2);
            assert!(pane.finish_files(&preparing));
        })
    });
    cx.run_until_parked();
}

#[gpui::test]
fn ssh_uploads_refuse_a_second_batch_until_the_first_finishes(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    let dir = tempfile::tempdir().unwrap();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let pane = view.chats.panes_mut().next().unwrap();
            pane.target.route = Route::Ssh("fixture-host".into());
            let id = pane.target.pane_id.clone();
            view.attach_chat_files(&id, vec![dir.path().join("missing.txt")], window, cx);
            view.attach_chat_files(&id, vec![dir.path().join("also-missing.txt")], window, cx);
            let pane = view.chats.pane(&id).unwrap();
            assert_eq!(pane.uploading(), 1);
            assert!(pane.error.is_some());
        })
    });
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.chats.panes().next().unwrap().uploading(), 0);
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let id = view.chats.panes().next().unwrap().target.pane_id.clone();
            view.attach_chat_files(&id, vec![dir.path().join("retry.txt")], window, cx);
            assert_eq!(view.chats.pane(&id).unwrap().uploading(), 1);
        })
    });
    cx.run_until_parked();
}

#[gpui::test]
fn a_full_composer_starts_no_ssh_upload(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    let dir = tempfile::tempdir().unwrap();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let pane = view.chats.panes_mut().next().unwrap();
            pane.target.route = Route::Ssh("fixture-host".into());
            pane.attachments = (0..MAX_ATTACHMENTS)
                .map(|n| PathBuf::from(format!("/remote/{n}.txt")))
                .collect();
            let id = pane.target.pane_id.clone();
            view.attach_chat_files(&id, vec![dir.path().join("missing.txt")], window, cx);
            let pane = view.chats.pane(&id).unwrap();
            assert_eq!(pane.uploading(), 0);
            assert_eq!(pane.attachments.len(), MAX_ATTACHMENTS);
            assert!(pane.error.is_some());
        })
    });
    cx.run_until_parked();
}

#[gpui::test]
fn ssh_uploads_reserve_only_the_remaining_attachment_capacity(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    let dir = tempfile::tempdir().unwrap();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let pane = view.chats.panes_mut().next().unwrap();
            pane.target.route = Route::Ssh("fixture-host".into());
            pane.attachments = (0..MAX_ATTACHMENTS - 1)
                .map(|n| PathBuf::from(format!("/remote/{n}.txt")))
                .collect();
            let id = pane.target.pane_id.clone();
            let paths = (0..MAX_ATTACHMENTS + 1)
                .map(|n| dir.path().join(format!("missing-{n}.txt")))
                .collect();
            view.attach_chat_files(&id, paths, window, cx);
            let pane = view.chats.pane(&id).unwrap();
            assert_eq!(pane.uploading(), 1);
            assert!(pane.error.is_some());
        })
    });
    cx.run_until_parked();
}

#[gpui::test]
fn an_upload_that_no_longer_fits_returns_the_whole_batch_for_cleanup(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let pane = view.chats.panes_mut().next().unwrap();
            let id = pane.target.pane_id.clone();
            let work = pane.begin_files(2);
            pane.attachments = (0..MAX_ATTACHMENTS - 1)
                .map(|n| PathBuf::from(format!("/remote/{n}.txt")))
                .collect();
            let before = pane.attachments.clone();
            let uploaded = Ok(vec![
                "/remote/first.txt".into(),
                "/remote/second.txt".into(),
            ]);
            assert!(!view.finish_chat_upload(&id, &work, &uploaded, window, cx));
            let pane = view.chats.pane(&id).unwrap();
            assert_eq!(pane.attachments, before);
            assert_eq!(pane.uploading(), 0);
            assert!(pane.error.is_some());
        })
    });
}

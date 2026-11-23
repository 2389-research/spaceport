// ABOUTME: Exercises asynchronous file work across chat conversation changes.
// ABOUTME: Keeps old dialogs and uploads from modifying replacement conversations.
use super::{
    overlay::{attach_with, open},
    *,
};
use gpui::TestAppContext;
use spaceport_client::ConnectTarget;

mod bounds;

fn replace(view: &gpui::Entity<crate::SpaceportWindow>, cx: &mut gpui::VisualTestContext) {
    view.update(cx, |view, _| {
        let mut target = view.chats.panes().next().unwrap().target.clone();
        target.source = Source::Program(EndpointId::random());
        view.chats
            .reconcile(vec![target], Instant::now(), &mut |_, _| {
                Err(Error::Attach(spaceport_chat_core::Error::Closed))
            });
    });
}

#[gpui::test]
fn a_file_dialog_cannot_attach_to_a_replacement_conversation(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    let pane = view.read_with(cx, |view, _| {
        view.chats.panes().next().unwrap().target.pane_id.clone()
    });
    cx.update(|window, cx| view.update(cx, |view, cx| view.browse_chat_files(&pane, window, cx)));
    replace(&view, cx);
    cx.simulate_path_prompt_response(|_| Some(vec![PathBuf::from("/fixture/old.txt")]));
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(view.chats.panes().next().unwrap().attachments.is_empty())
    });
}

#[gpui::test]
fn a_file_dialog_cannot_attach_to_a_new_epoch(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    let pane = view.read_with(cx, |view, _| {
        view.chats.panes().next().unwrap().target.pane_id.clone()
    });
    cx.update(|window, cx| view.update(cx, |view, cx| view.browse_chat_files(&pane, window, cx)));
    view.update(cx, |view, _| {
        view.chats
            .pane_mut(&pane)
            .unwrap()
            .session
            .receive(snapshot(cursor(2, 0), vec![]))
            .unwrap();
    });
    cx.simulate_path_prompt_response(|_| Some(vec![PathBuf::from("/fixture/old.txt")]));
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(view.chats.panes().next().unwrap().attachments.is_empty())
    });
}

#[gpui::test]
fn a_failed_upload_cannot_change_a_replacement_conversations_error(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.txt");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let pane = view.chats.panes_mut().next().unwrap();
            pane.target.route = Route::Ssh("fixture-host".into());
            let id = pane.target.pane_id.clone();
            view.attach_chat_files(&id, vec![missing], window, cx);
        })
    });
    replace(&view, cx);
    view.update(cx, |view, _| {
        view.chats.panes_mut().next().unwrap().error = Some("current error".into())
    });
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.chats.panes().next().unwrap().error.as_deref(),
            Some("current error")
        )
    });
}

#[gpui::test]
fn a_finished_upload_cannot_attach_or_debit_a_new_epochs_work(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let pane = view.chats.panes_mut().next().unwrap();
            let id = pane.target.pane_id.clone();
            let old = pane.begin_files(2);
            pane.session
                .receive(snapshot(cursor(2, 0), vec![]))
                .unwrap();
            let current = pane.begin_files(1);
            assert!(!view.finish_chat_upload(
                &id,
                &old,
                &Ok(vec!["/remote/old.txt".into()]),
                window,
                cx
            ));
            let pane = view.chats.pane_mut(&id).unwrap();
            assert_eq!(pane.uploading(), 1);
            assert!(pane.attachments.is_empty());
            assert!(pane.finish_files(&current));
        })
    });
}

#[gpui::test]
fn a_finished_upload_survives_sequence_advancement_and_attaches_once(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let pane = view.chats.panes_mut().next().unwrap();
            let id = pane.target.pane_id.clone();
            let work = pane.begin_files(1);
            pane.session
                .receive(snapshot(cursor(1, 3), vec![]))
                .unwrap();
            let uploaded = Ok(vec!["/remote/current.txt".into()]);
            assert!(view.finish_chat_upload(&id, &work, &uploaded, window, cx));
            assert!(!view.finish_chat_upload(&id, &work, &uploaded, window, cx));
            let pane = view.chats.pane(&id).unwrap();
            assert_eq!(pane.uploading(), 0);
            assert_eq!(pane.attachments, [PathBuf::from("/remote/current.txt")]);
        })
    });
}

#[gpui::test]
fn a_cancelled_file_dialog_does_not_block_or_debit_pending_files(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    let pane = view.read_with(cx, |view, _| {
        view.chats.panes().next().unwrap().target.pane_id.clone()
    });
    let work = view.update(cx, |view, _| {
        view.chats.pane_mut(&pane).unwrap().begin_files(2)
    });
    cx.update(|window, cx| view.update(cx, |view, cx| view.browse_chat_files(&pane, window, cx)));
    cx.simulate_path_prompt_response(|_| None);
    cx.run_until_parked();
    view.update(cx, |view, _| {
        let pane = view.chats.pane_mut(&pane).unwrap();
        assert_eq!(pane.uploading(), 2);
        assert!(pane.finish_files(&work));
        assert_eq!(pane.uploading(), 0);
    });
}

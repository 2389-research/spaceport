// ABOUTME: Verifies app status reads the shared Ask controller without submitting questions.
// ABOUTME: Preserves the status response for apps without an initialized controller.
use super::*;

#[gpui::test]
fn disabled_ask_panel_reports_the_setting_and_restart(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.cx.update(|cx| {
        view.update(cx, |view, _| view.config.sidebar_style.activity_bar = true);
        assert!(matches!(
            ui(call(UiOp::Panel { panel: crate::sidebar::ActivityPanel::Ask }), cx),
            Response::Error { code: ErrorCode::Unsupported, message }
                if message == crate::i18n::fl!("ask-feature-disabled")
        ));
    });
}

#[gpui::test]
fn status_reports_no_ask_controller_without_initializing_one(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let Response::Status(reply) = status(&Caller::default(), cx) else {
            panic!("status response");
        };
        let encoded = serde_json::to_value(reply).unwrap();
        assert!(encoded.get("ask").is_none());
    });
}

#[cfg(feature = "integration-test")]
#[gpui::test]
fn status_exposes_the_actual_shared_ask_facts(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut transcript = sheprd_chat_protocol::Transcript::new("Pi");
        transcript.status = sheprd_chat_protocol::Status::Working;
        transcript.entries.push(sheprd_chat_protocol::Entry {
            id: sheprd_chat_protocol::EntryId(1),
            kind: sheprd_chat_protocol::EntryKind::Agent {
                text: "Partial answer".into(),
            },
        });
        crate::ask::fixture(Some(transcript), cx);
        let Response::Status(reply) = status(&Caller::default(), cx) else {
            panic!("status response");
        };
        let encoded = serde_json::to_value(reply).unwrap();
        assert_eq!(encoded["ask"]["configured"], true);
        assert_eq!(encoded["ask"]["answering"], true);
        assert_eq!(encoded["ask"]["history_present"], true);
        assert_eq!(encoded["ask"]["loading"], false);
        assert_eq!(encoded["ask"]["agent"], "Pi");
        assert_eq!(encoded["ask"]["error"], serde_json::Value::Null);
        assert_eq!(encoded["pid"], std::process::id());
        assert_eq!(encoded["windows"], serde_json::json!([]));
    });
}

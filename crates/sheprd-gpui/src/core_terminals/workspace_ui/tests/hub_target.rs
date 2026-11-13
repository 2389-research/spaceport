// ABOUTME: Checks Core group selection binds the window to one Core identity.
// ABOUTME: Keeps later start requests from replacing a selected group runtime.

use super::*;
use crate::core_terminals::{Error as HubError, Start, State};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

static CALLS: Mutex<Vec<Start>> = Mutex::new(Vec::new());

fn record_join(start: Start) -> Result<sheprd_core_client::CoreBackend, HubError> {
    CALLS.lock().unwrap().push(start);
    Err(HubError::NotInstalled)
}

#[gpui::test]
fn selected_core_group_joins_only_its_existing_runtime(cx: &mut gpui::TestAppContext) {
    let group = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        observation::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"Core group"}
                ]}),
                version: 3,
            },
        );
        let key = crate::browser::CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
        CALLS.lock().unwrap().clear();
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_join_for_test(record_join);
            assert!(view.show_core_group(&key, cx));
            assert!(CALLS.lock().unwrap().is_empty());
            view.core_terminals.hub.connect(Start::IfMissing);
            settle(view);
            assert_eq!(*CALLS.lock().unwrap(), [Start::Join]);

            view.leave_core_group(cx);
            view.core_terminals.hub.connect(Start::IfMissing);
            settle(view);
            assert_eq!(*CALLS.lock().unwrap(), [Start::Join, Start::IfMissing]);
        });
    });
}

fn settle(view: &mut crate::SheprdWindow) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while matches!(view.core_terminals.hub.state(), State::Connecting(_)) {
        assert!(Instant::now() < deadline, "Core join did not finish");
        view.core_terminals.hub.poll();
        std::thread::yield_now();
    }
}

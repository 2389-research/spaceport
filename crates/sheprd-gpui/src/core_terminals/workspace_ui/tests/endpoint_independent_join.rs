// ABOUTME: Verifies that a selected Core group can rejoin its hub independently of legacy endpoints.
// ABOUTME: Uses a private connector so the test cannot dial or start a personal runtime.

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
fn selected_core_group_rejoins_without_a_legacy_local_endpoint(cx: &mut gpui::TestAppContext) {
    let group = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
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
            view.endpoints.clear();
            view.live.snapshot = None;
            assert!(view.endpoints.is_empty());
            view.poll_core_terminals(window, cx);
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let connecting = view.update(cx, |view, _| {
                view.core_terminals.hub.poll();
                matches!(view.core_terminals.hub.state(), State::Connecting(_))
            });
            if !connecting {
                break;
            }
            assert!(Instant::now() < deadline, "Core join did not finish");
            std::thread::yield_now();
        }
        assert_eq!(*CALLS.lock().unwrap(), [Start::Join]);
    });
}

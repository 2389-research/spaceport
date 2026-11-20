// ABOUTME: Exercises chat creation through private control and daemon connections.
// ABOUTME: Requires a real tab acknowledgement before a creation response is sent.
use super::*;
use crate::{
    control::{protocol::Request, socket},
    window::MockPeer,
};
use sheprd_chat_protocol::Agent;
use std::path::PathBuf;

#[gpui::test]
fn new_chat_waits_for_the_exact_created_tab_and_pane(cx: &mut gpui::TestAppContext) {
    let mut peer = MockPeer::advertising(&["tab.create"]);
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            peer.prepare_local_tab(view, cx);
            view.config.features.acp_chats = true;
            view.live.local_daemon_peer = true;
            view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
            view.endpoints[0].live = view.live.clone();
            view.chats.program = crate::chat::Program::Found(PathBuf::from("/fixture/sheprd-acp"));
        })
    });
    let address = socket::test_address();
    let server = socket::Server::bind(&address.path).unwrap();
    let path = address.path.clone();
    let waiting = std::thread::spawn(move || {
        socket::call(
            &path,
            &Request::Chat(call(ChatOp::New {
                agent: Agent::Claude.name().into(),
                pane: None,
            })),
        )
    });
    let incoming = server.next().unwrap();
    let mut state = crate::control::server::State::default();
    cx.cx
        .update(|cx| crate::control::server::serve(incoming, &mut state, cx));
    assert_eq!(
        state.watches.0.len(),
        1,
        "chat new must wait instead of acknowledging queued work"
    );
    let request = peer.request();
    assert_eq!(request["method"], "tab.create");
    cx.update(|window, cx| view.update(cx, |view, cx| {
        view.live.script_response = Some((request["id"].as_str().unwrap().into(), Some(Ok(serde_json::json!({"result":{"type":"tab_created","tab":{"tab_id":"w1:t9"},"root_pane":{"pane_id":"w1:p9"}}})))));
        view.poll_chats(window, cx);
    }));
    cx.cx.update(|cx| state.watches.serve(cx));
    let response = serde_json::to_value(waiting.join().unwrap().unwrap()).unwrap();
    assert_eq!(response["reply"]["kind"], "created");
    assert_eq!(response["reply"]["pane_id"], "w1:p9");
    assert_eq!(response["reply"]["tab"]["runtime"], "herdr");
    assert_eq!(response["reply"]["tab"]["tab_id"], "w1:t9");
    assert!(state.watches.0.is_empty());
}

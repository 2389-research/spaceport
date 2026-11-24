// ABOUTME: Exercises observed idle age through the window-targeted control request.
// ABOUTME: Checks first-seen agents and cross-workspace pane targeting.
use super::*;
use crate::sidebar::layout_tests::{fixture_window, snapshot};
use spaceport_client::protocol::{AgentStatus, ClientShellPane};
use std::sync::Arc;

#[gpui::test]
fn idle_read_uses_only_the_agent_in_the_callers_workspace(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut snap = snapshot(2);
        for agent in &snap.agents {
            snap.panes.push(ClientShellPane {
                pane_id: agent.pane_id.clone(),
                workspace_id: agent.workspace_id.clone(),
                tab_id: agent.tab_id.clone(),
                label: None,
                cwd: None,
                foreground_cwd: None,
                focused: false,
                right_click_passthrough: false,
            });
        }
        view.live.apply(
            spaceport_client::ClientEvent::Snapshot(Arc::new(snap.clone())),
            &crate::state::labels(),
        );
        snap.revision += 1;
        snap.agents[0].agent_status = AgentStatus::Idle;
        snap.agents[0].state_change_seq += 1;
        view.live.apply(
            spaceport_client::ClientEvent::Snapshot(Arc::new(snap)),
            &crate::state::labels(),
        );
        view
    });
    cx.cx.update(|cx| {
        let mut request = call(UiOp::AgentIdle { pane: None });
        request.caller.workspace_id = Some("w0".into());
        request.caller.pane_id = Some("p0".into());
        assert!(matches!(ui(request.clone(), cx), Response::App {
            reply: Reply::AgentIdle { pane, age: Some(age) }
        } if pane == "p0" && !age.lower_bound));
        request.op = UiOp::AgentIdle {
            pane: Some("p1".into()),
        };
        assert_eq!(code(&ui(request, cx)), Some(ErrorCode::NotFound));
    });
}

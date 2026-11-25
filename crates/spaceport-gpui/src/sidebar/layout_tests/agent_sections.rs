// ABOUTME: Checks the desktop Agents panel headings and keyboard ordering together.
// ABOUTME: Exercises timed groups and daemon-local sequence ordering for undated rows.
use super::*;
use crate::preferences::AgentSort;

#[gpui::test]
fn priority_sections_show_attention_then_timed_and_undated_idle(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            let mut listing = snapshot(1);
            let base = &listing.agents[0];
            listing.agents = [
                ("oldest", AgentStatus::Idle, 11),
                ("middle", AgentStatus::Idle, 12),
                ("recent", AgentStatus::Done, 13),
                ("early", AgentStatus::Idle, 2),
                ("work", AgentStatus::Working, 5),
                ("wait", AgentStatus::Blocked, 3),
                ("late", AgentStatus::Done, 9),
            ]
            .map(|(id, status, seq)| ClientShellAgent {
                pane_id: id.into(),
                agent_status: status,
                state_change_seq: seq,
                ..base.clone()
            })
            .into();
            let now = spaceport_client::agent_idle::now_ms();
            let hour = 3_600_000;
            let mut previous = listing.clone();
            previous.agents.truncate(3);
            for agent in &mut previous.agents {
                agent.agent_status = AgentStatus::Working;
                agent.state_change_seq = 1;
            }
            previous.revision = 1;
            view.live.idle_tracker.observe(&previous, now - 7 * hour);
            for (idx, age) in [6 * hour, 2 * hour, hour / 4].into_iter().enumerate() {
                previous.agents[idx] = listing.agents[idx].clone();
                previous.revision += 1;
                view.live.idle_tracker.observe(&previous, now - age);
            }
            listing.revision = previous.revision + 1;
            view.agent_sort = AgentSort::Priority;
            view.live.apply(
                spaceport_client::ClientEvent::Snapshot(Arc::new(listing)),
                crate::state::describe,
                crate::state::localize,
            );
            view
        });
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    cx.simulate_resize(size(px(900.), px(800.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    for selector in [
        "header-agents-waiting",
        "header-agents-working",
        "header-agents-recent",
        "header-agents-hours",
        "header-agents-long-idle",
        "header-agents-idle",
    ] {
        assert!(cx.debug_bounds(selector).is_some(), "missing {selector}");
    }
    cx.update(|_, cx| {
        let panes: Vec<_> = fixture
            .read(cx)
            .0
            .read(cx)
            .sidebar_agents()
            .into_iter()
            .map(|(_, pane)| pane)
            .collect();
        assert_eq!(
            panes,
            [
                "wait", "work", "recent", "middle", "oldest", "late", "early"
            ]
        );
    });
}

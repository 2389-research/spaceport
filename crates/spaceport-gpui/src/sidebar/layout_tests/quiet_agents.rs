// ABOUTME: Exercises Quiet through the Priority list and its navigation shortcuts.
// ABOUTME: Confirms that new activity surfaces a previously folded idle agent.
use super::*;

#[gpui::test]
fn quiet_agents_start_folded_and_resurface_when_working(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = snapshot(1);
        listing.agents.truncate(1);
        listing.agents[0].agent_status = AgentStatus::Idle;
        let yesterday = spaceport_client::agent_idle::now_ms() - 24 * 3_600_000;
        view.live.idle_tracker.observe(&listing, yesterday);
        view.live.snapshot = Some(Arc::new(listing));
        view.live.status = crate::state::ConnectionStatus::Connected;
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    for expanded in [false, true] {
        let heading = cx
            .debug_bounds("header-agents-quiet")
            .expect("Quiet heading");
        view.read_with(cx, |view, _| {
            assert_eq!(view.sidebar_agents().contains(&(0, "p0")), expanded);
        });
        assert_eq!(cx.debug_bounds("row-agent-p0").is_some(), expanded);
        cx.simulate_click(heading.center(), Default::default());
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
    }
    view.update(cx, |view, cx| {
        let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        listing.revision += 1;
        listing.agents[0].agent_status = AgentStatus::Working;
        view.live
            .idle_tracker
            .observe(listing, spaceport_client::agent_idle::now_ms());
        cx.notify();
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("header-agents-working").is_some());
    assert!(cx.debug_bounds("header-agents-quiet").is_none());
    view.read_with(cx, |view, _| {
        assert!(view.sidebar_agents().contains(&(0, "p0")))
    });
}

#[test]
fn quiet_respects_each_threshold_and_live_status() {
    use super::super::AgentSection;
    use spaceport_settings::QuietAfter;

    let mut live = LiveState {
        status: crate::state::ConnectionStatus::Connected,
        ..LiveState::default()
    };
    let mut listing = snapshot(1);
    listing.agents[0].agent_status = AgentStatus::Idle;
    let start = 1_000;
    live.idle_tracker.observe(&listing, start);
    for (after, hours) in [
        (QuietAfter::OneDay, 24),
        (QuietAfter::TwoDays, 48),
        (QuietAfter::OneWeek, 168),
    ] {
        let boundary = start + hours * 3_600_000;
        let agent = &listing.agents[0];
        assert_ne!(
            AgentSection::of(&live, agent, boundary - 1, after),
            AgentSection::Quiet
        );
        assert_eq!(
            AgentSection::of(&live, agent, boundary, after),
            AgentSection::Quiet
        );
        assert_eq!(
            AgentSection::of(&live, agent, start, after),
            AgentSection::Idle(spaceport_client::agent_idle::IdleAgeGroup::Unplaced)
        );
    }
    let agent = &mut listing.agents[0];
    for (status, expected) in [
        (AgentStatus::Done, AgentSection::Done),
        (AgentStatus::Blocked, AgentSection::Waiting),
        (AgentStatus::Working, AgentSection::Working),
    ] {
        agent.agent_status = status;
        assert_eq!(
            AgentSection::of(&live, agent, u64::MAX, QuietAfter::OneDay),
            expected
        );
    }
    let disconnected = LiveState::default();
    agent.agent_status = AgentStatus::Idle;
    assert_eq!(
        AgentSection::of(&disconnected, agent, u64::MAX, QuietAfter::OneDay),
        AgentSection::Offline
    );
}

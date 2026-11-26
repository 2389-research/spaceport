// ABOUTME: Clicks host names and status headings in the desktop sidebar.
// ABOUTME: Checks that folding affects only the chosen group and its shortcuts.
use super::*;

#[gpui::test]
fn clicking_a_host_name_folds_only_its_workspaces(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(snapshot(1)));
        let mut remote = layout_host("ssh:test", "Remote", ssh("unused"));
        remote.live_mut().snapshot = Some(Arc::new(snapshot(1)));
        view.endpoints.push(remote);
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));

    for expanded in [false, true] {
        let name = cx.debug_bounds("host-label-ssh:test").unwrap();
        assert!(
            cx.debug_bounds("spaces-scroll")
                .unwrap()
                .contains(&name.center())
        );
        cx.simulate_click(name.center(), Default::default());
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        assert_eq!(cx.debug_bounds("workspace-ssh:test-w0").is_some(), expanded);
        assert!(cx.debug_bounds("workspace-local-w0").is_some());
        view.read_with(cx, |view, _| assert_eq!(view.selected_endpoint, 0));
    }
}

#[gpui::test]
fn clicking_a_status_heading_folds_its_rows_and_shortcuts(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = snapshot(2);
        listing.agents[1].agent_status = AgentStatus::Idle;
        view.live.snapshot = Some(Arc::new(listing));
        view.live.status = crate::state::ConnectionStatus::Connected;
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    for expanded in [false, true] {
        let heading = cx.debug_bounds("header-agents-working").unwrap();
        cx.simulate_click(heading.center(), Default::default());
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        assert_eq!(cx.debug_bounds("row-agent-p0").is_some(), expanded);
        assert!(cx.debug_bounds("row-agent-p1").is_some());
        view.read_with(cx, |view, _| {
            assert_eq!(view.sidebar_agents().contains(&(0, "p0")), expanded);
        });
    }
}

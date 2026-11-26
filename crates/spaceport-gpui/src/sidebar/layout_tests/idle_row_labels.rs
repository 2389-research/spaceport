// ABOUTME: Checks which status words an Idle heading already supplies.
// ABOUTME: Exercises timed and undated agents in built-in and configured rows.
use super::*;
use crate::preferences::AgentSort;

#[gpui::test]
fn idle_headings_supply_the_status_word_for_their_rows(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let now = spaceport_client::agent_idle::now_ms();
        let mut listing = snapshot(2);
        listing.agents[0].terminal_title_stripped = Some("Timed".into());
        listing.agents[0].agent_status = AgentStatus::Working;
        view.live.idle_tracker.observe(&listing, now - 1_000_000);
        listing.revision += 1;
        listing.agents[0].agent_status = AgentStatus::Idle;
        listing.agents[0].state_change_seq += 1;
        view.live.idle_tracker.observe(&listing, now - 910_000);
        listing.agents[1].agent_status = AgentStatus::Idle;
        listing.revision += 1;
        view.live.snapshot = Some(Arc::new(listing));
        view.live.status = crate::state::ConnectionStatus::Connected;
        view
    });
    cx.simulate_resize(size(px(800.), px(850.)));
    for configured in [false, true] {
        for sort in [AgentSort::Priority, AgentSort::Grouped] {
            view.update(cx, |view, cx| {
                view.agent_sort = sort;
                view.config.usage.inline = configured;
                view.config.sidebar_layout.agents =
                    toml::from_str(r#"rows = [["agent", "state_text"]]"#).unwrap();
                cx.notify();
            });
            cx.update(|window, cx| {
                cx.default_global::<TextProbes>().0.clear();
                full_draw(window, cx).clear(cx);
                let text = &cx.global::<TextProbes>().0;
                let words = text
                    .keys()
                    .map(|t| t.replace(['\u{2068}', '\u{2069}'], ""))
                    .collect::<Vec<_>>();
                assert_eq!(
                    words.iter().any(|t| t == "15m"),
                    sort == AgentSort::Priority,
                    "{configured}: {words:?}"
                );
                assert_eq!(text.contains_key("idle"), sort == AgentSort::Grouped);
                assert!(!words.iter().any(|t| t.starts_with("idle · ")));
            });
        }
    }
}

// ABOUTME: Checks the focused agent's outline in the expanded sidebar and collapsed rail.
// ABOUTME: Reads the painted frame as focus moves between two agent panes.
use super::*;

#[gpui::test]
fn only_the_focused_agent_has_a_row_wide_outline(cx: &mut TestAppContext) {
    let (view, cx) = rail_window(cx, 1);
    cx.simulate_resize(size(px(800.), px(1000.)));
    for expanded in [true, false] {
        for mode in LayoutMode::ALL {
            for focused in [0, 1] {
                let accent = view.update(cx, |view, cx| {
                    view.live.status = crate::state::ConnectionStatus::Connected;
                    view.sidebar_visible = expanded;
                    view.sidebar_width = Some(160.);
                    view.config.layout.mode = mode;
                    let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
                    for (index, agent) in snapshot.agents.iter_mut().enumerate() {
                        agent.focused = index == focused;
                    }
                    cx.notify();
                    Hsla::from(rgb(view.theme.ink(view.theme.primary())))
                });
                cx.update(|window, cx| full_draw(window, cx).clear(cx));
                let rows = if expanded {
                    ["row-agent-p0", "row-agent-p1"]
                } else {
                    ["rail-agent-local-p0", "rail-agent-local-p1"]
                };
                for (index, selector) in rows.into_iter().enumerate() {
                    let row = cx.debug_bounds(selector).unwrap_or_else(|| {
                        panic!("{mode:?} {selector} missing, focused {focused}")
                    });
                    let outlined = cx.update(|window, _| {
                        let scale = window.scale_factor();
                        window.painted_quads().into_iter().any(|quad| {
                            same_box(quad.bounds, row.scale(scale), px(2.).scale(scale))
                                && quad.border_widths.top.0 > 0.
                                && quad.border_widths.bottom.0 > 0.
                                && matches_color(quad.border_color, accent)
                        })
                    });
                    assert_eq!(outlined, index == focused, "{mode:?} {selector}");
                }
            }
        }
    }
}

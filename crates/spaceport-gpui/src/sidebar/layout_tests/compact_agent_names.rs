// ABOUTME: Checks the app name as a desktop sidebar narrows and widens.
// ABOUTME: Covers native and configured rows while keeping the pane identifiable.
use super::*;

#[gpui::test]
fn narrow_rows_replace_the_app_name_with_its_icon(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = snapshot(1);
        listing.agents.truncate(1);
        listing.agents[0].title = Some("review".into());
        view.live.status = crate::state::ConnectionStatus::Connected;
        view.live.snapshot = Some(Arc::new(listing));
        view
    });
    cx.simulate_resize(size(px(800.), px(700.)));
    cx.run_until_parked();
    for mode in crate::config::LayoutMode::ALL {
        for configured in [false, true] {
            for (width, named) in [(320., true), (160., false), (320., true)] {
                view.update(cx, |view, cx| {
                    view.config.layout.mode = mode;
                    view.sidebar_width = Some(width);
                    view.config.sidebar_layout.agents = if configured {
                        toml::from_str("rows = [[\"state_icon\", \"agent\", \"pane\"]]").unwrap()
                    } else {
                        Default::default()
                    };
                    cx.notify();
                });
                cx.update(|window, cx| {
                    cx.default_global::<TextProbes>().0.clear();
                    full_draw(window, cx).clear(cx);
                    let text = &cx.global::<TextProbes>().0;
                    assert!(text.contains_key("review"), "{mode:?} {configured} {width}");
                    assert_eq!(
                        text.contains_key("Claude Code"),
                        named,
                        "{mode:?} {configured} {width}"
                    );
                });
                if matches!(
                    mode,
                    crate::config::LayoutMode::Classic { .. } | crate::config::LayoutMode::Devices
                ) {
                    assert!(cx.debug_bounds("agent-icon-agent-p0").is_some());
                }
            }
        }
    }
}

#[gpui::test]
fn agent_rows_show_the_terminal_directory_and_host_across_sidebar_widths(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = snapshot(1);
        listing.agents.truncate(1);
        listing.agents[0].pane_id = "wAB:pX".into();
        listing.agents[0].tab_id = "t1".into();
        listing.agents[0].terminal_title_stripped = Some("Review queue".into());
        listing.panes.push(ClientShellPane {
            pane_id: "wAB:pX".into(),
            workspace_id: "w0".into(),
            tab_id: "t1".into(),
            label: None,
            cwd: Some("/src/project".into()),
            foreground_cwd: Some("/src/project/sidebar".into()),
            focused: false,
            right_click_passthrough: false,
        });
        view.endpoints[0].label = "Relay".into();
        view.config.show_agents_header = false;
        view.live.status = crate::state::ConnectionStatus::Connected;
        view.live.snapshot = Some(Arc::new(listing));
        view
    });
    cx.simulate_resize(size(px(800.), px(700.)));
    for mode in crate::config::LayoutMode::ALL {
        for width in [232., 160., 320.] {
            view.update(cx, |view, cx| {
                view.config.layout.mode = mode;
                view.sidebar_width = Some(width);
                cx.notify();
            });
            cx.update(|window, cx| {
                cx.default_global::<TextProbes>().0.clear();
                full_draw(window, cx).clear(cx);
                let text = &cx.global::<TextProbes>().0;
                for label in ["#2", "Review queue", "sidebar", "Relay"] {
                    assert!(
                        text.contains_key(label),
                        "{mode:?} {width}: missing {label}"
                    );
                }
                assert!(!text.contains_key("wAB:pX"));
                assert!(!text.contains_key("#X"));
                let number = text["#2"].0;
                let directory = text["sidebar"].0;
                let host = text["Relay"].0;
                let title = text["Review queue"].0;
                assert_eq!(number.top(), directory.top(), "{mode:?} {width}");
                assert_eq!(directory.top(), host.top(), "{mode:?} {width}");
                assert!(number.right() <= directory.left() && directory.right() <= host.left());
                assert!(
                    title.bottom() <= host.top(),
                    "{mode:?} {width}: title above metadata"
                );
                assert!(title.size.width > directory.size.width);
                assert!(!text.contains_key("Claude Code"), "{mode:?} {width}");
            });
            let row = cx.debug_bounds("row-agent-wAB:pX").unwrap();
            let detail = cx.debug_bounds("detail-agent-wAB:pX").unwrap();
            assert!(
                detail.top() >= row.top() && detail.bottom() <= row.bottom(),
                "{mode:?} {width}: {detail:?} outside {row:?}"
            );
            let selector = match mode {
                crate::config::LayoutMode::Superset => "icon-agent-wAB:pX",
                _ => "agent-icon-agent-wAB:pX",
            };
            let icon = cx.debug_bounds(selector).unwrap();
            cx.update(|_, cx| {
                let number = cx.global::<TextProbes>().0["#2"].0;
                assert!(
                    icon.center().y >= number.top() && icon.center().y < number.bottom(),
                    "{mode:?} {width}: app icon beside metadata"
                );
                assert!(icon.right() <= number.left(), "{mode:?} {width}");
            });
        }
    }
}

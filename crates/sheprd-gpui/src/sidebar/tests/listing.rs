// ABOUTME: Compares the rows the sidebar paints with the rows its shortcuts step through.
// ABOUTME: Reads painted rows from the real expanded list and rail by their debug selectors.
//! What the sidebar lists. A row is named `host/id`, where `id` is the
//! workspace's or the agent pane's. Painted rows come from a full frame of
//! the real list, top to bottom; stepped rows from `sidebar_workspaces` and
//! `sidebar_agents`, which the previous, next, and numbered shortcuts use.

use crate::{
    SheprdWindow,
    endpoint::Endpoint,
    sidebar::layout_tests::{SidebarFixture, fixture_window, full_draw, snapshot},
};
use gpui::{Entity, TestAppContext, VisualTestContext, prelude::*, px, size};
use sheprd_client::ConnectTarget;
use std::sync::Arc;

mod folds;
mod rule;

const REMOTE: &str = "ssh:a";

/// A host with the fixture's six workspaces, one repository group among
/// them, and its two agents.
fn remote(id: &str, enabled: bool) -> Endpoint {
    let mut endpoint = Endpoint::new(
        id.into(),
        id.into(),
        ConnectTarget::Ssh {
            target: "example.invalid".into(),
            session: "default".into(),
        },
        enabled,
    );
    endpoint.live.snapshot = Some(Arc::new(snapshot(6)));
    endpoint.never_connect();
    endpoint
}

/// Local, selected, and whatever hosts `setup` adds, in a window tall
/// enough that neither list scrolls.
fn window(
    cx: &mut TestAppContext,
    setup: impl FnOnce(&mut SheprdWindow) + 'static,
) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    let (fixture, cx) = cx.add_window_view(move |window, cx| {
        let view = cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            view.live.snapshot = Some(Arc::new(snapshot(6)));
            view.endpoints[0].live = view.live.clone();
            setup(&mut view);
            view
        });
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    cx.simulate_resize(size(px(900.), px(4000.)));
    cx.run_until_parked();
    (view, cx)
}

/// Local and one remote host.
fn two_hosts(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    window(cx, |view| view.endpoints.push(remote(REMOTE, true)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Expanded,
    Rail,
}

/// What one mode lists: the rows it paints and the rows a shortcut steps to.
#[derive(Debug)]
struct Listed {
    painted_workspaces: Vec<String>,
    stepped_workspaces: Vec<String>,
    painted_agents: Vec<String>,
    stepped_agents: Vec<String>,
}

impl Listed {
    /// Each kind of row whose stepping differs from its painting, with both lists.
    fn drift(&self) -> Vec<String> {
        [
            (
                "workspaces",
                &self.painted_workspaces,
                &self.stepped_workspaces,
            ),
            ("agents", &self.painted_agents, &self.stepped_agents),
        ]
        .into_iter()
        .filter(|(_, painted, stepped)| painted != stepped)
        .map(|(kind, painted, stepped)| format!("{kind}: painted {painted:?}, stepped {stepped:?}"))
        .collect()
    }
}

/// Which of `candidates` the last frame painted under `prefix`, top to bottom.
fn painted(
    cx: &mut VisualTestContext,
    prefix: &str,
    candidates: &[(String, String)],
) -> Vec<String> {
    let mut rows: Vec<_> = candidates
        .iter()
        .filter_map(|(host, id)| {
            // `debug_bounds` wants a static selector; a test may leak one.
            let selector: &'static str = format!("{prefix}-{host}-{id}").leak();
            let bounds = cx.debug_bounds(selector)?;
            Some((f32::from(bounds.top()), format!("{host}/{id}")))
        })
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    rows.into_iter().map(|(_, row)| row).collect()
}

/// Shows the sidebar in `mode` and reads what it lists.
fn listed(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext, mode: Mode) -> Listed {
    view.update(cx, |view, cx| {
        view.sidebar_visible = mode == Mode::Expanded;
        cx.notify();
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    // Every row any host could list, whatever is folded or filtered.
    let (workspaces, agents) = view.read_with(cx, |view, _| {
        let mut workspaces = Vec::new();
        let mut agents = Vec::new();
        for (_, endpoint, live) in view.hosts() {
            let Some(snapshot) = live.snapshot.as_deref() else {
                continue;
            };
            workspaces.extend(
                snapshot
                    .workspaces
                    .iter()
                    .map(|workspace| (endpoint.id.clone(), workspace.workspace_id.clone())),
            );
            agents.extend(
                snapshot
                    .agents
                    .iter()
                    .map(|agent| (endpoint.id.clone(), agent.pane_id.clone())),
            );
        }
        (workspaces, agents)
    });
    let (workspace_prefix, agent_prefix) = match mode {
        Mode::Expanded => ("workspace", "agent"),
        Mode::Rail => ("rail-workspace", "rail-agent"),
    };
    let painted_workspaces = painted(cx, workspace_prefix, &workspaces);
    let painted_agents = painted(cx, agent_prefix, &agents);
    let (stepped_workspaces, stepped_agents) = view.read_with(cx, |view, _| {
        let name = |rows: Vec<crate::sidebar::order::Row<'_>>| -> Vec<String> {
            rows.into_iter()
                .map(|(host, id)| format!("{}/{id}", view.endpoints[host].id))
                .collect()
        };
        (name(view.sidebar_workspaces()), name(view.sidebar_agents()))
    });
    Listed {
        painted_workspaces,
        stepped_workspaces,
        painted_agents,
        stepped_agents,
    }
}

/// Asserts that, in `mode`, stepping visits exactly the painted rows in order.
#[track_caller]
fn assert_steps_what_it_paints(
    view: &Entity<SheprdWindow>,
    cx: &mut VisualTestContext,
    mode: Mode,
) -> Listed {
    let listed = listed(view, cx, mode);
    let drift = listed.drift();
    assert!(drift.is_empty(), "{mode:?} drifted: {}", drift.join("; "));
    listed
}

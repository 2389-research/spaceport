// ABOUTME: Checks that sidebar shortcuts step through exactly the rows the current mode paints.
// ABOUTME: Covers disabled, tag-folded and collapsed hosts, and agents the configured rows leave out.
use super::*;
use crate::device_tags::Tags;

/// Local and a remote host that is disabled but still holds a snapshot.
fn disabled_host(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    window(cx, |view| view.endpoints.push(remote(REMOTE, false)))
}

/// Local and a remote host in the folded "work" section. With `tags` the
/// list is grouped; without, the fold is left over and there are no sections.
fn tag_folded_host(
    cx: &mut TestAppContext,
    tags: bool,
) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    window(cx, move |view| {
        view.endpoints.push(remote(REMOTE, true));
        if tags {
            view.device_tags = Arc::new(Tags::tagged(&[(REMOTE, &["work"])]));
        }
        view.device_tag_folds.insert(Some("work".to_owned()));
        if !tags {
            view.device_tag_folds.insert(None);
        }
        assert_eq!(view.device_tag_folded(REMOTE), tags);
    })
}

/// Local and a remote host folded by its own arrow.
fn collapsed_host(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    window(cx, |view| {
        let mut host = remote(REMOTE, true);
        host.collapsed = true;
        view.endpoints.push(host);
    })
}

/// Configured agent rows, and a local agent they leave out: its workspace
/// has gone, so it has no rows.
fn unconfigured_agent(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    window(cx, |view| {
        view.endpoints.push(remote(REMOTE, true));
        view.config.usage.inline = true;
        view.config.sidebar_layout.agents.row_gap = 1;
        let mut listing = snapshot(6);
        listing.agents[1].workspace_id = "gone".into();
        view.live.snapshot = Some(Arc::new(listing));
        view.endpoints[0].live = view.live.clone();
    })
}

#[gpui::test]
fn two_hosts_expanded(cx: &mut TestAppContext) {
    let (view, cx) = two_hosts(cx);
    let listed = assert_steps_what_it_paints(&view, cx, Mode::Expanded);
    assert_eq!(listed.painted_workspaces.len(), 12);
    assert_eq!(listed.painted_agents.len(), 4);
}

#[gpui::test]
fn two_hosts_rail(cx: &mut TestAppContext) {
    let (view, cx) = two_hosts(cx);
    let listed = assert_steps_what_it_paints(&view, cx, Mode::Rail);
    assert_eq!(listed.painted_workspaces.len(), 12);
    assert_eq!(listed.painted_agents.len(), 4);
}

#[gpui::test]
fn disabled_host_expanded(cx: &mut TestAppContext) {
    let (view, cx) = disabled_host(cx);
    assert_steps_what_it_paints(&view, cx, Mode::Expanded);
}

#[gpui::test]
fn disabled_host_rail(cx: &mut TestAppContext) {
    let (view, cx) = disabled_host(cx);
    assert_steps_what_it_paints(&view, cx, Mode::Rail);
}

#[gpui::test]
fn tag_fold_without_sections_expanded(cx: &mut TestAppContext) {
    let (view, cx) = tag_folded_host(cx, false);
    assert_steps_what_it_paints(&view, cx, Mode::Expanded);
}

#[gpui::test]
fn tag_fold_without_sections_rail(cx: &mut TestAppContext) {
    let (view, cx) = tag_folded_host(cx, false);
    assert_steps_what_it_paints(&view, cx, Mode::Rail);
}

#[gpui::test]
fn tag_folded_section_expanded(cx: &mut TestAppContext) {
    let (view, cx) = tag_folded_host(cx, true);
    let listed = assert_steps_what_it_paints(&view, cx, Mode::Expanded);
    // The folded section's host is hidden; Local, untagged, is not.
    assert_eq!(listed.painted_workspaces.len(), 6);
    assert!(
        listed
            .painted_workspaces
            .iter()
            .all(|row| row.starts_with("local/"))
    );
}

#[gpui::test]
fn tag_folded_section_rail(cx: &mut TestAppContext) {
    let (view, cx) = tag_folded_host(cx, true);
    assert_steps_what_it_paints(&view, cx, Mode::Rail);
}

#[gpui::test]
fn collapsed_host_expanded(cx: &mut TestAppContext) {
    let (view, cx) = collapsed_host(cx);
    assert_steps_what_it_paints(&view, cx, Mode::Expanded);
}

#[gpui::test]
fn collapsed_host_rail(cx: &mut TestAppContext) {
    let (view, cx) = collapsed_host(cx);
    assert_steps_what_it_paints(&view, cx, Mode::Rail);
}

#[gpui::test]
fn unconfigured_agent_expanded(cx: &mut TestAppContext) {
    let (view, cx) = unconfigured_agent(cx);
    assert_steps_what_it_paints(&view, cx, Mode::Expanded);
}

#[gpui::test]
fn unconfigured_agent_rail(cx: &mut TestAppContext) {
    let (view, cx) = unconfigured_agent(cx);
    assert_steps_what_it_paints(&view, cx, Mode::Rail);
}

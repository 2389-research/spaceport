// ABOUTME: Observes the counters owned by real primary and parked client handles.
// ABOUTME: Exercises connection swaps and missing handles through the profile group report.
use super::{draw, parked, terminals_side_by_side, window};
use crate::{
    SheprdWindow, browser::GroupId, controls::Command, endpoint::Endpoint,
    sidebar::layout_tests::snapshot, window::MockPeer, working_profile::report::Group,
};
use gpui::{App, Entity, TestAppContext, VisualTestContext};
use sheprd_client::ConnectTarget;

fn split_with_peers<'a>(
    cx: &'a mut TestAppContext,
    primary: &MockPeer,
    secondary: &MockPeer,
) -> (Entity<SheprdWindow>, &'a mut VisualTestContext, GroupId) {
    let (view, cx) = window(cx);
    draw(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.command(Command::SplitEditor, window, cx)
        });
    });
    draw(cx);
    let left = view.update(cx, |view, _| {
        // Leave a different endpoint at index zero.
        view.endpoints.insert(
            0,
            Endpoint::new(
                "unused".into(),
                "Unused".into(),
                ConnectTarget::Socket("/unused-profile-test.sock".into()),
                false,
            ),
        );
        view.selected_endpoint = 1;
        let slots = view.group_slots();
        let [left, right] = [slots[0].id, slots[1].id];
        terminals_side_by_side(view, left, right);
        let endpoint = &mut view.endpoints[view.selected_endpoint];
        endpoint.connection.handle = Some(primary.client.handle.clone());
        let target = endpoint.connection.target.clone();
        let mut on_t1 = snapshot(40);
        on_t1.focused_workspace_id = Some("w0".into());
        on_t1.focused_tab_id = Some("t1".into());
        let key = view.browser_key().unwrap();
        view.group_terminals
            .parked
            .push(parked(left, key, target, secondary, on_t1));
        left
    });
    (view, cx, left)
}

fn observed(view: &SheprdWindow, cx: &App) -> [Group; 2] {
    let [left, right]: [Group; 2] = view.working_profile_groups(cx).try_into().unwrap();
    assert_eq!(left.tab.as_deref(), Some("t1"));
    assert_eq!(right.tab.as_deref(), Some("t0"));
    assert!(left.shown_terminal && right.shown_terminal);
    assert!(left.connected && right.connected);
    [left, right]
}

#[gpui::test]
fn patch_stats_follow_each_group_through_a_bridge_swap(cx: &mut TestAppContext) {
    let primary = MockPeer::new();
    let secondary = MockPeer::new();
    let primary_stats = primary.client.handle.patch_stats().unwrap();
    let secondary_stats = secondary.client.handle.patch_stats().unwrap();
    assert_ne!(primary_stats.connection_id, secondary_stats.connection_id);

    let (view, cx, left) = split_with_peers(cx, &primary, &secondary);
    let before = view.read_with(cx, observed);
    assert!(!before[0].primary && before[1].primary);
    cx.update(|window, cx| view.update(cx, |view, cx| view.activate_group(left, window, cx)));
    let after = view.read_with(cx, observed);
    assert!(after[0].primary && !after[1].primary);

    for groups in [before, after] {
        assert_eq!(
            groups.map(|group| group.patch_stats),
            [Some(secondary_stats), Some(primary_stats)],
            "the observer must report the bridges that show t1 and t0"
        );
    }
}

#[gpui::test]
fn missing_group_handles_omit_patch_stats(cx: &mut TestAppContext) {
    let primary = MockPeer::new();
    let secondary = MockPeer::new();
    let (view, cx, _) = split_with_peers(cx, &primary, &secondary);

    let without_handles = view.update(cx, |view, cx| {
        let selected = view.selected_endpoint;
        let primary = view.endpoints[selected].connection.handle.take().unwrap();
        let [_, without_primary] = observed(view, cx);
        view.endpoints[selected].connection.handle = Some(primary);
        let parked = view.group_terminals.parked[0]
            .connection
            .handle
            .take()
            .unwrap();
        let [without_parked, _] = observed(view, cx);
        view.group_terminals.parked[0].connection.handle = Some(parked);
        [without_primary, without_parked]
    });
    for group in without_handles {
        assert_eq!(group.patch_stats, None);
        let encoded = serde_json::to_value(group).unwrap();
        assert!(encoded.get("patch_stats").is_none());
        let decoded: Group = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded.patch_stats, None);
    }
}

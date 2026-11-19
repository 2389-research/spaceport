// ABOUTME: Keeps close-batch ownership stable when default tab labels renumber.
// ABOUTME: Retains refusal for custom renames and changes in label ownership.
use super::*;
use crate::browser::Pick;

fn check_label_change(cx: &mut TestAppContext, custom: bool, changed_custom: bool, renamed: bool) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let peer = crate::window::MockPeer::advertising(&["tab.close"]);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            peer.prepare_local_tab(view, cx);
            let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            for (id, label) in [("first", "1"), ("second", "2")] {
                let mut tab = snapshot.tabs[0].clone();
                tab.tab_id = id.into();
                tab.label = label.into();
                tab.custom_label = custom;
                snapshot.tabs.push(tab);
            }
            *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
            let batch = super::super::batch::Batch::capture(
                view,
                vec![Pick::Herdr("first".into()), Pick::Herdr("second".into())],
                cx,
            )
            .unwrap();
            let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            snapshot.tabs.retain(|tab| tab.tab_id != "first");
            let second = snapshot
                .tabs
                .iter_mut()
                .find(|tab| tab.tab_id == "second")
                .unwrap();
            if renamed {
                second.label = "1".into();
            }
            second.custom_label = changed_custom;
            *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
            let result = batch.validate(view, &batch.items[1..], cx);
            if custom != changed_custom || (custom && renamed) {
                assert!(matches!(result, Err(Error::StaleCloseTarget)));
            } else {
                assert!(
                    matches!(result, Ok(super::super::batch::Inspection::Current)),
                    "default renumbering must not strand an accepted batch"
                );
            }
        })
    });
}

#[gpui::test]
fn default_tab_renumbering_preserves_the_remaining_close_target(cx: &mut TestAppContext) {
    check_label_change(cx, false, false, true);
}

#[gpui::test]
fn custom_tab_rename_refuses_the_remaining_close_target(cx: &mut TestAppContext) {
    check_label_change(cx, true, true, true);
}

#[gpui::test]
fn changing_label_ownership_refuses_even_when_the_label_stays_equal(cx: &mut TestAppContext) {
    check_label_change(cx, false, true, false);
    check_label_change(cx, true, false, false);
}

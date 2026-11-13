// ABOUTME: Exercises Core group tab tear-off without a legacy daemon endpoint.
// ABOUTME: Uses an exact private document fixture and records the dragged tab.

use super::*;

#[gpui::test]
fn a_core_tab_tears_off_without_a_legacy_endpoint(cx: &mut TestAppContext) {
    let group = uuid::Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_core_window);
    let (key, tab, other) = cx.update(|window, cx| {
        crate::core_terminals::set_document_for_test_as(
            cx,
            "hub-a",
            sheprd_core_client::wire::Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": [
                    {"id": group, "label": "Core group"}
                ]}),
                version: 3,
            },
        );
        let key = crate::browser::CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
        let tab = Store::update(cx, |store| store.open_core(&key, None)).unwrap();
        let other = Store::update(cx, |store| store.open_core(&key, None)).unwrap();
        view.update(cx, |view, cx| {
            assert!(view.show_core_group(&key, cx));
            view.endpoints.clear();
            view.live.snapshot = None;
            view.show_browser_tab(tab, window, cx);
        });
        (key, tab, other)
    });
    cx.simulate_resize(size(px(1600.), px(600.)));
    for _ in 0..2 {
        draw(cx);
    }
    carry_to(cx, &format!("browser-tab-{tab}"), OUTSIDE);
    drop_at(cx, OUTSIDE);

    let torn = torn(cx);
    assert_eq!(torn.len(), 1);
    assert_eq!(torn[0].workspace, key.parts());
    assert_eq!(torn[0].pick, Pick::Page(tab));
    assert_eq!(torn[0].others, [Pick::Page(other)]);
    assert!(matches!(&torn[0].route, TornRoute::Core { key: carried, .. } if carried == &key));
    view.read_with(cx, |view, _| {
        assert!(view.endpoints.is_empty());
        assert!(view.browser.hidden.hides(&key.parts(), &Pick::Page(tab)));
    });

    let target_window = cx.add_window(crate::sidebar::layout_tests::fixture_core_window);
    target_window
        .update(cx, |view, window, cx| {
            assert!(view.show_core_group(&key, cx));
            view.endpoints.clear();
            view.live.snapshot = None;
            assert!(view.arrive(torn[0].clone(), window, cx));
            assert_eq!(
                view.group_pick(view.group_slots()[0].id),
                Some(Pick::Page(tab))
            );
            assert!(view.browser.hidden.hides(&key.parts(), &Pick::Page(other)));
        })
        .unwrap();
    target_window
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(!view.browser.hidden.hides(&key.parts(), &Pick::Page(tab)));
    });
}

#[gpui::test]
fn a_core_tab_returns_to_its_source_after_a_stale_arrival(cx: &mut TestAppContext) {
    let group = uuid::Uuid::from_u128(72);
    let (source, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_core_window);
    let document = || sheprd_core_client::wire::Doc {
        value: serde_json::json!({"schema_version": 1, "workspaces": [
            {"id": group, "label": "Core group"}
        ]}),
        version: 3,
    };
    let (key, tab) = cx.update(|window, cx| {
        crate::core_terminals::set_document_for_test_as(cx, "hub-a", document());
        let key = crate::browser::CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
        let tab = Store::update(cx, |store| store.open_core(&key, None)).unwrap();
        source.update(cx, |view, cx| {
            assert!(view.show_core_group(&key, cx));
            view.endpoints.clear();
            view.live.snapshot = None;
            view.show_browser_tab(tab, window, cx);
        });
        (key, tab)
    });
    cx.simulate_resize(size(px(1600.), px(600.)));
    for _ in 0..2 {
        draw(cx);
    }
    carry_to(cx, &format!("browser-tab-{tab}"), OUTSIDE);
    drop_at(cx, OUTSIDE);
    let torn = torn(cx).remove(0);
    assert_eq!(torn.workspace, key.parts());

    let target_window = cx.add_window(crate::sidebar::layout_tests::fixture_core_window);
    target_window
        .update(cx, |view, _, cx| {
            assert!(view.show_core_group(&key, cx));
            view.endpoints.clear();
            view.live.snapshot = None;
        })
        .unwrap();
    cx.update(|_, cx| crate::core_terminals::set_document_for_test_as(cx, "hub-a", document()));
    source.read_with(cx, |view, _| {
        assert!(view.browser.hidden.hides(&key.parts(), &Pick::Page(tab)));
    });
    target_window
        .update(cx, |view, window, cx| {
            assert!(!view.arrive(torn, window, cx))
        })
        .unwrap();
    source.read_with(cx, |view, _| {
        assert!(!view.browser.hidden.hides(&key.parts(), &Pick::Page(tab)));
    });
    cx.update(|_, cx| assert!(cx.global::<Store>().get(tab).is_some()));
}

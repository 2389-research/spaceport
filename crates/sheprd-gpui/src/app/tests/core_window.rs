// ABOUTME: Checks that a second Core window keeps the exact group identity.
// ABOUTME: The fixture joins through a private connector and creates no legacy endpoint.

use crate::{
    SheprdWindow,
    browser::CoreGroupKey,
    core_terminals::{self, Error, Start, State},
    sidebar::layout_tests::fixture_window,
};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

fn private_join(start: Start) -> Result<sheprd_core_client::CoreBackend, Error> {
    assert_eq!(start, Start::Join);
    Err(Error::NotInstalled)
}

#[gpui::test]
fn additional_core_window_keeps_exact_group_without_legacy_endpoint(cx: &mut gpui::TestAppContext) {
    let group = Uuid::from_u128(71);
    let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
    let (first, cx) = cx.add_window_view(fixture_window);
    let before = cx.update(|_, cx| main_windows(cx));
    cx.update(|_, cx| {
        core_terminals::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"Core group"}
                ]}),
                version: 3,
            },
        );
    });
    first.update(cx, |view, cx| {
        assert!(view.show_core_group(&key, cx));
        view.endpoints.clear();
        view.live.snapshot = None;
        super::super::open_additional_core_window_then(
            key.clone(),
            None,
            |view, _, _| view.core_terminals.hub.set_join_for_test(private_join),
            |_| panic!("Core group was available"),
            cx,
            true,
        );
    });
    cx.run_until_parked();
    let windows = cx.update(|_, cx| main_windows(cx));
    assert_eq!(windows.len(), 2);
    let second = windows
        .into_iter()
        .find(|window| !before.contains(window))
        .unwrap();
    second
        .read_with(cx, |view, _| {
            assert!(view.endpoints.is_empty());
            assert_eq!(view.selected_core_group(), Some(&key));
        })
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    loop {
        let joined = second
            .update(cx, |view, _, _| {
                view.core_terminals.hub.poll();
                matches!(view.core_terminals.hub.state(), State::Unavailable(error) if matches!(error.as_ref(), Error::NotInstalled))
            })
            .unwrap();
        if joined {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Core join did not answer"
        );
        std::thread::yield_now();
    }
}

#[gpui::test]
fn additional_core_window_refuses_a_different_hub(cx: &mut gpui::TestAppContext) {
    let group = Uuid::from_u128(71);
    let (_, cx) = cx.add_window_view(fixture_window);
    cx.update(|_, cx| {
        core_terminals::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"Core group"}
                ]}),
                version: 3,
            },
        );
        let key = CoreGroupKey::new("hub-b", None, "spaceport", group).unwrap();
        let opened = super::super::open_window_placed(
            super::super::WindowLaunch::Core(key),
            crate::updater::Updater::secondary(),
            None,
            cx,
            true,
        );
        assert!(opened.is_err());
        assert_eq!(main_windows(cx).len(), 1);
    });
}

fn main_windows(cx: &gpui::App) -> Vec<gpui::WindowHandle<SheprdWindow>> {
    cx.windows()
        .iter()
        .filter_map(gpui::AnyWindowHandle::downcast::<SheprdWindow>)
        .collect()
}

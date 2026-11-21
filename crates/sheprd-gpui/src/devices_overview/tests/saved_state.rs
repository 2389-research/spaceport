// ABOUTME: Exercises the Devices overview command with unreadable saved layouts.
// ABOUTME: Checks that a restored overview remains available when new tabs are blocked.
use super::*;
use crate::{
    browser::{Location, Store, Tab},
    window::Flash,
};

fn install_saved_overview(
    view: &Entity<SheprdWindow>,
    cx: &mut VisualTestContext,
    workspace: &str,
) -> (tempfile::TempDir, Tab) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("browser-tabs.json");
    let tab = Tab {
        id: 7_u64.into(),
        scope: view.read_with(cx, |view, _| crate::browser::scope(&view.endpoints[0])),
        workspace_id: workspace.into(),
        location: Some(Location::Devices),
        title: Location::Devices.default_title(),
        origin: None,
    };
    let saved = serde_json::json!({"tabs": [&tab], "next": 8});
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    std::fs::write(dir.path().join("editor-groups.json"), b"{").unwrap();
    let store = Store::load(Some(path));
    cx.update(|_, cx| store.install(cx));
    (dir, tab)
}

#[gpui::test]
fn the_overview_command_warns_before_creating_a_tab_with_unreadable_layouts(
    cx: &mut gpui::TestAppContext,
) {
    let (view, visual) = window(cx);
    let (_dir, saved) = install_saved_overview(&view, visual, "w1");
    open(&view, visual);
    let (flash, shown, tabs) = view.read_with(visual, |view, cx| {
        (
            view.flash.as_ref().map(|(flash, _)| flash.clone()),
            view.devices_overview_open(),
            cx.global::<Store>().tabs().cloned().collect::<Vec<_>>(),
        )
    });
    // Finish any queued save before releasing the fixture directory.
    cx.quit();

    assert_eq!(
        flash,
        Some(Flash::warning(
            "Saved tabs or layouts need repair. Check browser-tabs.json and editor-groups.json, \
             then restart Spaceport before opening new tabs."
        ))
    );
    assert!(!shown);
    assert_eq!(tabs, [saved]);
}

#[gpui::test]
fn the_overview_command_can_reopen_a_saved_tab_with_unreadable_layouts(
    cx: &mut gpui::TestAppContext,
) {
    let (view, visual) = window(cx);
    let (_dir, saved) = install_saved_overview(&view, visual, "w0");
    open(&view, visual);
    let drawn = visual.debug_bounds("devices-tab").is_some();
    let (shown, warned, tabs) = view.read_with(visual, |view, cx| {
        (
            view.devices_overview.pages.contains_key(&saved.id),
            view.flash.is_some(),
            cx.global::<Store>().tabs().cloned().collect::<Vec<_>>(),
        )
    });
    cx.quit();

    assert!(drawn && shown);
    assert!(!warned);
    assert_eq!(tabs, [saved]);
}

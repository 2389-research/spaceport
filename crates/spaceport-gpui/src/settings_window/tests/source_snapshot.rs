//! The one value Settings reads the window it was opened from through.
use super::*;
use spaceport_client::ConnectTarget;

fn open_settings_from(
    cx: &mut TestAppContext,
    source: &WindowHandle<SpaceportWindow>,
) -> WindowHandle<SettingsWindow> {
    let weak = cx.update(|cx| source.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
    cx.update(|cx| {
        open_fixture(weak, cx);
        cx.global::<SettingsWindowHandle>().window.unwrap()
    })
}

#[gpui::test]
fn the_editor_starts_from_the_appearance_of_the_window_it_opened_from(cx: &mut TestAppContext) {
    let source = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    cx.update(|cx| {
        source
            .update(cx, |view, _, _| view.config.theme = "Nord".to_owned())
            .unwrap();
    });
    let settings = open_settings_from(cx, &source);
    cx.update(|cx| {
        let view = settings.read(cx).unwrap();
        assert_eq!(view.config.theme, "Nord");
        // The draft is diffed against what the window showed, not against a
        // reload's result.
        assert_eq!(view.loaded_config.theme, "Nord");
    });
}

#[gpui::test]
fn the_snapshot_follows_the_source_and_forgets_its_connection_when_it_closes(
    cx: &mut TestAppContext,
) {
    let source = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    let target = ConnectTarget::Socket("/unused-source.sock".into());
    let settings = open_settings_from(cx, &source);
    cx.update(|cx| {
        assert!(settings.read(cx).unwrap().source_window.hosts.is_empty());
    });
    // A host connects, its picker opens and a listing starts: the source says
    // so by notifying, which is what Settings follows.
    cx.update(|cx| {
        source
            .update(cx, |view, _, cx| {
                let endpoint = &mut view.endpoints[0];
                endpoint.connection.target = target.clone();
                endpoint.live_mut().status = crate::state::ConnectionStatus::Connected;
                view.menu.page = Some(crate::menu::Page::Themes);
                view.integrations.busy = true;
                cx.notify();
            })
            .unwrap();
    });
    cx.update(|cx| {
        let view = settings.read(cx).unwrap();
        assert_eq!(view.source_window.hosts.len(), 1);
        assert_eq!(view.source_window.hosts[0].target, target);
        assert_eq!(view.source_window.target, Some(target.clone()));
        assert!(view.source_window.theme_page_open);
        assert!(view.source_window.integrations_busy);
    });
    // Closing the window ends everything that described a live connection and
    // keeps the target a new window would still open on.
    cx.update(|cx| {
        source
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
    });
    cx.run_until_parked();
    cx.update(|cx| {
        let view = settings.read(cx).unwrap();
        assert!(view.source_window.hosts.is_empty());
        assert!(!view.source_window.theme_page_open);
        assert!(!view.source_window.integrations_busy);
        assert_eq!(view.source_window.target, Some(target));
    });
}

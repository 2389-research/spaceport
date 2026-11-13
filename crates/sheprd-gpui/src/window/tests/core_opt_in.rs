// ABOUTME: Prevents implicit Core connections and fallback while Core is disabled.
// ABOUTME: Uses a private connector that fails the test if any worker reaches it.
use super::*;
use crate::core_terminals::{Error, Start, State};
use sheprd_client::ConnectTarget;

#[test]
fn disabled_core_keeps_missing_herdr_guidance_in_the_status_bar() {
    crate::i18n::leak_tests::under_en_xa("disabled Core status", |cx, recorder| {
        let (view, cx) = cx.add_window_view(fixture_window);
        let expected = view.update(cx, |view, cx| {
            view.live.status = crate::state::ConnectionStatus::Disconnected;
            view.live.missing_installation = true;
            view.live.error = Some("private Herdr fixture is missing".into());
            cx.notify();
            view.live.status_text(None)
        });
        crate::i18n::leak_tests::draw(cx);
        let shown = recorder.take();
        assert!(
            shown.iter().any(|line| line.contains(&expected)),
            "missing Herdr guidance: {shown:?}"
        );
        let core = crate::i18n::fl!("status-bar-core-not-connected");
        assert!(
            !shown.iter().any(|line| line.contains(&core)),
            "disabled Core leaked status: {shown:?}"
        );
    });
}

fn forbidden_join(_: Start) -> Result<sheprd_core_client::CoreBackend, Error> {
    panic!("disabled Core must never dial or start a runtime");
}

#[gpui::test]
fn disabled_core_never_joins_or_starts_even_when_herdr_is_missing(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update_in(cx, |view, window, cx| {
        view.endpoints[0].connection.target = ConnectTarget::Local;
        view.config.herdr_2_terminals = true;
        view.live.missing_installation = true;
        view.core_terminals.hub.set_join_for_test(forbidden_join);
        view.poll_core_terminals(window, cx);
        assert!(matches!(view.core_terminals.hub.state(), State::Idle));
        assert!(!view.new_tabs_use_core());
    });
}

#[gpui::test]
fn saved_core_tabs_stay_hidden_without_being_discarded(cx: &mut gpui::TestAppContext) {
    use crate::browser::{Location, Store};
    let (view, cx) = cx.add_window_view(fixture_window);
    let tab = view.update_in(cx, |view, window, cx| {
        let mut current = snapshot(1);
        current.focused_workspace_id = Some("w0".into());
        view.live.snapshot = Some(Arc::new(current));
        let (scope, workspace) = view.browser_key().unwrap();
        let tab = Store::update(cx, |store| {
            store.open(
                scope,
                &workspace,
                Some(Location::CoreTerminal {
                    panes: crate::core_terminals::panes::Panes::single(
                        sheprd_backend::TerminalId::new("private-saved-terminal").unwrap(),
                    ),
                    observe: false,
                }),
                None,
            )
        })
        .unwrap();
        view.config.herdr_core_enabled = true;
        assert!(view.browser_tab_ids(cx).is_empty());
        assert!(view.browser_tab_labels(cx).is_empty());
        view.show_browser_tab(tab, window, cx);
        view.poll_browser(window, cx);
        assert!(cx.global::<Store>().get(tab).is_some());
        assert!(
            !view.core_enabled(),
            "pending opt-in must not affect existing windows"
        );
        cx.notify();
        tab
    });
    crate::i18n::leak_tests::draw(cx);
    assert!(
        cx.debug_bounds(format!("browser-tab-{tab}").leak())
            .is_none()
    );
    let (other, cx) = cx.add_window_view(fixture_window);
    other.read_with(cx, |view, _| {
        assert!(!view.core_enabled(), "new windows share the startup gate")
    });
}

#![allow(clippy::unwrap_used)]

use super::*;
use crate::sidebar::layout_tests::fixture_window;
use gpui::{VisualTestContext, px, size};
use sheprd_core_client::{Error as CoreError, Method};

#[test]
fn unknown_core_add_stays_fenced_after_the_form_is_gone() {
    let gate = Gate::default();
    let request = Request::new("user@lab.invalid", "lab", "default").unwrap();
    assert_eq!(gate.begin(request.clone()), Ok(()));
    assert_eq!(
        gate.begin(request.clone()),
        Err(State::InFlight(request.clone()))
    );
    gate.finish(&Err(CoreError::Timeout(Method::MachineAdd)));
    assert_eq!(
        gate.begin(Request::new("other.invalid", "", "").unwrap()),
        Err(State::Unknown(request))
    );
}

#[test]
fn a_definite_core_refusal_releases_the_add_gate() {
    let gate = Gate::default();
    let request = Request::new("user@lab.invalid", "lab", "default").unwrap();
    assert_eq!(gate.begin(request.clone()), Ok(()));
    gate.finish(&Err(CoreError::Refused {
        method: Method::MachineAdd,
        kind: Some("needs_login".into()),
        message: "login required".into(),
        hint: None,
    }));
    assert_eq!(gate.begin(request), Ok(()));
}

#[test]
fn only_a_confirmed_machine_id_releases_the_add_gate() {
    let machine = |id: &str| Machine {
        host: "lab".into(),
        id: id.into(),
        name: "lab".into(),
        session: None,
        target: "user@lab.invalid".into(),
        temporary: false,
        version: "0.1.18".into(),
        via: false,
    };
    let request = Request::new("user@lab.invalid", "lab", "").unwrap();
    let gate = Gate::default();
    gate.begin(request.clone()).unwrap();
    gate.finish(&Ok(machine("")));
    assert_eq!(gate.state(), State::Unknown(request.clone()));

    let confirmed = Gate::default();
    confirmed.begin(request.clone()).unwrap();
    confirmed.finish(&Ok(machine("remote-core-id")));
    assert_eq!(confirmed.begin(request), Ok(()));
}

#[test]
fn core_add_keeps_the_submitted_target_and_optional_fields() {
    let request = Request::new(" user@lab.invalid ", " Lab ", " ").unwrap();
    assert_eq!(request.target, "user@lab.invalid");
    assert_eq!(request.name.as_deref(), Some("Lab"));
    assert_eq!(request.session, None);
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

#[gpui::test]
fn core_add_form_keeps_the_fields_without_host_key_bypass(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.simulate_resize(size(px(800.), px(700.)));
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_add_device(window, cx)));
    draw(cx);
    assert!(cx.debug_bounds("device-setup-dialog").is_some());
    assert!(cx.debug_bounds("device-setup-skip-host-key").is_none());
    assert!(cx.debug_bounds("device-setup-submit").is_some());
}

#[gpui::test]
fn unknown_add_remains_blocked_after_the_dialog_closes(cx: &mut gpui::TestAppContext) {
    let gate = Gate::default();
    let request = Request::new("user@lab.invalid", "Lab", "").unwrap();
    gate.begin(request.clone()).unwrap();
    gate.finish(&Err(CoreError::Timeout(Method::MachineAdd)));
    cx.update(|cx| cx.set_global(gate));
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_add_device(window, cx);
            assert!(matches!(
                view.menu.device_setup.as_ref().unwrap().step,
                Step::CoreUnknown
            ));
            assert!(
                view.menu
                    .device_setup
                    .as_ref()
                    .unwrap()
                    .error
                    .as_ref()
                    .unwrap()
                    .contains("user@lab.invalid")
            );
            assert!(
                view.menu
                    .device_setup
                    .as_ref()
                    .unwrap()
                    .error
                    .as_ref()
                    .unwrap()
                    .contains("Restarting Spaceport does not prove the outcome")
            );
            view.dismiss_menu(window, cx);
            view.open_add_device(window, cx);
            view.submit_device_setup(window, cx);
            assert!(matches!(
                view.menu.device_setup.as_ref().unwrap().step,
                Step::CoreUnknown
            ));
        });
    });
}

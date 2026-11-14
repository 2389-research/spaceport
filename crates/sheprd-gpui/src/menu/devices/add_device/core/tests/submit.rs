// ABOUTME: Checks that submitting the Add Device form starts the Core flow.
// ABOUTME: Stubs the Core join so no real Core is contacted.
use super::*;
use crate::core_terminals::{Error as HubError, Start, State as HubState};
use std::time::{Duration, Instant};

fn refuse_join(_: Start) -> Result<CoreBackend, HubError> {
    Err(HubError::NotInstalled)
}

fn step(view: &SheprdWindow) -> &Step {
    &view.menu.device_setup.as_ref().unwrap().step
}

#[gpui::test]
fn submitting_the_form_connects_to_core_before_adding(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_core_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_join_for_test(refuse_join);
            view.open_add_device(window, cx);

            // Without a target nothing is submitted.
            view.submit_core_device(window, cx);
            assert!(matches!(step(view), Step::Form));
            assert!(view.menu.device_setup.as_ref().unwrap().error.is_some());

            let target = view.menu.device_setup.as_ref().unwrap().fields[0].clone();
            target.update(cx, |input, cx| input.set_text_at("user@lab.invalid", 0, cx));
            view.submit_core_device(window, cx);
            assert!(matches!(
                step(view),
                Step::CoreConnecting(request) if request.target == "user@lab.invalid"
            ));
            assert!(view.menu.device_setup.as_ref().unwrap().error.is_none());

            // A Core that cannot be joined returns the form with the reason.
            let deadline = Instant::now() + Duration::from_secs(5);
            while matches!(view.core_terminals.hub.state(), HubState::Connecting(_)) {
                assert!(Instant::now() < deadline, "Core join did not finish");
                view.core_terminals.hub.poll();
                std::thread::yield_now();
            }
            view.poll_device_setup(window, cx);
            assert!(matches!(step(view), Step::Form));
            assert!(view.menu.device_setup.as_ref().unwrap().error.is_some());
        });
    });
}

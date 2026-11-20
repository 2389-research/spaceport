// ABOUTME: Builds device setup popups without Core, WSL, or catalog operations.
// ABOUTME: Keeps native input and pending-task ownership available to cleanup tests.
use super::*;

pub(in crate::menu) fn fixture(
    page: Page,
    window: &mut gpui::Window,
    cx: &mut Context<SheprdWindow>,
) -> SheprdWindow {
    let mut view = crate::sidebar::layout_tests::fixture_core_window(window, cx);
    install(&mut view, page, window, cx);
    view
}

pub(super) fn install(
    view: &mut SheprdWindow,
    page: Page,
    window: &mut gpui::Window,
    cx: &mut Context<SheprdWindow>,
) {
    view.open_menu(window, cx);
    match page {
        Page::AddDevice => view.open_add_device(window, cx),
        Page::AddWsl => {
            open_form(view, &["Private-Alpha", "Private-Beta"], cx);
            let setup = view.menu.wsl_setup.as_mut().unwrap();
            setup.selected = Some("Private-Alpha".into());
            setup.session.update(cx, |input, cx| {
                input.set_appearance(view.config.ui.clone(), view.theme.clone(), cx);
                input.set_placeholder("default", cx);
                window.focus(&input.focus, cx);
            });
        }
        Page::RemoveWsl => view.open_remove_wsl("wsl:Private-Alpha", Point::default(), window, cx),
        _ => unreachable!(),
    }
}

pub(in crate::menu) fn long_fixture(
    page: Page,
    window: &mut gpui::Window,
    cx: &mut Context<SheprdWindow>,
) -> SheprdWindow {
    let mut view = fixture(page, window, cx);
    match page {
        Page::AddDevice => {
            view.menu.device_setup.as_mut().unwrap().error =
                Some("Private fixture details. ".repeat(20))
        }
        Page::AddWsl => {
            view.menu.wsl_setup.as_mut().unwrap().distros = Distros::Ready(
                (0..80)
                    .map(|index| format!("Private-distro-{index:02}"))
                    .collect(),
            )
        }
        Page::RemoveWsl => view.menu.wsl_remove = Some(format!("Private-{}", "x".repeat(48))),
        _ => {}
    }
    view
}

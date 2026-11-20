// ABOUTME: Supplies private host popup fixtures for departure and ownership checks.
// ABOUTME: Avoids saved-profile writes and real SSH children during regression tests.
use super::*;
use gpui::{Context, Window};

pub(in crate::menu) fn fixture(
    page: Page,
    window: &mut Window,
    cx: &mut Context<crate::SheprdWindow>,
) -> crate::SheprdWindow {
    let mut view = fixture_window(window, cx);
    add_host(&mut view);
    cx.set_global(crate::device_tags::DeviceTags::default());
    view.open_host_menu(HOST, point(px(20.), px(20.)), window, cx);
    match page {
        Page::RenameDevice => view.activate_host_menu(Action::Rename, window, cx),
        Page::ForwardPort => {
            #[cfg(unix)]
            view.activate_host_menu(Action::ForwardPort, window, cx);
        }
        Page::DeviceTags => view.activate_host_menu(
            Action::Tags(super::super::tags::Mode::AlsoTagged),
            window,
            cx,
        ),
        Page::RemoveDevice => {
            view.activate_host_menu(Action::Remove, window, cx);
            // A confirmed fixture cannot invoke a saved-profile CLI worker.
            view.menu.host.as_mut().unwrap().saved = Saved::Local;
        }
        _ => {}
    }
    view
}

pub(in crate::menu) fn long_tags_fixture(
    window: &mut Window,
    cx: &mut Context<crate::SheprdWindow>,
) -> crate::SheprdWindow {
    let mut view = fixture(Page::Host, window, cx);
    let names: Vec<_> = (0..80).map(|index| format!("fixture-{index:02}")).collect();
    let records: Vec<_> = names
        .iter()
        .map(|name| (name.as_str(), vec![name.as_str()]))
        .collect();
    let records: Vec<_> = records
        .iter()
        .map(|(id, tags)| (*id, tags.as_slice()))
        .collect();
    view.device_tags = crate::device_tags::Tags::tagged(&records).into();
    view.activate_host_menu(
        Action::Tags(super::super::tags::Mode::AlsoTagged),
        window,
        cx,
    );
    view
}

// ABOUTME: Shares private device setup and picker fixtures.
// ABOUTME: Covers host targeting and retained device behavior across Core opt-in modes.
use super::{
    Page,
    add_device::{Setup, Step},
};
use crate::{SheprdWindow, search_input::SearchInput, sidebar::layout_tests::fixture_window};
use gpui::{AppContext, Context};

mod core_catalog;
mod core_flag;
mod core_remote_document;
mod en_xa;
mod fold;
mod manual_entry;
mod rail;
mod scope;
mod selection;

fn open_form(view: &mut SheprdWindow, step: Step, cx: &mut Context<SheprdWindow>) {
    view.menu.device_setup = Some(Setup {
        fields: std::array::from_fn(|_| cx.new(SearchInput::new)),
        step,
        error: None,
    });
    view.menu.page = Some(Page::AddDevice);
}

#[gpui::test]
fn a_saved_device_closes_the_dialog_by_itself(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            open_form(view, Step::Saved, cx);
            view.poll_device_setup(window, cx);
            assert!(view.menu.page.is_none());
            assert!(view.menu.device_setup.is_none());
            assert!(view.focus.is_focused(window));
        });
    });
}

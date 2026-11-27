//! What the popup chrome measures before it draws: which pages hang from the
//! footer, which dim the window, and how much room a footer menu has.

use crate::appear::Appear;
use crate::chrome::{Hosts, Metrics, Text, dims, footer_room, layer};
use crate::page::{Page, WorkspaceAction};
use gpui::{Font, Size, point, px, rgba, size};
use spaceport_settings::Theme;
use std::time::Instant;

fn text(size: f32) -> Text {
    Text {
        font: Font::default(),
        line_height: size * 20. / 14.,
        size,
    }
}

fn viewport(width: f32, height: f32) -> Size<gpui::Pixels> {
    size(px(width), px(height))
}

struct Fixture {
    appear: Appear,
    theme: Theme,
}

impl Fixture {
    fn new() -> Self {
        Self {
            appear: Appear::default(),
            theme: Theme::default(),
        }
    }

    fn hosts(
        &self,
        page: Page,
        size: f32,
        window: Size<gpui::Pixels>,
        session_modal: bool,
    ) -> Hosts<'_, ()> {
        Hosts {
            add_device: matches!(page, Page::AddDevice | Page::AddWsl),
            anchor: point(px(300.), px(600.)),
            appear: &self.appear,
            backdrop: rgba(0x000000b0),
            host_forwards: false,
            leaving: None,
            metrics: Metrics {
                chrome: 36.,
                viewport: window,
            },
            now: Instant::now(),
            palette_previews: false,
            popover: None,
            session_modal,
            text: text(size),
            theme: &self.theme,
        }
    }
}

#[test]
fn the_all_devices_picker_rises_from_the_footer_until_a_session_form_dims_it() {
    let fixture = Fixture::new();
    let window = viewport(1200., 800.);
    let hosts = fixture.hosts(Page::Devices, 12., window, false);
    let picker = layer(&hosts, Page::Devices);
    assert!(picker.footer_anchored);
    assert!(picker.rises);
    assert!(!picker.dims);
    assert!(!picker.pointer_anchored);

    let hosts = fixture.hosts(Page::Devices, 12., window, true);
    let form = layer(&hosts, Page::Devices);
    assert!(form.session_modal);
    assert!(!form.footer_anchored);
    assert!(!form.rises);
    assert!(form.dims);
}

#[test]
fn a_list_of_actions_hangs_at_the_pointer_and_a_dialog_dims_the_window() {
    assert!(!dims(Page::Menu, false));
    assert!(!dims(Page::Application, false));
    assert!(dims(Page::Palette, false));
    assert!(dims(Page::Dialog(WorkspaceAction::NewTab), false));
    assert!(!dims(Page::Tab, false));
}

#[test]
fn a_footer_menu_opens_where_the_room_is() {
    let window = viewport(1200., 800.);
    let (room, above) = footer_room(point(px(300.), px(760.)), window, 36.);
    assert!(above);
    assert_eq!(room, px(704.));
    let (room, above) = footer_room(point(px(300.), px(200.)), window, 36.);
    assert!(!above);
    assert_eq!(room, px(580.));
}

#[test]
fn the_add_device_rail_folds_to_icons_before_its_form_runs_out_of_room() {
    let fixture = Fixture::new();
    let narrow = viewport(500., 400.);
    let wide = viewport(1200., 800.);
    let hosts = fixture.hosts(Page::AddDevice, 12., narrow, false);
    let narrow_layer = layer(&hosts, Page::AddDevice);
    assert!(narrow_layer.add_device);
    assert!(narrow_layer.compact_rail);

    // The rail's room grows with the UI font, so a larger face folds sooner.
    let hosts = fixture.hosts(Page::AddDevice, 12., wide, false);
    assert!(!layer(&hosts, Page::AddDevice).compact_rail);
    let hosts = fixture.hosts(Page::AddDevice, 24., viewport(1100., 800.), false);
    assert!(layer(&hosts, Page::AddDevice).compact_rail);
}

#[test]
fn a_page_on_its_way_out_shows_as_much_as_it_is_leaving_with() {
    let fixture = Fixture::new();
    let window = viewport(1200., 800.);
    let hosts = fixture.hosts(Page::Devices, 12., window, false);
    // A list of popup pages settles at once; one that just closed is drawn
    // at the share its leave had reached.
    assert_eq!(layer(&hosts, Page::Devices).shown, 1.);
    let mut hosts = fixture.hosts(Page::Devices, 12., window, false);
    hosts.leaving = Some(0.25);
    let leaving = layer(&hosts, Page::Devices);
    assert_eq!(leaving.shown, 0.25);
    assert_eq!(leaving.leaving, Some(0.25));
    assert!(leaving.lift > px(0.));
}

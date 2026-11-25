//! The wheel rule both clients share: which grid a report lands in, where in
//! it, and how many events a count becomes.

use super::*;
use spaceport_protocol::{FrameData, SurfaceRect};

fn pane(mouse_reporting: bool, alternate_screen: bool) -> PaneSurfacePane {
    PaneSurfacePane {
        pane_id: "pane".into(),
        content_revision: 1,
        rect: SurfaceRect {
            x: 0,
            y: 0,
            width: 40,
            height: 24,
        },
        inner_rect: SurfaceRect {
            x: 1,
            y: 1,
            width: 38,
            height: 22,
        },
        scrollbar_rect: None,
        scroll: None,
        focused: false,
        mouse_reporting,
        sgr_pixel_mouse: false,
        alternate_screen_active: alternate_screen,
        pixel_width: 380,
        pixel_height: 440,
    }
}

fn popup(mouse_reporting: bool, sgr_pixel_mouse: bool) -> ClientShellPopupSurface {
    ClientShellPopupSurface {
        terminal_id: "popup".into(),
        title: String::new(),
        width: None,
        height: None,
        frame: FrameData {
            cells: vec![],
            width: 20,
            height: 10,
            cursor: None,
            hyperlinks: vec![],
            graphics: vec![],
        },
        mouse_reporting,
        sgr_pixel_mouse,
        pixel_width: 200,
        pixel_height: 200,
    }
}

fn lines(hit: WheelHit, steps: u16) -> Vec<u16> {
    hit.reports(ClientMouseKind::ScrollUp, steps, 0)
        .map(|event| match event {
            ClientPaneInputEvent::Mouse { lines, .. } => lines,
            event => panic!("unexpected wheel event {event:?}"),
        })
        .collect()
}

/// Herdr turns each event into exactly one report or arrow key, so a fast
/// swipe of five rows has to reach a program reading the wheel as five events.
/// The scrollback still moves by the whole count at once.
#[test]
fn an_application_reading_the_wheel_gets_every_step() {
    let scrollback = WheelGrid::pane(&pane(false, false));
    assert_eq!(lines(scrollback.at_cell(2, 2), 5), [5]);
    let reporting = WheelGrid::pane(&pane(true, false));
    assert_eq!(lines(reporting.at_cell(2, 2), 5), [1; 5]);
    // The alternate screen reads the wheel as arrow keys without reporting
    // the mouse at all.
    let alternate = WheelGrid::pane(&pane(false, true));
    assert_eq!(lines(alternate.at_cell(2, 2), 128), [1; 128]);
    // A popup does not say whether it is on the alternate screen.
    assert_eq!(
        lines(WheelGrid::popup(&popup(false, false)).at_cell(0, 0), 5),
        [5]
    );
    assert_eq!(
        lines(WheelGrid::popup(&popup(true, false)).at_cell(0, 0), 5),
        [1; 5]
    );
}

/// A program that asked for pixel positions is told the pointer's own place in
/// them; every other program is told the cell. A client that knows only the
/// cell it touched reports that cell's top-left pixel.
#[test]
fn a_pixel_program_is_told_the_pointer_and_the_rest_the_cell() {
    let paint = WheelGrid::pane(&pane(false, false));
    assert_eq!(paint.geometry(), None);
    assert_eq!(
        paint.at_pixels(25., 45., 10., 20.).position(),
        ClientMousePosition::Cell { column: 2, row: 2 }
    );
    assert_eq!(paint.at_pixels(25., 45., 10., 20.).cell(), (2, 2));

    let mut pixel = pane(false, false);
    pixel.sgr_pixel_mouse = true;
    let pixel = WheelGrid::pane(&pixel);
    assert_eq!(
        pixel.geometry(),
        Some(ClientMouseGeometry {
            cols: 38,
            rows: 22,
            width_px: 380,
            height_px: 440,
        })
    );
    assert_eq!(
        pixel.at_pixels(25., 45., 10., 20.).position(),
        ClientMousePosition::Pixels {
            x: 25,
            y: 45,
            column: 2,
            row: 2,
        }
    );
    assert_eq!(
        pixel.at_cell(2, 2).position(),
        ClientMousePosition::Pixels {
            x: 20,
            y: 40,
            column: 2,
            row: 2,
        }
    );
}

/// A program that never reported its pixel size gets cell positions, even
/// when it asked for pixel ones.
#[test]
fn an_unmeasured_grid_reports_cells() {
    let mut unmeasured = pane(true, false);
    unmeasured.sgr_pixel_mouse = true;
    unmeasured.pixel_width = 0;
    assert_eq!(WheelGrid::pane(&unmeasured).geometry(), None);
    assert_eq!(
        WheelGrid::pane(&unmeasured)
            .at_pixels(25., 45., 10., 20.)
            .position(),
        ClientMousePosition::Cell { column: 2, row: 2 }
    );
}

/// A mouse gesture's report carries the grid's own extent and the modifiers
/// the user held.
#[test]
fn a_report_carries_the_geometry_and_modifiers() {
    let mut pixel = pane(false, false);
    pixel.sgr_pixel_mouse = true;
    let hit = WheelGrid::pane(&pixel).at_cell(3, 4);
    assert_eq!(
        hit.report(
            ClientMouseKind::Down(spaceport_protocol::ClientMouseButton::Left),
            1,
            1
        ),
        ClientPaneInputEvent::Mouse {
            kind: ClientMouseKind::Down(spaceport_protocol::ClientMouseButton::Left),
            position: ClientMousePosition::Pixels {
                x: 30,
                y: 80,
                column: 3,
                row: 4,
            },
            geometry: Some(ClientMouseGeometry {
                cols: 38,
                rows: 22,
                width_px: 380,
                height_px: 440,
            }),
            modifiers: 1,
            lines: 1,
        }
    );
}

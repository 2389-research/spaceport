// ABOUTME: Exercises the cell and terminal fields the phone carries unchanged.
// ABOUTME: Covers hyperlink indexes with their target table and scroll metrics.
use super::*;

/// The fixture surface with `hyperlink` on the window's first and third cells
/// and one target in the frame's table.
fn linked_surface() -> PaneSurfaceFrame {
    let mut frame = surface();
    frame.frame.hyperlinks = vec!["https://herdr.dev/docs?q=one#two".into()];
    frame.frame.cells[1].hyperlink = Some(0);
    frame.frame.cells[4].hyperlink = Some(0);
    frame
}

#[test]
fn a_cell_keeps_its_hyperlink_index_and_the_frame_keeps_the_targets() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    state.surface(Arc::new(linked_surface()));
    let terminal = state.view().terminal.unwrap();
    assert_eq!(terminal.hyperlinks, ["https://herdr.dev/docs?q=one#two"]);
    assert_eq!(
        terminal
            .cells
            .iter()
            .map(|cell| cell.hyperlink)
            .collect::<Vec<_>>(),
        [Some(0), None, Some(0), None]
    );
    let target = terminal.cells[0]
        .hyperlink
        .and_then(|index| terminal.hyperlinks.get(index as usize));
    assert_eq!(
        target.map(String::as_str),
        Some("https://herdr.dev/docs?q=one#two")
    );
}

#[test]
fn a_terminal_keeps_the_pane_scroll_position() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    let mut frame = surface();
    frame.panes[0].scroll = Some(PaneSurfaceScrollMetrics {
        offset_from_bottom: 4,
        max_offset_from_bottom: 120,
        viewport_rows: 2,
    });
    state.surface(Arc::new(frame));
    assert_eq!(
        state.view().terminal.unwrap().scroll,
        Some(MobileScroll {
            offset_from_bottom: 4,
            max_offset_from_bottom: 120,
            viewport_rows: 2,
        })
    );
}

#[test]
fn a_popup_keeps_its_own_targets_and_has_no_scroll_position() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    let mut frame = linked_surface();
    let mut popup = cells("OK", 2);
    popup.hyperlinks = vec!["mailto:ops@herdr.dev".into()];
    popup.cells[1].hyperlink = Some(0);
    frame.popup = Some(Box::new(ClientShellPopupSurface {
        terminal_id: "confirm".into(),
        title: String::new(),
        width: None,
        height: None,
        frame: popup,
        mouse_reporting: false,
        sgr_pixel_mouse: false,
        pixel_width: 16,
        pixel_height: 16,
    }));
    state.surface(Arc::new(frame));
    let terminal = state.view().terminal.unwrap();
    assert_eq!(terminal.hyperlinks, ["mailto:ops@herdr.dev"]);
    assert_eq!(
        terminal
            .cells
            .iter()
            .map(|cell| cell.hyperlink)
            .collect::<Vec<_>>(),
        [None, Some(0)]
    );
    assert_eq!(terminal.scroll, None);
}

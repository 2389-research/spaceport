// ABOUTME: Checks the output progress reported for a headless terminal group.
// ABOUTME: Establishes the authored row through the window's normal presentation path.
use super::{draw, fixture_window_on_t0, surface};
use crate::working_profile::report::{OutputProgress, PaneOutput};
use gpui::TestAppContext;
use sheprd_client::protocol::{CellData, ClientShellPane};
use std::sync::Arc;

#[gpui::test]
fn reports_output_progress_from_the_presented_pane(cx: &mut TestAppContext) {
    const PANE: &str = "p0";
    const OUTPUT_ROW: &str = "SP_PROFILE_READY SP_OUTPUT:STEP:01!";
    const ROW: usize = 2;

    let (view, cx) = fixture_window_on_t0(cx);
    let (boot, projection_revision) = view.update(cx, |view, _| {
        let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        snapshot.focused_pane_id = Some(PANE.into());
        snapshot.panes.push(ClientShellPane {
            pane_id: PANE.into(),
            workspace_id: "w0".into(),
            tab_id: "t0".into(),
            label: None,
            cwd: None,
            foreground_cwd: None,
            focused: true,
            right_click_passthrough: false,
        });
        let mut picture = surface(snapshot, PANE);
        let frame = &mut Arc::make_mut(&mut picture).frame;
        let width = usize::from(frame.width);
        frame.cells = vec![
            CellData {
                symbol: " ".into(),
                fg: 0,
                bg: 0,
                modifier: 0,
                skip: false,
                hyperlink: None,
            };
            width * usize::from(frame.height)
        ];
        let row = frame.cells.chunks_exact_mut(width).nth(ROW).unwrap();
        assert!(OUTPUT_ROW.len() <= row.len());
        for (cell, symbol) in row.iter_mut().zip(OUTPUT_ROW.chars()) {
            cell.symbol = symbol.to_string();
        }
        frame.validate().unwrap();
        let identity = (snapshot.boot_id.clone(), snapshot.revision);
        view.live.surface = Some(picture);
        view.endpoints[view.selected_endpoint].live = view.live.clone();
        assert!(view.live.surface_ready());
        identity
    });
    draw(cx);
    assert!(cx.debug_bounds("terminal").is_some());

    view.read_with(cx, |view, cx| {
        let groups = view.working_profile_groups(cx);
        let [group] = groups.as_slice() else {
            panic!("expected one drawn terminal group, got {groups:?}");
        };
        assert!(group.primary && group.shown_terminal && group.connected);
        assert_eq!(group.tab.as_deref(), Some("t0"));
        let frame = group.presented.as_ref().unwrap();
        assert!(frame.current && frame.marker_visible);
        assert_eq!(frame.boot, boot);
        assert_eq!(frame.projection_revision, projection_revision);
        assert_eq!(frame.surface_revision, 1);
        assert_eq!((frame.columns, frame.rows), (80, 24));
        assert_eq!(frame.pane_ids, [PANE]);
        assert!(frame.nonblank_cells > 0);

        let (shown, _) = view.presentation.profile_presented().unwrap();
        shown.frame.validate().unwrap();
        let [pane] = shown.panes.as_slice() else {
            panic!("expected one pane in the presented surface");
        };
        assert_eq!(pane.pane_id, PANE);
        let rect = pane.inner_rect;
        let top = usize::from(rect.y);
        assert!((top..top + usize::from(rect.height)).contains(&ROW));
        let width = usize::from(shown.frame.width);
        let left = usize::from(rect.x);
        let right = left + usize::from(rect.width);
        let cells = shown.frame.cells.chunks_exact(width).nth(ROW).unwrap();
        let row: String = cells[left..right]
            .iter()
            .map(|cell| cell.symbol.as_str())
            .collect();
        assert_eq!(row.trim_end(), OUTPUT_ROW);

        assert_eq!(
            frame.outputs,
            Some(vec![PaneOutput {
                pane_id: PANE.into(),
                progress: OutputProgress {
                    last_step: 1,
                    complete: false,
                },
            }]),
            "expected step 1 from the presented pane"
        );
    });
}

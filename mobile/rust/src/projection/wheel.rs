// ABOUTME: Maps a touch scroll to Herdr's pane or popup mouse contract.
// ABOUTME: Collapses scrollback motion and preserves discrete application wheel events.
use super::{InputTarget, Projection};
use crate::MobileError;
use spaceport_client::protocol::{
    ClientMouseGeometry, ClientMouseKind, ClientMousePosition, ClientPaneInputEvent,
};

impl Projection {
    pub(crate) fn wheel(
        &self,
        target: &InputTarget,
        lines: i16,
        column: u16,
        row: u16,
    ) -> Result<Vec<ClientPaneInputEvent>, MobileError> {
        if self.target().as_ref() != Some(target) {
            return Err(MobileError::StaleView);
        }
        let (_, surface) = self.coherent().ok_or(MobileError::StaleView)?;
        let (width, height, width_px, height_px, pixels, discrete) =
            if let Some(popup) = &surface.popup {
                (
                    popup.frame.width,
                    popup.frame.height,
                    popup.pixel_width,
                    popup.pixel_height,
                    popup.sgr_pixel_mouse,
                    popup.mouse_reporting,
                )
            } else {
                let pane = surface
                    .panes
                    .iter()
                    .find(|p| p.pane_id == target.pane_id)
                    .ok_or(MobileError::StaleView)?;
                (
                    pane.inner_rect.width,
                    pane.inner_rect.height,
                    pane.pixel_width,
                    pane.pixel_height,
                    pane.sgr_pixel_mouse,
                    pane.mouse_reporting || pane.alternate_screen_active,
                )
            };
        if column >= width || row >= height || lines.unsigned_abs() > 128 {
            return Err(MobileError::InvalidScroll);
        }
        if lines == 0 {
            return Ok(vec![]);
        }
        let geometry = (pixels && width_px > 0 && height_px > 0).then_some(ClientMouseGeometry {
            cols: width,
            rows: height,
            width_px,
            height_px,
        });
        let position = if geometry.is_some() {
            let pixel = |at: u16, extent: u32, cells: u16| {
                (u64::from(at) * u64::from(extent) / u64::from(cells)) as u32
            };
            ClientMousePosition::Pixels {
                x: pixel(column, width_px, width),
                y: pixel(row, height_px, height),
                column,
                row,
            }
        } else {
            ClientMousePosition::Cell { column, row }
        };
        let steps = lines.unsigned_abs();
        let event = ClientPaneInputEvent::Mouse {
            kind: if lines > 0 {
                ClientMouseKind::ScrollUp
            } else {
                ClientMouseKind::ScrollDown
            },
            position,
            geometry,
            modifiers: 0,
            lines: if discrete { 1 } else { steps },
        };
        Ok(vec![event; if discrete { usize::from(steps) } else { 1 }])
    }
}

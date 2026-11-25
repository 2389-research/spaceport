// ABOUTME: Maps a touch scroll to Herdr's pane or popup mouse contract.
// ABOUTME: Reports through the wheel rule this build shares with the desktop.
use super::{InputTarget, Projection};
use crate::MobileError;
use spaceport_backend::WheelGrid;
use spaceport_client::protocol::{ClientMouseKind, ClientPaneInputEvent};

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
        let grid = if let Some(popup) = &surface.popup {
            WheelGrid::popup(popup)
        } else {
            let pane = surface
                .panes
                .iter()
                .find(|p| p.pane_id == target.pane_id)
                .ok_or(MobileError::StaleView)?;
            WheelGrid::pane(pane)
        };
        if column >= grid.columns() || row >= grid.rows() || lines.unsigned_abs() > 128 {
            return Err(MobileError::InvalidScroll);
        }
        if lines == 0 {
            return Ok(vec![]);
        }
        let kind = if lines > 0 {
            ClientMouseKind::ScrollUp
        } else {
            ClientMouseKind::ScrollDown
        };
        Ok(grid
            .at_cell(column, row)
            .reports(kind, lines.unsigned_abs(), 0)
            .collect())
    }
}

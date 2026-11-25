//! Where a wheel or pointer report lands, and how it travels: the cell and
//! pixel extents a pane or popup reported, the position inside them, and
//! whether the program reads the wheel one step at a time.
//!
//! A program that reports the mouse, or that is on the alternate screen where
//! Herdr turns the wheel into arrow keys, reads the wheel stepwise: Herdr makes
//! exactly one report or key of each event, so a count has to travel as that
//! many events, as a terminal sends one report per wheel step.

use spaceport_protocol::{
    ClientMouseGeometry, ClientMouseKind, ClientMousePosition, ClientPaneInputEvent,
    ClientShellPopupSurface, PaneSurfacePane,
};

/// The grid a wheel or pointer report lands in, as the daemon described it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WheelGrid {
    alternate_screen: bool,
    columns: u16,
    mouse_reporting: bool,
    pixel_height: u32,
    pixel_mouse: bool,
    pixel_width: u32,
    rows: u16,
}

impl WheelGrid {
    /// A popup's grid. A popup does not say whether it is on the alternate
    /// screen, so only its own mouse reporting makes it stepwise.
    pub fn popup(popup: &ClientShellPopupSurface) -> Self {
        Self {
            alternate_screen: false,
            columns: popup.frame.width,
            mouse_reporting: popup.mouse_reporting,
            pixel_height: popup.pixel_height,
            pixel_mouse: popup.sgr_pixel_mouse,
            pixel_width: popup.pixel_width,
            rows: popup.frame.height,
        }
    }

    /// A pane's grid: its inner rect, which is the part the split leaves for
    /// cells.
    pub fn pane(pane: &PaneSurfacePane) -> Self {
        Self {
            alternate_screen: pane.alternate_screen_active,
            columns: pane.inner_rect.width,
            mouse_reporting: pane.mouse_reporting,
            pixel_height: pane.pixel_height,
            pixel_mouse: pane.sgr_pixel_mouse,
            pixel_width: pane.pixel_width,
            rows: pane.inner_rect.height,
        }
    }

    pub fn columns(self) -> u16 {
        self.columns
    }

    pub fn rows(self) -> u16 {
        self.rows
    }

    /// Whether the program reports the mouse, which is the only way Herdr
    /// turns a horizontal wheel into anything.
    pub fn mouse_reporting(self) -> bool {
        self.mouse_reporting
    }

    /// Whether the wheel reaches the program rather than the scrollback.
    pub fn stepwise(self) -> bool {
        self.mouse_reporting || self.alternate_screen
    }

    /// The pixel extent a report carries, when the program reads positions in
    /// pixels and the daemon measured the grid.
    pub fn geometry(self) -> Option<ClientMouseGeometry> {
        (self.pixel_mouse && self.pixel_width > 0 && self.pixel_height > 0).then_some(
            ClientMouseGeometry {
                cols: self.columns,
                rows: self.rows,
                width_px: self.pixel_width,
                height_px: self.pixel_height,
            },
        )
    }

    /// Where a pointer `x`, `y` pixels into the grid reports, one cell being
    /// `cell_width` by `cell_height`.
    pub fn at_pixels(self, x: f32, y: f32, cell_width: f32, cell_height: f32) -> WheelHit {
        let pixel = |at: f32, cells: u16, cell_size: f32, extent: u32| {
            (at / (f32::from(cells) * cell_size) * extent as f32).floor() as u32
        };
        let pixels = (
            pixel(x, self.columns, cell_width, self.pixel_width),
            pixel(y, self.rows, cell_height, self.pixel_height),
        );
        self.hit(
            (x / cell_width).floor() as u16,
            (y / cell_height).floor() as u16,
            pixels,
        )
    }

    /// Where the cell at `column`, `row` reports, for a client that knows the
    /// cell it touched and not its place inside it: the cell's own top-left
    /// corner stands for it. `column` and `row` must be inside the grid.
    pub fn at_cell(self, column: u16, row: u16) -> WheelHit {
        let pixel = |at: u16, cells: u16, extent: u32| {
            (u64::from(at) * u64::from(extent) / u64::from(cells.max(1))) as u32
        };
        let pixels = (
            pixel(column, self.columns, self.pixel_width),
            pixel(row, self.rows, self.pixel_height),
        );
        self.hit(column, row, pixels)
    }

    /// A program reading pixels is told the pointer's own place in them;
    /// everything else is told the cell.
    fn hit(self, column: u16, row: u16, (x, y): (u32, u32)) -> WheelHit {
        let geometry = self.geometry();
        let position = match geometry {
            Some(_) => ClientMousePosition::Pixels { x, y, column, row },
            None => ClientMousePosition::Cell { column, row },
        };
        WheelHit {
            geometry,
            position,
            stepwise: self.stepwise(),
        }
    }
}

/// Where inside a grid one report aims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WheelHit {
    geometry: Option<ClientMouseGeometry>,
    position: ClientMousePosition,
    stepwise: bool,
}

impl WheelHit {
    pub fn position(self) -> ClientMousePosition {
        self.position
    }

    pub fn geometry(self) -> Option<ClientMouseGeometry> {
        self.geometry
    }

    /// The cell the report names, a pixel position included.
    pub fn cell(self) -> (u16, u16) {
        match self.position {
            ClientMousePosition::Cell { column, row }
            | ClientMousePosition::Pixels { column, row, .. } => (column, row),
        }
    }

    /// One mouse report of this position carrying `lines` steps.
    pub fn report(self, kind: ClientMouseKind, lines: u16, modifiers: u8) -> ClientPaneInputEvent {
        ClientPaneInputEvent::Mouse {
            kind,
            position: self.position,
            geometry: self.geometry,
            modifiers,
            lines,
        }
    }

    /// The reports for `steps` whole steps of one kind. Scrollback gets one
    /// event carrying the whole count, which Herdr scrolls by; a program
    /// reading the wheel gets one single-step event per step.
    pub fn reports(
        self,
        kind: ClientMouseKind,
        steps: u16,
        modifiers: u8,
    ) -> impl Iterator<Item = ClientPaneInputEvent> {
        let (events, each) = if self.stepwise {
            (steps, 1)
        } else {
            (1, steps)
        };
        (0..events).map(move |_| self.report(kind, each, modifiers))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests;

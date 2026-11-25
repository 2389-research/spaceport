//! The rectangles a daemon reports for one frame: which pane shows where, in
//! cells. A client paints or navigates only a complete map, so the rectangles
//! must tile the frame exactly once.

use spaceport_backend::TerminalId;

/// A pane's part of a frame, counted in whole cells from its top-left corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl CellRect {
    /// Whether `other` shares a cell with this rectangle. Both stay inside the
    /// frame, so neither sum can overflow.
    fn overlaps(self, other: Self) -> bool {
        self.x < other.x + other.width
            && other.x < self.x + self.width
            && self.y < other.y + other.height
            && other.y < self.y + self.height
    }
}

/// Where each pane of one frame shows, in cells, in the order the daemon
/// listed them.
///
/// Built only from rectangles that tile the frame exactly once: a client
/// leaning on a partial or ambiguous map would miss a pane, or hand a touch to
/// one that is not there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellMap {
    columns: u16,
    rows: u16,
    panes: Vec<(TerminalId, CellRect)>,
}

impl CellMap {
    /// The map `panes` make of a `columns` × `rows` frame, or `None` when they
    /// do not tile it: a rectangle with no area or outside the frame, two panes
    /// over one cell, a pane named twice, a cell no pane claims, or no panes at
    /// all.
    pub fn new(
        columns: u16,
        rows: u16,
        panes: impl IntoIterator<Item = (TerminalId, CellRect)>,
    ) -> Option<Self> {
        if columns == 0 || rows == 0 {
            return None;
        }
        let mut claimed = 0u32;
        let mut placed: Vec<(TerminalId, CellRect)> = Vec::new();
        for (id, rect) in panes {
            let right = rect.x.checked_add(rect.width)?;
            let bottom = rect.y.checked_add(rect.height)?;
            if rect.width == 0
                || rect.height == 0
                || right > columns
                || bottom > rows
                || placed.iter().any(|(_, other)| rect.overlaps(*other))
                || placed.iter().any(|(other, _)| *other == id)
            {
                return None;
            }
            claimed += u32::from(rect.width) * u32::from(rect.height);
            placed.push((id, rect));
        }
        // Inside the frame and clear of each other, the panes cover every cell
        // exactly once when their areas add up to the frame's.
        (claimed == u32::from(columns) * u32::from(rows)).then_some(Self {
            columns,
            rows,
            panes: placed,
        })
    }

    pub fn columns(&self) -> u16 {
        self.columns
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    /// Each pane and its part of the frame, in the order they were reported.
    pub fn panes(&self) -> &[(TerminalId, CellRect)] {
        &self.panes
    }
}

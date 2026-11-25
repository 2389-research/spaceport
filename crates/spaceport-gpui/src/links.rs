//! Links the daemon reads from its own terminal state: `pane.link.resolve`
//! names every cell a link covers, including the rows a soft-wrapped URL
//! continues onto, so a hover underlines the whole link.
//!
//! It is read-only. The daemon reports spans and never acts on a click, so no
//! plugin link handler can run, and this client opens what it reads itself.
//! It is also optional: a daemon that does not advertise it keeps the
//! row-local detector in `terminal::links`, which needs no round trip.

use crate::terminal::{CellMetrics, InputTarget, wheel_target};
use serde::Deserialize;
use serde_json::{Value, json};
use spaceport_client::{
    ClientEvent, Method,
    protocol::{PaneSurfaceFrame, SurfaceRect},
    response,
    scrollback::content_settled,
};

/// The resolve request in flight on one connection. Its response stops here
/// instead of reaching the live state, so a hover that went stale is never
/// reported as a connection error. Like the live inbox, a reconnect replaces
/// the whole mailbox, so a late answer cannot land in the next connection.
#[derive(Default)]
pub(crate) struct LinkInbox {
    resolving: Option<Pending>,
}

struct Pending {
    id: String,
    result: Option<crate::Result<Value>>,
}

impl LinkInbox {
    /// Records the answers to this mailbox's own requests, passing every
    /// other event on.
    pub(crate) fn apply(&mut self, event: ClientEvent) -> Option<ClientEvent> {
        if let ClientEvent::Disconnected { .. } = &event
            && let Some(pending) = &mut self.resolving
        {
            pending
                .result
                .get_or_insert(Err(crate::Error::NotConnected));
        }
        let (id, rejected) = match &event {
            ClientEvent::Response { request_id, .. } => (request_id, false),
            ClientEvent::CommandRejected {
                request_id: Some(request_id),
                ..
            } => (request_id, true),
            _ => return Some(event),
        };
        let Some(pending) = self
            .resolving
            .as_mut()
            .filter(|pending| pending.id == *id && pending.result.is_none())
        else {
            return Some(event);
        };
        pending.result = Some(match event {
            ClientEvent::Response { response, .. } if !rejected => Ok(response),
            ClientEvent::CommandRejected { reason, .. } => Err(crate::Error::Client(reason)),
            _ => Err(crate::Error::NotConnected),
        });
        None
    }

    /// Queues one request while holding the mailbox, so even an immediate
    /// rejection finds it registered. A second request waits until the first
    /// is answered and taken. The worker rejects a method the daemon did not
    /// advertise.
    pub(crate) fn request(
        &mut self,
        handle: &spaceport_client::ClientHandle,
        cell: &LinkCell,
    ) -> crate::Result<String> {
        if self.resolving.is_some() {
            return Err(crate::Error::ConnectionBusy);
        }
        let id = handle.request(&cell.boot, Method::PaneLinkResolve, cell.params())?;
        self.resolving = Some(Pending {
            id: id.clone(),
            result: None,
        });
        Ok(id)
    }

    /// The answer to request `id`, freeing the slot. `None` while it is
    /// still in flight, or when the slot belongs to another request.
    pub(crate) fn take(&mut self, id: &str) -> Option<crate::Result<Value>> {
        if self
            .resolving
            .as_ref()
            .is_none_or(|pending| pending.id != id || pending.result.is_none())
        {
            return None;
        }
        self.resolving.take()?.result
    }
}

/// One pane cell and the content it was read from. The daemon refuses a
/// request whose pane has since changed, scrolled, or moved, and this client
/// drops an answer for a cell the pointer has left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkCell {
    boot: String,
    pane_id: String,
    inner_rect: SurfaceRect,
    content_revision: u64,
    offset_from_bottom: Option<u64>,
    /// Viewport row and column within the pane's inner rect.
    row: u16,
    col: u16,
}

impl LinkCell {
    /// The pane cell under a point in surface pixels. A popup covers every
    /// pane, and the daemon resolves pane content only, so a popup yields
    /// nothing here and keeps the local detector.
    pub(crate) fn at(
        surface: &PaneSurfaceFrame,
        x: f32,
        y: f32,
        cell_width: f32,
        cell_height: f32,
    ) -> Option<Self> {
        let InputTarget::Pane(pane_id) =
            wheel_target(surface, x, y, cell_width, cell_height)?.target
        else {
            return None;
        };
        let pane = surface.panes.iter().find(|pane| pane.pane_id == pane_id)?;
        // The daemon would refuse a revision that is a write in progress.
        if !content_settled(pane.content_revision) {
            return None;
        }
        let rect = pane.inner_rect;
        let (col, row) = CellMetrics::new(cell_width, cell_height).cell_at(x, y);
        let (col, row) = (col.checked_sub(rect.x)?, row.checked_sub(rect.y)?);
        (col < rect.width && row < rect.height).then(|| Self {
            boot: surface.boot_id.clone(),
            pane_id,
            inner_rect: rect,
            content_revision: pane.content_revision,
            offset_from_bottom: pane.scroll.map(|scroll| scroll.offset_from_bottom),
            row,
            col,
        })
    }

    /// Whether both cells read the same pane content, wherever in it they are.
    pub(crate) fn same_content(&self, other: &Self) -> bool {
        self.boot == other.boot
            && self.pane_id == other.pane_id
            && self.inner_rect == other.inner_rect
            && self.offset_from_bottom == other.offset_from_bottom
            && self.content_revision == other.content_revision
    }

    /// Whether `surface` still shows the content this cell was read from.
    pub(crate) fn current(&self, surface: &PaneSurfaceFrame) -> bool {
        surface.boot_id == self.boot
            && surface.popup.is_none()
            && surface.panes.iter().any(|pane| {
                pane.pane_id == self.pane_id
                    && pane.inner_rect == self.inner_rect
                    && pane.content_revision == self.content_revision
                    && pane.scroll.map(|scroll| scroll.offset_from_bottom)
                        == self.offset_from_bottom
            })
    }

    fn params(&self) -> Value {
        json!({
            "pane_id": self.pane_id,
            "viewport_row": self.row,
            "col": self.col,
            "content_revision": self.content_revision,
            "offset_from_bottom": self.offset_from_bottom,
        })
    }
}

/// One row of a resolved link, in the pane's viewport. Both columns are
/// inclusive, as the daemon reports them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) struct LinkRegion {
    pub row: u16,
    pub start_col: u16,
    pub end_col: u16,
}

impl LinkRegion {
    fn contains(&self, cell: &LinkCell) -> bool {
        self.row == cell.row && (self.start_col..=self.end_col).contains(&cell.col)
    }
}

/// A resolved link and the cell it was resolved for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedLink {
    pub cell: LinkCell,
    pub regions: Vec<LinkRegion>,
}

impl ResolvedLink {
    /// The link the daemon resolved for `cell`. Anything malformed, out of
    /// the pane, or not covering the asked-for cell is no link: the daemon's
    /// answer is untrusted data, and a hover may simply show nothing.
    pub(crate) fn from_response(cell: LinkCell, response: &Value) -> Self {
        #[derive(Deserialize)]
        struct Resolved {
            regions: Vec<LinkRegion>,
        }
        let rect = cell.inner_rect;
        let regions = response::typed::<Resolved>(response, "pane_link_resolved")
            .ok()
            .map(|resolved| resolved.regions)
            .filter(|regions| {
                regions.len() <= usize::from(rect.height)
                    && regions.iter().all(|region| {
                        region.row < rect.height
                            && region.start_col <= region.end_col
                            && region.end_col < rect.width
                    })
                    && regions.iter().any(|region| region.contains(&cell))
            })
            .unwrap_or_default();
        Self { cell, regions }
    }

    /// Whether `cell` reads the same content and lies on this link.
    pub(crate) fn covers(&self, cell: &LinkCell) -> bool {
        self.cell.same_content(cell) && self.regions.iter().any(|region| region.contains(cell))
    }

    /// The covered cells as `(row, columns)` in the surface frame's grid,
    /// the shape the painter takes.
    pub(crate) fn frame_rows(&self) -> impl Iterator<Item = (u16, std::ops::Range<u16>)> + '_ {
        let rect = self.cell.inner_rect;
        self.regions.iter().map(move |region| {
            (
                rect.y.saturating_add(region.row),
                rect.x.saturating_add(region.start_col)
                    ..rect.x.saturating_add(region.end_col).saturating_add(1),
            )
        })
    }
}

#[cfg(test)]
mod tests;

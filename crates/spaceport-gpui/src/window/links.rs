//! Links under the pointer. A pointer that only passes over text asks
//! nothing and draws nothing. While the link modifier is held over a pane,
//! the text there is verified and, once it is, underlined: a well-formed
//! address at once, a file path when a check off the UI thread finds it on
//! disk, and a web address through the daemon, which is asked which cells
//! the link covers so that one wrapping onto the next row is underlined
//! whole.
//!
//! Only a click with the link modifier held opens a link, and this client
//! opens it itself: a web address in the browser, a file in the default
//! application. The daemon is asked which cells a link covers and nothing
//! else, so no link handler runs for a click. Daemons without that method
//! keep the row-local detector.
// i18n: migrated

use super::{SpaceportWindow, file_links::FileLink};
use crate::terminal::PaneLink;
use crate::{
    browser::WebUrl,
    links::{LinkCell, LinkInbox, ResolvedLink},
    terminal::{ExternalUrl, RowLink, RowTarget},
};
use gpui::{Context, Modifiers, Pixels, Point, Task, Window};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

/// Pointer movement is coalesced: at most one resolve leaves per delay, for
/// wherever the pointer is when it fires.
const RESOLVE_DELAY: Duration = Duration::from_millis(40);

#[derive(Default)]
pub(crate) struct DaemonLinks {
    /// The pane cell under the pointer while the link modifier is held.
    hover: Option<LinkCell>,
    /// The latest answer. It stays shown while the pointer moves along the
    /// link and the pane content it was read from is unchanged.
    resolved: Option<ResolvedLink>,
    resolving: Option<Resolving>,
    delay: Option<Task<()>>,
    /// What the row-local detector reads under the pointer while the link
    /// modifier is held, once it is verified: underlined when the daemon
    /// resolved nothing there. A file path counts once it is found on disk.
    local: Option<RowLink>,
    /// The file path last looked for on disk, and whether it is there;
    /// `None` while the check runs.
    checked: Option<(FileLink, Option<bool>)>,
    /// Where the pointer last was, for the check's answer to be applied to.
    pointer: Option<(Point<Pixels>, Modifiers)>,
}

/// A resolve request in the mailbox it was queued on. A reconnect or another
/// endpoint replaces that mailbox, which retires the request unanswered.
struct Resolving {
    id: String,
    inbox: Arc<Mutex<LinkInbox>>,
    /// The cell it was asked for.
    cell: LinkCell,
}

impl Resolving {
    /// The answer once it has arrived, or `Retired` once it never can.
    fn poll(&self, current: &Arc<Mutex<LinkInbox>>) -> PollResult {
        if !Arc::ptr_eq(&self.inbox, current) {
            return PollResult::Retired;
        }
        match self
            .inbox
            .try_lock()
            .ok()
            .and_then(|mut inbox| inbox.take(&self.id))
        {
            Some(result) => PollResult::Answered(result),
            None => PollResult::Waiting,
        }
    }
}

enum PollResult {
    Waiting,
    Retired,
    Answered(crate::Result<serde_json::Value>),
}

/// What a press with the link modifier held is on, with whatever was already
/// verified about it. A press is one of these or no link at all, and only a
/// click on one acts: the daemon is never asked to.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PressedTarget {
    /// A web address the row-local detector validated, which the click opens
    /// in the browser.
    Web(WebUrl),
    /// A local file, which the click opens in the system's application once
    /// the check off the UI thread finds it.
    File(FileLink),
    /// An explicit hyperlink this client never opens: an address with another
    /// scheme, a file on another host, or a destination it cannot read. A
    /// click only explains it.
    Unopened(RowTarget),
}

/// A press that may become a click on a link.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PressedLink {
    pub(crate) target: PressedTarget,
    pub(crate) position: Point<Pixels>,
}

impl PressedLink {
    /// Whether a release reads the same link as this press did.
    ///
    /// A link is what it points at, so a redraw that leaves the same target
    /// under the pointer keeps the click wherever the content revision moved.
    /// Text replaced with another target, or with none, is a different click.
    fn same_link(&self, release: &Self) -> bool {
        self.target == release.target
    }
}

impl PressedTarget {
    /// What a click on the link opens, or which refusal it explains.
    fn click(self) -> Option<ClickTarget> {
        match self {
            Self::Web(url) => Some(ClickTarget::Web(url)),
            Self::File(link) => Some(ClickTarget::File(link)),
            Self::Unopened(RowTarget::External(url)) => Some(ClickTarget::External(url)),
            Self::Unopened(RowTarget::OtherHost(host)) => {
                Some(ClickTarget::Unopened(crate::Error::LinkOnAnotherHost {
                    host,
                }))
            }
            Self::Unopened(RowTarget::Malformed) => {
                Some(ClickTarget::Unopened(crate::Error::MalformedLink))
            }
            // A press files a web address and a path under their own
            // variants, so no click reads one here.
            Self::Unopened(RowTarget::Web(_) | RowTarget::Path { .. }) => None,
        }
    }
}

impl SpaceportWindow {
    fn link_cell_at(&self, position: Point<Pixels>) -> Option<LinkCell> {
        if self.menu.page.is_some()
            || !self.live.surface_ready()
            || !self.bounds.contains(&position)
        {
            return None;
        }
        let (x, y) = self.drawn_position(position)?;
        LinkCell::at(
            self.live.surface.as_deref()?,
            x,
            y,
            self.cell_width,
            self.config.terminal.line_height(),
        )
    }

    pub(crate) fn terminal_hyperlink_at(&self, position: Point<Pixels>) -> bool {
        if self.menu.page.is_some()
            || !self.live.surface_ready()
            || !self.bounds.contains(&position)
        {
            return false;
        }
        let Some((x, y)) = self.drawn_position(position) else {
            return false;
        };
        self.live.surface.as_deref().is_some_and(|surface| {
            crate::terminal::pane_hyperlink_at(
                surface,
                x,
                y,
                self.cell_width,
                self.config.terminal.line_height(),
            )
        })
    }

    /// The resolved link under the pointer, while it still reads the live
    /// content it was resolved from.
    pub(crate) fn hovered_daemon_link(&self) -> Option<&ResolvedLink> {
        let hover = self.links.hover.as_ref()?;
        let link = self
            .links
            .resolved
            .as_ref()
            .filter(|link| link.covers(hover))?;
        link.cell
            .current(self.live.surface.as_deref()?)
            .then_some(link)
    }

    /// The verified row-local link under the pointer while the link modifier
    /// is held.
    pub(crate) fn hovered_local_link(&self) -> Option<&RowLink> {
        self.links.local.as_ref()
    }

    /// Whether the file `link` names is on disk, as far as is known yet. The
    /// first question about a path starts a check on the background
    /// executor, whose answer is applied to wherever the pointer is then.
    fn file_found(&mut self, link: FileLink, cx: &mut Context<Self>) -> bool {
        if let Some((checked, found)) = &self.links.checked
            && *checked == link
        {
            return *found == Some(true);
        }
        self.links.checked = Some((link.clone(), None));
        let asked = link.clone();
        let found = cx.background_executor().spawn(async move { asked.found() });
        cx.spawn(async move |this, cx| {
            let found = found.await;
            // The window may have closed while waiting; nothing is owed then.
            let _ = this.update(cx, |this, cx| {
                // The pointer may have moved on to another path meanwhile.
                if !matches!(&this.links.checked, Some((checked, None)) if *checked == link) {
                    return;
                }
                this.links.checked = Some((link, Some(found)));
                if let Some((position, modifiers)) = this.links.pointer {
                    this.hover_link(position, modifiers, cx);
                }
            });
        })
        .detach();
        false
    }

    /// The file `link` names, when a click may open it: always for an
    /// explicit hyperlink, and for printed text unless a check found it
    /// missing. A click before the check ends is not lost: opening reads the
    /// disk again, and opens nothing for a printed path that is not there.
    fn clickable_file(&self, link: PaneLink) -> Option<FileLink> {
        let file = self.file_link(link)?;
        let missing =
            matches!(&self.links.checked, Some((checked, Some(false))) if *checked == file);
        (file.explicit || !missing).then_some(file)
    }

    /// Whether a press is on a link known to be one: anything but a printed
    /// path whose check has not found it yet, which may name nothing.
    pub(crate) fn verified_press(&self, press: &PressedLink) -> bool {
        match &press.target {
            PressedTarget::File(file) => {
                file.explicit
                    || matches!(&self.links.checked, Some((checked, Some(true))) if checked == file)
            }
            PressedTarget::Web(_) | PressedTarget::Unopened(_) => true,
        }
    }

    /// Follows the pointer and the link modifier. Without the modifier
    /// nothing is read, asked, or shown; with it, the text under the pointer
    /// is verified, and the daemon is asked about a cell no earlier answer
    /// covers.
    pub(crate) fn hover_link(
        &mut self,
        position: Point<Pixels>,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        self.links.pointer = Some((position, modifiers));
        let mut examined = false;
        let local = crate::keymap::secondary(&modifiers)
            .then(|| self.local_link_at(position))
            .flatten()
            .and_then(|link| {
                let row = link.link.clone();
                match row.target {
                    // A well-formed address is all that can be verified of it.
                    RowTarget::Web(_) | RowTarget::External(_) => Some(row),
                    // Nothing verifies these, so they keep what they have at rest.
                    RowTarget::OtherHost(_) | RowTarget::Malformed => None,
                    RowTarget::Path { .. } => {
                        let file = self.file_link(link)?;
                        examined = true;
                        self.file_found(file, cx).then_some(row)
                    }
                }
            });
        // The file may appear or go away meanwhile, so a path is checked
        // again each time the user looks at it anew: after letting go of the
        // modifier, or after leaving the path.
        if !examined {
            self.links.checked = None;
        }
        if local != self.links.local {
            self.links.local = local;
            cx.notify();
        }
        let cell = (crate::keymap::secondary(&modifiers) && self.live.supports_link_resolve)
            .then(|| self.link_cell_at(position))
            .flatten();
        if cell == self.links.hover {
            return;
        }
        let shown = self.hovered_daemon_link().cloned();
        self.links.hover = cell;
        if self.hovered_daemon_link() != shown.as_ref() {
            cx.notify();
        }
        self.schedule_link_resolve(cx);
    }

    /// The hovered cell when no answer, in flight or arrived, covers it.
    fn unresolved_hover(&self) -> Option<&LinkCell> {
        let hover = self.links.hover.as_ref()?;
        let answered = self
            .links
            .resolved
            .as_ref()
            .is_some_and(|link| link.cell == *hover || link.covers(hover));
        (!answered).then_some(hover)
    }

    fn schedule_link_resolve(&mut self, cx: &mut Context<Self>) {
        if self.links.delay.is_some()
            || self.links.resolving.is_some()
            || self.unresolved_hover().is_none()
        {
            return;
        }
        self.links.delay = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RESOLVE_DELAY).await;
            // The window may have closed while waiting; nothing is owed then.
            let _ = this.update(cx, |this, cx| {
                this.links.delay = None;
                this.send_link_resolve(cx);
            });
        }));
    }

    fn send_link_resolve(&mut self, cx: &mut Context<Self>) {
        if self.links.resolving.is_some() {
            return;
        }
        let Some(cell) = self.unresolved_hover().cloned() else {
            return;
        };
        let connection = &self.endpoints[self.selected_endpoint].connection;
        match connection.request_link(&cell) {
            Ok(id) => {
                self.links.resolving = Some(Resolving {
                    id,
                    inbox: connection.links.clone(),
                    cell,
                });
            }
            // The event reader holds the mailbox for a moment; try again.
            Err(crate::Error::ConnectionBusy) => self.schedule_link_resolve(cx),
            // Hover is optional: a cell that cannot be asked about shows no
            // link and is not asked about again.
            Err(error) => {
                tracing::debug!(%error, "Link resolve not sent");
                self.links.resolved = Some(ResolvedLink {
                    cell,
                    regions: Vec::new(),
                });
            }
        }
    }

    /// Folds link answers into the window, and follows a pane that changed
    /// or scrolled under a still pointer.
    pub(crate) fn poll_links(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(endpoint) = self.endpoints.get(self.selected_endpoint) else {
            return;
        };
        let current = endpoint.connection.links.clone();
        if let Some(flight) = &self.links.resolving {
            match flight.poll(&current) {
                PollResult::Waiting => {}
                PollResult::Retired => self.links.resolving = None,
                PollResult::Answered(result) => {
                    let cell = flight.cell.clone();
                    self.links.resolving = None;
                    let shown = self.hovered_daemon_link().cloned();
                    self.links.resolved = Some(match result {
                        Ok(response) => ResolvedLink::from_response(cell, &response),
                        Err(_) => ResolvedLink {
                            cell,
                            regions: Vec::new(),
                        },
                    });
                    if self.hovered_daemon_link() != shown.as_ref() {
                        cx.notify();
                    }
                }
            }
        }
        self.hover_link(window.mouse_position(), window.modifiers(), cx);
        self.schedule_link_resolve(cx);
    }

    /// The link a press at `position` is on, by either reading. Only a press
    /// with the link modifier held is one: a plain click selects, focuses, or
    /// goes to the application, and never opens anything.
    pub(crate) fn terminal_link_press(
        &self,
        position: Point<Pixels>,
        modifiers: Modifiers,
    ) -> Option<PressedLink> {
        if !crate::keymap::secondary(&modifiers) {
            return None;
        }
        Some(PressedLink {
            target: self.press_target(position)?,
            position,
        })
    }

    /// What a press at `position` is on, by either reading: the address the
    /// row-local detector validated there, or else the link it reads.
    fn press_target(&self, position: Point<Pixels>) -> Option<PressedTarget> {
        if let Some(url) = self.terminal_link_at(position) {
            return WebUrl::try_from(url.as_str()).ok().map(PressedTarget::Web);
        }
        let link = self.local_link_at(position)?;
        match link.link.target {
            // A web address is the one `terminal_link_at` reads above.
            RowTarget::Web(_) => None,
            RowTarget::Path { .. } => self.clickable_file(link).map(PressedTarget::File),
            other => Some(PressedTarget::Unopened(other)),
        }
    }

    /// Opens the link a click released on, if it is the one it pressed.
    pub(crate) fn activate_terminal_link(
        &mut self,
        pressed: &PressedLink,
        release: Point<Pixels>,
        modifiers: Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(release) = self.terminal_link_press(release, modifiers) else {
            return false;
        };
        if !pressed.same_link(&release) {
            return false;
        }
        self.open_link(release.target.click(), window, cx);
        true
    }

    /// Opens what a click on a link points at. Terminal output is untrusted,
    /// so a web address opens in the browser, a file only once the check off
    /// the UI thread finds it, and another scheme asks first unless it is one
    /// that opens without asking.
    fn open_link(
        &mut self,
        target: Option<ClickTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match target {
            Some(ClickTarget::Web(url)) => cx.open_url(url.as_str()),
            Some(ClickTarget::File(link)) => self.open_file_link(link, cx),
            Some(ClickTarget::External(url)) if url.trusted() => cx.open_url(url.as_str()),
            Some(ClickTarget::External(url)) => self.ask_to_open_link(url, window, cx),
            Some(ClickTarget::Unopened(error)) => {
                self.show_flash(super::flash::Flash::warning(error.to_string()), cx)
            }
            None => {}
        }
    }
}

/// What a click with the link modifier held opens, read from the link under
/// it.
enum ClickTarget {
    Web(WebUrl),
    /// An address with another scheme: opened at once when the scheme is
    /// trusted, and after the user confirms it otherwise.
    External(ExternalUrl),
    /// An explicit hyperlink this client never opens, which a click only
    /// explains.
    Unopened(crate::Error),
    /// A local file checked before the system opens it.
    File(FileLink),
}

#[cfg(test)]
mod tests;

//! Control target matching and native page operations.

use crate::{
    SheprdWindow,
    browser::{Scope, TabId},
    control::Target,
};

impl SheprdWindow {
    /// The endpoint and workspace a control request names, if this window
    /// shows it. `strict` requires the caller's own daemon; otherwise any
    /// endpoint showing the named workspace qualifies. Only the Unix control
    /// socket asks.
    pub(crate) fn control_target(
        &self,
        target: &Target<'_>,
        strict: bool,
    ) -> Option<(usize, String)> {
        self.endpoints
            .iter()
            .enumerate()
            .find_map(|(index, endpoint)| {
                if strict {
                    let matches = match target.daemon {
                        Some(daemon) => {
                            endpoint.connection.target.socket_path().ok().as_deref() == Some(daemon)
                        }
                        None => index == self.selected_endpoint,
                    };
                    if !matches {
                        return None;
                    }
                } else if target.workspace.is_none() {
                    return None;
                }
                let live = if index == self.selected_endpoint {
                    &self.live
                } else {
                    &endpoint.live
                };
                let snapshot = live.snapshot.as_ref()?;
                let workspace = match target.workspace {
                    Some(id) => id,
                    None => snapshot.focused_workspace_id.as_deref()?,
                };
                snapshot
                    .workspaces
                    .iter()
                    .any(|candidate| candidate.workspace_id == workspace)
                    .then(|| (index, workspace.to_owned()))
            })
    }

    /// Whether the host this window shows is the daemon at `daemon`, listing
    /// workspaces or not, connected or not. Window-wide commands act on the
    /// shown host, so only that one may stand for the caller's daemon.
    pub(crate) fn shows_daemon(&self, daemon: &std::path::Path) -> bool {
        self.endpoints
            .get(self.selected_endpoint)
            .is_some_and(|endpoint| {
                endpoint.connection.target.socket_path().ok().as_deref() == Some(daemon)
            })
    }

    /// The endpoint of this window whose live snapshot lists `workspace` of
    /// `scope`, whether or not the window shows it now.
    pub(crate) fn endpoint_showing(&self, scope: &Scope, workspace: &str) -> Option<usize> {
        self.endpoints
            .iter()
            .enumerate()
            .find_map(|(index, endpoint)| {
                if &super::scope(endpoint) != scope {
                    return None;
                }
                let live = if index == self.selected_endpoint {
                    &self.live
                } else {
                    &endpoint.live
                };
                live.snapshot
                    .as_ref()?
                    .workspaces
                    .iter()
                    .any(|candidate| candidate.workspace_id == workspace)
                    .then_some(index)
            })
    }
}

/// The text a control request reads from a page: `null` until the page has
/// loaded, then its visible text, bounded before it leaves the page. The
/// app's own script; nothing a caller sends is ever run.
#[cfg(any(target_os = "macos", windows))]
// i18n: data
const PAGE_TEXT_SCRIPT: &str = "(function () {\
  if (document.readyState !== 'complete') { return null; }\
  var body = document.body;\
  return body ? String(body.innerText).slice(0, 65536) : '';\
})()";

/// A control request's view of this window's native pages.
#[cfg(any(target_os = "macos", windows))]
impl SheprdWindow {
    /// Whether this window has made the page of tab `id`.
    pub(crate) fn has_page(&self, id: TabId) -> bool {
        self.browser.pages.contains(id)
    }

    /// Loads `location` in tab `id`'s page, if this window has made it or
    /// is building it.
    pub(crate) fn load_requested_page(&mut self, id: TabId, location: &crate::browser::Location) {
        self.browser.pages.load(id, location);
    }

    /// Goes back, or forward, in tab `id`'s page. Whether the window has it.
    pub(crate) fn page_history(&self, id: TabId, forward: bool) -> bool {
        if !self.has_page(id) {
            return false;
        }
        if forward {
            self.browser.pages.forward(id);
        } else {
            self.browser.pages.back(id);
        }
        true
    }

    /// Asks tab `id`'s page for its text, handed to `done` as JSON: `null`
    /// while the page is still loading. Whether the window has the page.
    pub(crate) fn page_text(
        &self,
        id: TabId,
        done: impl Fn(String) + Send + 'static,
    ) -> crate::Result<bool> {
        let Some(page) = self.browser.pages.page(id) else {
            return Ok(false);
        };
        page.raw()
            .evaluate_script_with_callback(PAGE_TEXT_SCRIPT, done)?;
        Ok(true)
    }
}

#[cfg(target_os = "macos")]
impl SheprdWindow {
    /// Takes a picture of tab `id`'s page as it shows now, handed to `done`
    /// as TIFF bytes. Whether one was asked for: the page must be on
    /// screen, where it was laid out last frame.
    pub(crate) fn page_screenshot(
        &self,
        id: TabId,
        done: impl FnOnce(Option<Vec<u8>>) + 'static,
    ) -> bool {
        let Some(page) = self.browser.pages.page(id) else {
            return false;
        };
        if !self.browser.pages.is_shown(id) {
            return false;
        }
        let Some(bounds) = self.browser.pages.drawn_at(id) else {
            return false;
        };
        let rect = crate::browser::annotate::Rect {
            x: 0.,
            y: 0.,
            width: f64::from(f32::from(bounds.size.width)),
            height: f64::from(f32::from(bounds.size.height)),
        };
        crate::browser::snapshot::capture(page.raw(), rect, done);
        true
    }
}

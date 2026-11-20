// ABOUTME: Prepares acknowledged pane splits and focus trails before native input.
// ABOUTME: Derives measured targets from actual pane IDs and visible grid geometry.
use super::{Baseline, Command, Outcome, SheprdWindow, baseline, same_members, same_scene};
use crate::{endpoint::Step, navigation::NavigationTarget};
use anyhow::{Context as _, Result, ensure};
use gpui::{Context, Window};
use sheprd_client::protocol::{ClientShellSnapshot, SurfaceRect};
use std::collections::VecDeque;

enum Pending {
    Focus(String),
    Split,
}

pub(super) struct Preparation {
    before: Baseline,
    focuses: Option<VecDeque<String>>,
    original: String,
    outcome: Outcome,
    pending: Option<Pending>,
}

impl Preparation {
    pub(super) fn new(before: Baseline, outcome: Outcome) -> Self {
        Self {
            original: before.focused_pane.clone(),
            before,
            focuses: None,
            outcome,
            pending: None,
        }
    }

    pub(super) fn poll(
        &mut self,
        view: &mut SheprdWindow,
        window: &mut Window,
        cx: &mut Context<SheprdWindow>,
    ) -> Result<Option<Baseline>> {
        let current = baseline(view, window, cx)?;
        same_scene(&self.before, &current)?;
        ensure!(
            self.before.tabs == current.tabs
                && self.before.workspace_tabs == current.workspace_tabs
                && self.before.focused_tab == current.focused_tab
                && self.before.fullscreen == current.fullscreen,
            "pane preparation changed tabs or fullscreen state"
        );
        if let Some(pending) = &self.pending {
            let acknowledged = match pending {
                Pending::Focus(target) => {
                    same_members(&self.before, &current)?;
                    current.focused_pane == *target
                }
                Pending::Split => {
                    ensure!(
                        self.before
                            .panes
                            .iter()
                            .all(|id| current.panes.contains(id))
                            && current.panes.len() <= self.before.panes.len() + 1,
                        "pane preparation lost members or created extra panes"
                    );
                    current.panes.len() == self.before.panes.len() + 1
                        && !self.before.panes.contains(&current.focused_pane)
                }
            };
            if !acknowledged {
                return Ok(None);
            }
            ensure!(
                current.revision > self.before.revision,
                "pane preparation has no acknowledged daemon revision"
            );
            self.pending = None;
        } else {
            same_members(&self.before, &current)?;
        }
        self.before = current;
        let snapshot = view
            .live
            .snapshot
            .as_deref()
            .context("pane snapshot missing")?;
        let ids = tab_panes(snapshot);
        ensure!(
            (1..=4).contains(&ids.len()),
            "pane preparation has an unexpected grid size"
        );
        if ids.len() < 4 {
            if ids.len() == 3 && self.before.focused_pane != self.original {
                self.focus(self.original.clone(), view, cx)?;
            } else {
                let command = if ids.len() == 1 {
                    Command::SplitRight
                } else {
                    Command::SplitDown
                };
                self.pending = Some(Pending::Split);
                view.command(command, window, cx);
            }
            return Ok(None);
        }
        let prepared = grid(view)?;
        if self.focuses.is_none() {
            self.focuses = Some(match self.outcome {
                Outcome::Back => {
                    [prepared.top_left, prepared.top_right, prepared.bottom_right].into()
                }
                Outcome::Left => [prepared.top_right].into(),
                Outcome::Down | Outcome::Right => [prepared.top_left].into(),
                Outcome::Up => [prepared.bottom_left].into(),
                Outcome::NextPane | Outcome::PreviousPane | Outcome::Forward => VecDeque::new(),
                _ => anyhow::bail!("non-pane outcome reached pane preparation"),
            });
        }
        while let Some(target) = self.focuses.as_mut().and_then(VecDeque::pop_front) {
            if target != self.before.focused_pane {
                self.focus(target, view, cx)?;
                return Ok(None);
            }
        }
        let snapshot = view
            .live
            .snapshot
            .as_deref()
            .context("pane snapshot missing")?;
        let grid = grid(view)?;
        let target = match self.outcome {
            Outcome::NextPane | Outcome::PreviousPane => {
                let ids = tab_panes(snapshot);
                let at = ids
                    .iter()
                    .position(|id| *id == self.before.focused_pane)
                    .context("focused pane missing from its tab")?;
                let offset = if self.outcome == Outcome::NextPane {
                    1
                } else {
                    ids.len() - 1
                };
                ids[(at + offset) % ids.len()].to_owned()
            }
            Outcome::Back | Outcome::Forward => {
                let step = if self.outcome == Outcome::Back {
                    Step::Back
                } else {
                    Step::Forward
                };
                let expected = if self.outcome == Outcome::Back {
                    grid.top_right
                } else {
                    grid.bottom_right
                };
                let (_, actual) = view.endpoints[view.selected_endpoint]
                    .history
                    .peek(step, snapshot)
                    .context("prepared pane history has no distinct target")?;
                ensure!(
                    actual == expected,
                    "pane history differs from the acknowledged focus trail"
                );
                expected
            }
            Outcome::Left | Outcome::Up => grid.top_left,
            Outcome::Down => grid.bottom_left,
            Outcome::Right => grid.top_right,
            _ => anyhow::bail!("non-pane outcome reached pane target selection"),
        };
        ensure!(
            target != self.before.focused_pane,
            "pane case starts at its target"
        );
        window.focus(&view.focus, cx);
        let mut ready = baseline(view, window, cx)?;
        ready.pane_target = Some(target);
        Ok(Some(ready))
    }

    fn focus(
        &mut self,
        target: String,
        view: &mut SheprdWindow,
        cx: &mut Context<SheprdWindow>,
    ) -> Result<()> {
        // Record the outstanding action before dispatch; never replay an uncertain request.
        self.pending = Some(Pending::Focus(target.clone()));
        ensure!(
            view.navigate(NavigationTarget::Pane(&target), cx),
            "pane preparation focus was refused"
        );
        Ok(())
    }
}

fn tab_panes(snapshot: &ClientShellSnapshot) -> Vec<&str> {
    snapshot
        .panes
        .iter()
        .filter(|pane| {
            Some(&pane.workspace_id) == snapshot.focused_workspace_id.as_ref()
                && Some(&pane.tab_id) == snapshot.focused_tab_id.as_ref()
        })
        .map(|pane| pane.pane_id.as_str())
        .collect()
}

struct Grid {
    bottom_left: String,
    bottom_right: String,
    top_left: String,
    top_right: String,
}

fn grid(view: &SheprdWindow) -> Result<Grid> {
    let snapshot = view
        .live
        .snapshot
        .as_deref()
        .context("grid snapshot missing")?;
    let ids = tab_panes(snapshot);
    let surface = view.live.surface.as_ref().context("grid surface missing")?;
    let mut panes = surface
        .panes
        .iter()
        .filter(|pane| ids.contains(&pane.pane_id.as_str()))
        .collect::<Vec<_>>();
    ensure!(
        panes.len() == 4 && ids.len() == 4,
        "native pane grid needs four visible panes"
    );
    panes.sort_by_key(|pane| (pane.rect.y, pane.rect.x));
    let top = &mut panes[..2];
    top.sort_by_key(|pane| pane.rect.x);
    let bottom = &mut panes[2..];
    bottom.sort_by_key(|pane| pane.rect.x);
    let [tl, tr, bl, br] = panes.as_slice() else {
        anyhow::bail!("pane grid length changed")
    };
    let left_of =
        |a: SurfaceRect, b: SurfaceRect| u32::from(a.x) + u32::from(a.width) <= u32::from(b.x);
    let above =
        |a: SurfaceRect, b: SurfaceRect| u32::from(a.y) + u32::from(a.height) <= u32::from(b.y);
    ensure!(
        panes
            .iter()
            .all(|pane| pane.rect.width > 0 && pane.rect.height > 0)
            && left_of(tl.rect, tr.rect)
            && left_of(bl.rect, br.rect)
            && above(tl.rect, bl.rect)
            && above(tr.rect, br.rect)
            && tl.rect.x == bl.rect.x
            && tr.rect.x == br.rect.x
            && tl.rect.y == tr.rect.y
            && bl.rect.y == br.rect.y,
        "native panes do not form the prepared two-by-two geometry"
    );
    Ok(Grid {
        bottom_left: bl.pane_id.clone(),
        bottom_right: br.pane_id.clone(),
        top_left: tl.pane_id.clone(),
        top_right: tr.pane_id.clone(),
    })
}

pub(super) fn observed(
    view: &SheprdWindow,
    before: &Baseline,
    after: &Baseline,
    id: &str,
    window: &Window,
) -> Result<bool> {
    same_members(before, after)?;
    ensure!(
        before.focused_tab == after.focused_tab && before.fullscreen == after.fullscreen,
        "native pane key changed tab or fullscreen state"
    );
    let target = before
        .pane_target
        .as_ref()
        .context("native pane target missing")?;
    if &after.focused_pane != target {
        return Ok(false);
    }
    ensure!(
        after.revision > before.revision,
        "native pane focus has no new daemon revision"
    );
    ensure!(
        view.focus.is_focused(window) && view.menu.page.is_none(),
        "native pane key moved focus into chrome"
    );
    grid(view)?;
    eprintln!(
        "LINUX_KEYS_CASE id={id} pane_before={:.80} pane_target={target:.80} pane_after={:.80} geometry=two-by-two revision_acknowledged=true",
        before.focused_pane, after.focused_pane
    );
    Ok(true)
}

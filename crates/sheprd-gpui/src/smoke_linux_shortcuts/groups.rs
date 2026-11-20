// ABOUTME: Checks native editor-group splitting against real presentation ownership.
// ABOUTME: Closes only the exact newly created empty group between measured cases.
use super::SheprdWindow;
use crate::browser::{GroupId, Pick, Shown};
use anyhow::{Result, ensure};
use gpui::{App, Context, Window};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Baseline {
    active: Option<GroupId>,
    ids: Vec<GroupId>,
    tabs: Vec<Vec<Pick>>,
}

pub(super) fn snapshot(view: &SheprdWindow, cx: &App) -> Baseline {
    let ids = view
        .group_slots()
        .into_iter()
        .map(|slot| slot.id)
        .collect::<Vec<_>>();
    let tabs = ids.iter().map(|id| view.group_tabs(*id, cx)).collect();
    Baseline {
        active: view.active_group(),
        ids,
        tabs,
    }
}

pub(super) fn prepared(before: &Baseline) -> Result<()> {
    let [original] = before.ids.as_slice() else {
        anyhow::bail!("native group split requires one original group");
    };
    ensure!(
        before.active == Some(*original),
        "native group split does not start in the original group"
    );
    ensure!(
        !before.tabs[0].is_empty(),
        "native original group has no tabs to preserve"
    );
    Ok(())
}

pub(super) fn created(
    before: &Baseline,
    after: &Baseline,
    view: &SheprdWindow,
    cx: &App,
) -> Result<Option<GroupId>> {
    prepared(before)?;
    if after.ids == before.ids {
        return Ok(None);
    }
    let [left, right] = after.ids.as_slice() else {
        anyhow::bail!("native group split did not produce exactly two groups");
    };
    ensure!(
        *left == before.ids[0] && right != left,
        "native group split changed the original or right-hand identity"
    );
    ensure!(
        after.tabs[0] == before.tabs[0] && after.tabs[1].is_empty(),
        "native group split moved tabs or populated the new group"
    );
    if after.active != Some(*right) || view.group_shown(*right, cx) != Shown::Empty {
        return Ok(None);
    }
    Ok(Some(*right))
}

pub(super) fn close(
    view: &mut SheprdWindow,
    before: &Baseline,
    owned: GroupId,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> Result<()> {
    ensure!(
        created(before, &snapshot(view, cx), view, cx)? == Some(owned),
        "native group cleanup lost its exact owned empty group"
    );
    view.close_group(owned, window, cx);
    Ok(())
}

pub(super) fn restored(before: &Baseline, after: &Baseline) -> bool {
    before == after
}

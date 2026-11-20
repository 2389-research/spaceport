// ABOUTME: Prepares saved palette state before native popup shortcut cases.
// ABOUTME: Observes real search focus and resumed state without injecting measured input.
use super::{Outcome, Page, SheprdWindow};
use anyhow::{Context as _, Result, ensure};
use gpui::{App, Context, Window};

pub(super) fn close(
    view: &mut SheprdWindow,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> Result<()> {
    view.dismiss_menu(window, cx);
    ensure!(
        view.menu.page.is_none(),
        "native popup preparation could not close the previous page"
    );
    window.focus(&view.focus, cx);
    Ok(())
}

pub(super) fn start_resume(
    view: &mut SheprdWindow,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> Result<()> {
    close(view, window, cx)?;
    view.open_palette(crate::palette::Filter::Commands, window, cx);
    let search = view
        .menu
        .palette
        .as_ref()
        .context("resume preparation did not open a palette")?
        .search
        .clone();
    search.update(cx, |input, cx| input.set_text_selected("font", cx));
    view.filter_palette("font", cx);
    Ok(())
}

pub(super) fn finish_resume(
    view: &mut SheprdWindow,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> Result<bool> {
    ensure!(
        view.menu.page == Some(Page::Palette),
        "resume preparation lost its palette"
    );
    let palette = view
        .menu
        .palette
        .as_ref()
        .context("resume preparation lost its palette state")?;
    if palette.search.read(cx).text() != "font" {
        return Ok(false);
    }
    let expected = palette.resume();
    close(view, window, cx)?;
    ensure!(
        view.menu.palette_resume.as_ref() == Some(&expected),
        "resume preparation did not save the actual palette state"
    );
    Ok(true)
}

pub(super) fn observed(
    view: &SheprdWindow,
    saved: Option<&crate::palette::Resume>,
    outcome: Outcome,
    window: &Window,
    cx: &App,
) -> Result<bool> {
    match outcome {
        Outcome::Keybindings => Ok(view.menu.page == Some(Page::Keybinds)
            && view.menu.keybinds_search.as_ref().is_some_and(|search| {
                let input = search.read(cx);
                input.focus.is_focused(window) && input.text().is_empty()
            })),
        Outcome::Palette | Outcome::ResumePalette => {
            if view.menu.page != Some(Page::Palette) {
                return Ok(false);
            }
            let palette = view
                .menu
                .palette
                .as_ref()
                .context("native palette page has no palette state")?;
            let search = palette.search.read(cx);
            if !search.focus.is_focused(window) {
                return Ok(false);
            }
            if outcome == Outcome::ResumePalette {
                let expected = saved.context("native resume baseline is missing")?;
                Ok(search.text() == "font" && palette.resume() == *expected)
            } else {
                Ok(search.text().is_empty())
            }
        }
        _ => anyhow::bail!("non-popup outcome reached the popup observer"),
    }
}

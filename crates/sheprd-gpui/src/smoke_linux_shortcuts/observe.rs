// ABOUTME: Observes real native shortcut outcomes without changing the measured scene.
// ABOUTME: Checks exact runtime identities, focus, pane bytes, and presentation results.
use super::{
    Baseline, Outcome, Page, SheprdWindow, baseline, capture, groups, numbered_tab, panes, popups,
    same_members, same_runtime, same_scene, same_terminal, same_terminal_state, settings_fixture,
    stepped_tab,
};
use anyhow::{Context as _, Result, ensure};
use gpui::{App, Window};

pub(super) fn observed(
    view: &SheprdWindow,
    before: &Baseline,
    outcome: Outcome,
    nonce: &str,
    id: &str,
    window: &Window,
    cx: &mut App,
) -> Result<bool> {
    let after = baseline(view, window, cx)?;
    if outcome == Outcome::CycleWindows {
        same_scene(before, &after)?;
        same_members(before, &after)?;
        ensure!(
            view.live.status.is_connected(),
            "native cycle disconnected the original window"
        );
        let owned = before
            .cycle
            .as_ref()
            .context("native cycle observation lost its owned window")?;
        return owned.observed(before, cx);
    }
    if outcome == Outcome::Settings {
        same_runtime(before, &after)?;
        same_terminal_state(before, &after)?;
        let Some(handle) = settings_fixture::window(cx) else {
            return Ok(false);
        };
        ensure!(
            !before.windows.contains(&handle),
            "native Settings case began with its window open"
        );
        ensure!(
            after.windows.len() == before.windows.len() + 1
                && before.windows.iter().all(|old| after.windows.contains(old)),
            "native Settings action changed unexpected windows"
        );
        let active = settings_fixture::active(cx)?;
        if active {
            eprintln!(
                "LINUX_KEYS_CASE id={id} settings_window_active=true original_windows_preserved=true"
            );
        }
        return Ok(active);
    }
    same_scene(before, &after)?;
    match outcome {
        Outcome::Back
        | Outcome::Forward
        | Outcome::NextPane
        | Outcome::PreviousPane
        | Outcome::Left
        | Outcome::Down
        | Outcome::Up
        | Outcome::Right => panes::observed(view, before, &after, id, window),
        Outcome::CycleWindows => anyhow::bail!("native cycle bypassed its window observer"),
        Outcome::SplitEditor => {
            same_terminal_state(before, &after)?;
            let created = groups::created(&before.groups, &after.groups, view, cx)?;
            if let Some(group) = created {
                eprintln!(
                    "LINUX_KEYS_CASE id={id} empty_right_group={group:?} active=true runtime_members_preserved=true"
                );
            }
            Ok(created.is_some())
        }
        Outcome::Settings => anyhow::bail!("Settings outcome bypassed its window observer"),
        Outcome::Fullscreen(expected) => {
            same_members(before, &after)?;
            ensure!(
                before.focused_tab == after.focused_tab
                    && before.focused_pane == after.focused_pane,
                "native fullscreen key changed the focused terminal"
            );
            ensure!(
                before.fullscreen != expected,
                "native fullscreen case began at its target"
            );
            let done = after.fullscreen == expected && view.focus.is_focused(window);
            if done {
                eprintln!(
                    "LINUX_KEYS_CASE id={id} fullscreen_before={} fullscreen_after={}",
                    before.fullscreen, after.fullscreen
                );
            }
            Ok(done)
        }
        Outcome::NextRegion | Outcome::PreviousRegion => {
            same_terminal(before, &after)?;
            let expected = if outcome == Outcome::NextRegion {
                crate::window::chrome_focus::Region::TitleBar
            } else {
                crate::window::chrome_focus::Region::StatusBar
            };
            let done = view.menu.page.is_none()
                && view.chrome_region == Some(expected)
                && window.focused(cx).is_some()
                && !view.chrome_entry_focus.is_focused(window)
                && !view.focus.is_focused(window);
            if done {
                eprintln!("LINUX_KEYS_CASE id={id} region={expected:?} control_focused=true");
            }
            Ok(done)
        }
        Outcome::Keybindings | Outcome::Palette | Outcome::ResumePalette => {
            same_terminal(before, &after)?;
            popups::observed(view, before.palette_resume.as_ref(), outcome, window, cx)
        }
        Outcome::IncreaseFont | Outcome::DecreaseFont | Outcome::ResetFont => {
            same_terminal(before, &after)?;
            ensure!(
                after.configured_font == before.configured_font,
                "native font key changed the configured baseline"
            );
            ensure!(
                view.focus.is_focused(window),
                "native font key moved terminal focus"
            );
            let expected = match outcome {
                Outcome::IncreaseFont => before.font_size + crate::config::FONT_SIZE_STEP,
                Outcome::DecreaseFont => before.font_size - crate::config::FONT_SIZE_STEP,
                _ => before.configured_font,
            };
            ensure!(
                before.font_size != expected,
                "native font case has no size change to observe"
            );
            if after.font_size != expected || after.cell_height == before.cell_height {
                return Ok(false);
            }
            ensure!(
                after.cell_height > 0,
                "native font key produced an empty cell height"
            );
            eprintln!(
                "LINUX_KEYS_CASE id={id} font_before={} font_after={} cell_height_before={} cell_height_after={}",
                before.font_size, after.font_size, before.cell_height, after.cell_height
            );
            Ok(true)
        }
        Outcome::NewTab => {
            ensure!(
                after
                    .tabs
                    .iter()
                    .filter(|id| before.tabs.contains(id))
                    .count()
                    == before.tabs.len(),
                "native new-tab action lost an existing tab"
            );
            ensure!(
                after.tabs.len() <= before.tabs.len() + 1,
                "native new-tab action created extra tabs"
            );
            if after.tabs.len() != before.tabs.len() + 1 {
                return Ok(false);
            }
            Ok(!before.tabs.contains(&after.focused_tab)
                && after.workspace_tabs.contains(&after.focused_tab))
        }
        Outcome::Tab(_) | Outcome::NextTab | Outcome::PreviousTab => {
            same_members(before, &after)?;
            let target = match outcome {
                Outcome::Tab(number) => numbered_tab(before, number)?,
                _ => stepped_tab(before, outcome == Outcome::NextTab)?,
            };
            ensure!(
                before.focused_tab != target,
                "native tab case began on its target"
            );
            if after.focused_tab != target {
                return Ok(false);
            }
            ensure!(
                after.revision > before.revision,
                "native tab focus has no new daemon revision"
            );
            eprintln!(
                "LINUX_KEYS_CASE id={id} before={:.80} target={target:.80} after={:.80}",
                before.focused_tab, after.focused_tab
            );
            Ok(true)
        }
        Outcome::PolishSearch => {
            same_members(before, &after)?;
            ensure!(
                view.menu.page == Some(Page::Themes),
                "Polish input replaced the owned search popup"
            );
            let search = view
                .menu
                .themes
                .as_ref()
                .and_then(|picker| picker.search())
                .context("Polish search field is missing")?;
            Ok(search.read(cx).text() == "łŃ")
        }
        Outcome::TerminalBytes(expected) => {
            same_terminal(before, &after)?;
            ensure!(
                view.focus.is_focused(window),
                "native pane key moved focus away from the terminal"
            );
            match capture::read(&capture::screen(view)?, nonce, id, expected.len()) {
                capture::Capture::Pending => Ok(false),
                capture::Capture::Malformed => {
                    anyhow::bail!("native pane capture is malformed for {id}")
                }
                capture::Capture::Complete(bytes) => {
                    ensure!(
                        bytes == expected,
                        "native pane bytes differ for {id}: {bytes:02x?}"
                    );
                    eprintln!(
                        "LINUX_KEYS_CASE id={id} bytes={bytes:02x?} count={} pane={:.80}",
                        bytes.len(),
                        after.focused_pane
                    );
                    Ok(true)
                }
            }
        }
    }
}

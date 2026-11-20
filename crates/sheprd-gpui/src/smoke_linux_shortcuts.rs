// ABOUTME: Observes real Linux shortcut and text outcomes against an isolated Herdr.
// ABOUTME: Prepares each case before READY and never injects measured keyboard input.
use crate::settings_window::shortcut_fixture as settings_fixture;
use crate::{Command, SheprdWindow, menu::Page, smoke};
use anyhow::{Context as _, Result, ensure};
use gpui::*;
use std::time::{Duration, Instant};

#[path = "../../test-support/linux_shortcuts.rs"]
mod cases;
use cases::{CASES, Notice, Outcome, notice};
#[path = "smoke_linux_shortcuts/capture.rs"]
mod capture;
#[path = "smoke_linux_shortcuts/groups.rs"]
mod groups;
#[path = "smoke_linux_shortcuts/observe.rs"]
mod observe;
use observe::observed;
#[path = "smoke_linux_shortcuts/popups.rs"]
mod popups;
#[path = "smoke_linux_shortcuts/windows.rs"]
mod windows;

struct Baseline {
    boot: String,
    cell_height: u32,
    configured_font: f32,
    cycle: Option<windows::Owned>,
    focused_pane: String,
    focused_tab: String,
    font_size: f32,
    fullscreen: bool,
    groups: groups::Baseline,
    palette_resume: Option<crate::palette::Resume>,
    panes: Vec<String>,
    revision: u64,
    tabs: Vec<String>,
    windows: Vec<AnyWindowHandle>,
    workspace: String,
    workspace_tabs: Vec<String>,
    workspaces: Vec<String>,
}

enum Stage {
    // keep-sorted start
    AwaitCaptureReady {
        baseline: Baseline,
        index: usize,
    },
    AwaitCaptureTab {
        baseline: Baseline,
        index: usize,
    },
    AwaitCreation {
        baseline: Baseline,
        index: usize,
    },
    AwaitCycle {
        baseline: Baseline,
        index: usize,
        opening: windows::Opening,
    },
    AwaitCycleFocus {
        baseline: Baseline,
        index: usize,
        owned: windows::Owned,
    },
    AwaitFocus {
        baseline: Baseline,
        index: usize,
        target: String,
    },
    AwaitGroupClose {
        baseline: Baseline,
        index: usize,
    },
    AwaitResume(usize),
    AwaitSettingsClose {
        handle: AnyWindowHandle,
        index: usize,
    },
    AwaitWindowClose {
        baseline: Baseline,
        index: usize,
        owned: windows::Owned,
    },
    CloseGroup {
        baseline: Baseline,
        group: crate::browser::GroupId,
        index: usize,
    },
    CloseWindow {
        baseline: Baseline,
        index: usize,
        owned: windows::Owned,
    },
    Mapped,
    Mapping,
    Observe {
        baseline: Baseline,
        index: usize,
        started: Instant,
    },
    Prepare(usize),
    Ready {
        baseline: Baseline,
        index: usize,
    },
    // keep-sorted end
}

enum Step {
    Continue(Stage),
    Done,
    Paint(Stage),
}

pub(super) fn start(handle: WindowHandle<SheprdWindow>, nonce: String, cx: &mut App) {
    smoke::set_exit_code(1);
    cx.spawn(async move |cx| {
        let result = run(handle, &nonce, cx).await;
        smoke::conclude(
            "LINUX_KEYS native",
            "real daemon tabs, shifted digits, pane key bytes, and Polish search text",
            result,
        );
    })
    .detach();
}

fn coherent(view: &SheprdWindow) -> bool {
    view.live.status.is_connected()
        && !view.live.activation.fenced()
        && view.navigation_ready()
        && view
            .live
            .snapshot
            .as_ref()
            .zip(view.live.surface.as_ref())
            .is_some_and(|(snapshot, surface)| {
                snapshot.boot_id == surface.boot_id
                    && snapshot.revision == surface.projection_revision
                    && snapshot
                        .focused_tab_id
                        .as_ref()
                        .is_some_and(|id| !id.is_empty())
                    && snapshot
                        .focused_workspace_id
                        .as_ref()
                        .is_some_and(|id| !id.is_empty())
            })
}

fn baseline(view: &SheprdWindow, window: &Window, cx: &App) -> Result<Baseline> {
    let snapshot = view
        .live
        .snapshot
        .as_ref()
        .context("native snapshot is missing")?;
    let mut tabs = snapshot
        .tabs
        .iter()
        .map(|tab| tab.tab_id.clone())
        .collect::<Vec<_>>();
    tabs.sort();
    let mut workspaces = snapshot
        .workspaces
        .iter()
        .map(|workspace| workspace.workspace_id.clone())
        .collect::<Vec<_>>();
    workspaces.sort();
    let mut panes = snapshot
        .panes
        .iter()
        .map(|pane| pane.pane_id.clone())
        .collect::<Vec<_>>();
    panes.sort();
    Ok(Baseline {
        boot: snapshot.boot_id.clone(),
        cell_height: view.options.cell_height_px,
        configured_font: view.configured_terminal_size,
        cycle: None,
        focused_pane: snapshot
            .focused_pane_id
            .clone()
            .context("native focused pane is missing")?,
        focused_tab: snapshot
            .focused_tab_id
            .clone()
            .context("native focused tab is missing")?,
        font_size: view.config.terminal.size,
        fullscreen: window.is_fullscreen(),
        groups: groups::snapshot(view, cx),
        palette_resume: view.menu.palette_resume.clone(),
        panes,
        revision: snapshot.revision,
        tabs,
        windows: cx.windows(),
        workspace: snapshot
            .focused_workspace_id
            .clone()
            .context("native workspace is missing")?,
        workspace_tabs: snapshot
            .tabs
            .iter()
            .filter(|tab| Some(&tab.workspace_id) == snapshot.focused_workspace_id.as_ref())
            .map(|tab| tab.tab_id.clone())
            .collect(),
        workspaces,
    })
}

fn search_ready(view: &SheprdWindow, window: &Window, cx: &App) -> bool {
    view.menu.page == Some(Page::Themes)
        && view
            .menu
            .themes
            .as_ref()
            .and_then(|picker| picker.search())
            .is_some_and(|search| {
                let search = search.read(cx);
                search.focus.is_focused(window) && search.text().is_empty()
            })
}

fn same_scene(before: &Baseline, after: &Baseline) -> Result<()> {
    same_runtime(before, after)?;
    ensure!(
        after.windows == before.windows,
        "native case changed window identities"
    );
    Ok(())
}

fn same_runtime(before: &Baseline, after: &Baseline) -> Result<()> {
    ensure!(after.boot == before.boot, "native case changed daemon boot");
    ensure!(
        after.workspace == before.workspace && after.workspaces == before.workspaces,
        "native case changed workspace identity"
    );
    Ok(())
}

fn same_members(before: &Baseline, after: &Baseline) -> Result<()> {
    ensure!(
        after.panes == before.panes,
        "native case changed pane identities"
    );
    ensure!(
        after.tabs == before.tabs,
        "native case changed tab identities"
    );
    ensure!(
        after.workspace_tabs == before.workspace_tabs,
        "native case changed workspace tab order"
    );
    Ok(())
}

fn same_terminal(before: &Baseline, after: &Baseline) -> Result<()> {
    same_scene(before, after)?;
    same_terminal_state(before, after)
}

fn same_terminal_state(before: &Baseline, after: &Baseline) -> Result<()> {
    same_members(before, after)?;
    ensure!(
        before.focused_tab == after.focused_tab && before.focused_pane == after.focused_pane,
        "native pane key changed the focused terminal"
    );
    ensure!(
        before.fullscreen == after.fullscreen,
        "native pane key changed fullscreen state"
    );
    Ok(())
}

fn numbered_tab(before: &Baseline, number: u8) -> Result<&str> {
    cases::tab_at_position(&before.workspace_tabs, number)
        .context("native numbered tab is missing or zero")
}

fn stepped_tab(before: &Baseline, forward: bool) -> Result<&str> {
    let tabs = &before.workspace_tabs;
    ensure!(tabs.len() >= 2, "native tab step needs distinct targets");
    let current = tabs
        .iter()
        .position(|id| id == &before.focused_tab)
        .context("native current tab is outside the workspace")?;
    let target = (current + if forward { 1 } else { tabs.len() - 1 }) % tabs.len();
    Ok(&tabs[target])
}

fn advance(
    stage: Stage,
    nonce: &str,
    view: &mut SheprdWindow,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> Result<Step> {
    if let Stage::Observe { started, index, .. } = &stage {
        ensure!(
            started.elapsed() < Duration::from_secs(12),
            "native case {} timed out",
            CASES[*index].id
        );
    }
    ensure!(
        view.notices.is_none() && view.live.notices.is_none(),
        "native case has a client notice"
    );
    // Cross-window preparation must progress while the original window is inactive.
    let stage = match stage {
        Stage::AwaitCycle {
            baseline: before,
            index,
            opening,
        } => {
            let Some(owned) = opening.poll(&before, cx)? else {
                return Ok(Step::Continue(Stage::AwaitCycle {
                    baseline: before,
                    index,
                    opening,
                }));
            };
            let after = baseline(view, window, cx)?;
            same_runtime(&before, &after)?;
            same_members(&before, &after)?;
            owned.activate_original(cx);
            return Ok(Step::Continue(Stage::AwaitCycleFocus {
                baseline: before,
                index,
                owned,
            }));
        }
        Stage::AwaitCycleFocus {
            baseline: before,
            index,
            owned,
        } => {
            if !window.is_window_active() || !coherent(view) || !owned.inactive(cx)? {
                return Ok(Step::Continue(Stage::AwaitCycleFocus {
                    baseline: before,
                    index,
                    owned,
                }));
            }
            let mut current = baseline(view, window, cx)?;
            same_runtime(&before, &current)?;
            same_members(&before, &current)?;
            owned.check_addition(&before, &current.windows)?;
            current.cycle = Some(owned);
            return Ok(Step::Paint(Stage::Ready {
                baseline: current,
                index,
            }));
        }
        Stage::CloseWindow {
            baseline,
            index,
            owned,
        } => {
            owned.request_close(cx);
            return Ok(Step::Continue(Stage::AwaitWindowClose {
                baseline,
                index,
                owned,
            }));
        }
        Stage::AwaitWindowClose {
            baseline: before,
            index,
            owned,
        } => {
            if !owned.closed(cx) {
                return Ok(Step::Continue(Stage::AwaitWindowClose {
                    baseline: before,
                    index,
                    owned,
                }));
            }
            let after = baseline(view, window, cx)?;
            same_runtime(&before, &after)?;
            same_members(&before, &after)?;
            owned.check_removal(&before, &after)?;
            owned.activate_original(cx);
            eprintln!(
                "LINUX_KEYS_CASE owned_second_window_closed=true before_case={}",
                CASES[index].id
            );
            return Ok(Step::Continue(Stage::Prepare(index)));
        }
        stage => stage,
    };
    let cycling = matches!(&stage, Stage::Observe { index, .. } if CASES[*index].outcome == Outcome::CycleWindows);
    if (!cycling && !coherent(view)) || view.bounds.size.width <= px(0.) {
        return Ok(Step::Continue(stage));
    }
    if !cycling {
        view.live
            .surface
            .as_ref()
            .context("native surface is missing")?
            .frame
            .validate()?;
    }
    match stage {
        Stage::AwaitCycle { .. }
        | Stage::AwaitCycleFocus { .. }
        | Stage::CloseWindow { .. }
        | Stage::AwaitWindowClose { .. } => {
            anyhow::bail!("native cross-window stage bypassed its owner")
        }
        Stage::CloseGroup {
            baseline,
            group,
            index,
        } => {
            groups::close(view, &baseline.groups, group, window, cx)?;
            Ok(Step::Paint(Stage::AwaitGroupClose { baseline, index }))
        }
        Stage::AwaitGroupClose {
            baseline: before,
            index,
        } => {
            let after = baseline(view, window, cx)?;
            same_scene(&before, &after)?;
            same_terminal_state(&before, &after)?;
            if !groups::restored(&before.groups, &after.groups) {
                return Ok(Step::Continue(Stage::AwaitGroupClose {
                    baseline: before,
                    index,
                }));
            }
            eprintln!(
                "LINUX_KEYS_CASE empty_group_closed=true before_case={}",
                CASES[index].id
            );
            Ok(Step::Continue(Stage::Prepare(index)))
        }
        Stage::AwaitSettingsClose { handle, index } => {
            if cx.windows().contains(&handle) {
                return Ok(Step::Continue(Stage::AwaitSettingsClose { handle, index }));
            }
            eprintln!(
                "LINUX_KEYS_CASE settings_window_closed=true before_case={}",
                CASES[index].id
            );
            window.activate_window();
            Ok(Step::Continue(Stage::Prepare(index)))
        }
        Stage::AwaitResume(index) => {
            if !popups::finish_resume(view, window, cx)? {
                return Ok(Step::Continue(Stage::AwaitResume(index)));
            }
            Ok(Step::Paint(Stage::Ready {
                baseline: baseline(view, window, cx)?,
                index,
            }))
        }
        Stage::AwaitCaptureTab {
            baseline: before,
            index,
        } => {
            let case = &CASES[index];
            if !observed(view, &before, Outcome::NewTab, nonce, case.id, window, cx)? {
                return Ok(Step::Continue(Stage::AwaitCaptureTab {
                    baseline: before,
                    index,
                }));
            }
            let Outcome::TerminalBytes(bytes) = case.outcome else {
                anyhow::bail!("capture stage has a non-terminal case");
            };
            // Each capture owns a fresh shell, so terminal keyboard modes cannot leak between cases.
            capture::start(view, nonce, case.id, bytes.len())?;
            Ok(Step::Continue(Stage::AwaitCaptureReady {
                baseline: baseline(view, window, cx)?,
                index,
            }))
        }
        Stage::AwaitCaptureReady {
            baseline: before,
            index,
        } => {
            let case = &CASES[index];
            same_terminal(&before, &baseline(view, window, cx)?)?;
            let screen = capture::screen(view)?;
            ensure!(
                !screen.contains(&capture::marker(nonce, case.id, "DONE")),
                "capture finished before native input"
            );
            if !screen.contains(&capture::marker(nonce, case.id, "READY")) {
                return Ok(Step::Continue(Stage::AwaitCaptureReady {
                    baseline: before,
                    index,
                }));
            }
            window.focus(&view.focus, cx);
            Ok(Step::Paint(Stage::Ready {
                baseline: baseline(view, window, cx)?,
                index,
            }))
        }
        Stage::AwaitCreation { baseline, index } => {
            if !observed(
                view,
                &baseline,
                Outcome::NewTab,
                nonce,
                CASES[index].id,
                window,
                cx,
            )? {
                return Ok(Step::Continue(Stage::AwaitCreation { baseline, index }));
            }
            Ok(Step::Continue(Stage::Prepare(index)))
        }
        Stage::AwaitFocus {
            baseline: before,
            index,
            target,
        } => {
            let after = baseline(view, window, cx)?;
            same_scene(&before, &after)?;
            same_members(&before, &after)?;
            if after.focused_tab != target {
                return Ok(Step::Continue(Stage::AwaitFocus {
                    baseline: before,
                    index,
                    target,
                }));
            }
            ensure!(
                after.revision > before.revision,
                "preparation focus has no new daemon revision"
            );
            Ok(Step::Continue(Stage::Prepare(index)))
        }
        Stage::Mapping => {
            window.focus(&view.focus, cx);
            Ok(Step::Paint(Stage::Mapped))
        }
        Stage::Mapped => {
            eprintln!("{}", notice(nonce, Notice::Mapped));
            Ok(Step::Continue(Stage::Prepare(0)))
        }
        Stage::Prepare(index) => {
            if let Some(handle) = settings_fixture::window(cx) {
                settings_fixture::request_close(cx)?;
                return Ok(Step::Continue(Stage::AwaitSettingsClose { handle, index }));
            }
            if !window.is_window_active() {
                return Ok(Step::Continue(Stage::Prepare(index)));
            }
            let case = &CASES[index];
            if case.chords.first().copied() == Some(cases::PREFIX) {
                ensure!(
                    view.keymap
                        .is_prefix(&crate::keymap::parse_keystroke("ctrl-b")?),
                    "native prefix differs from the resolved private keymap"
                );
            }
            if matches!(
                case.outcome,
                Outcome::IncreaseFont | Outcome::DecreaseFont | Outcome::ResetFont
            ) {
                if view.config_load.is_some() {
                    return Ok(Step::Continue(Stage::Prepare(index)));
                }
                if case.outcome == Outcome::ResetFont
                    && view.config.terminal.size == view.configured_terminal_size
                {
                    view.command(Command::IncreaseFontSize, window, cx);
                    view.command(Command::IncreaseFontSize, window, cx);
                    return Ok(Step::Paint(Stage::Prepare(index)));
                }
            }
            let baseline = baseline(view, window, cx)?;
            match case.outcome {
                Outcome::CycleWindows => {
                    popups::close(view, window, cx)?;
                    let opening = windows::open(view, window, cx)?;
                    return Ok(Step::Continue(Stage::AwaitCycle {
                        baseline,
                        index,
                        opening,
                    }));
                }
                Outcome::SplitEditor => {
                    groups::prepared(&baseline.groups)?;
                    popups::close(view, window, cx)?;
                }
                Outcome::Settings => popups::close(view, window, cx)?,
                Outcome::Fullscreen(expected) => {
                    ensure!(
                        baseline.fullscreen != expected,
                        "native fullscreen preparation is already at the target"
                    );
                    window.focus(&view.focus, cx);
                }
                Outcome::NextRegion | Outcome::PreviousRegion => {
                    ensure!(
                        !view.zen_layout(),
                        "native region preparation requires visible chrome"
                    );
                    if !view.status_bar_visible {
                        view.command(Command::ToggleStatusBar, window, cx);
                    }
                    window.focus(&view.focus, cx);
                }
                Outcome::NextTab | Outcome::PreviousTab => {
                    stepped_tab(&baseline, case.outcome == Outcome::NextTab)?;
                    window.focus(&view.focus, cx);
                }
                Outcome::Keybindings | Outcome::Palette => popups::close(view, window, cx)?,
                Outcome::ResumePalette => {
                    popups::start_resume(view, window, cx)?;
                    return Ok(Step::Paint(Stage::AwaitResume(index)));
                }
                Outcome::IncreaseFont | Outcome::DecreaseFont | Outcome::ResetFont => {
                    ensure!(
                        baseline.cell_height > 0,
                        "native font baseline has no cell height"
                    );
                    let range = &crate::config::FONT_SIZE_RANGE;
                    ensure!(
                        baseline.font_size > *range.start() && baseline.font_size < *range.end(),
                        "native font baseline is clamped"
                    );
                    window.focus(&view.focus, cx);
                }
                Outcome::NewTab => window.focus(&view.focus, cx),
                Outcome::Tab(number) => {
                    let required = CASES
                        .iter()
                        .filter_map(|case| match case.outcome {
                            Outcome::Tab(number) => Some(usize::from(number)),
                            _ => None,
                        })
                        .max()
                        .unwrap_or_default()
                        .max(2);
                    if baseline.workspace_tabs.len() < required {
                        view.command(Command::Tab, window, cx);
                        return Ok(Step::Continue(Stage::AwaitCreation { baseline, index }));
                    }
                    ensure!(
                        baseline.workspace_tabs.len() == required,
                        "native tab scene has unexpected extra tabs"
                    );
                    if baseline.focused_tab == numbered_tab(&baseline, number)? {
                        let starting_number = if number == 1 { 2 } else { 1 };
                        let target = numbered_tab(&baseline, starting_number)?.to_owned();
                        view.command(Command::TabNumber(starting_number), window, cx);
                        return Ok(Step::Continue(Stage::AwaitFocus {
                            baseline,
                            index,
                            target,
                        }));
                    }
                    window.focus(&view.focus, cx);
                }
                Outcome::PolishSearch => view.open_theme_picker(window, cx),
                Outcome::TerminalBytes(_) => {
                    view.command(Command::Tab, window, cx);
                    return Ok(Step::Continue(Stage::AwaitCaptureTab { baseline, index }));
                }
            }
            eprintln!(
                "LINUX_KEYS_CASE id={} expected={:?} physical_chords={}",
                case.id,
                case.outcome,
                case.chords.len()
            );
            Ok(Step::Paint(Stage::Ready { baseline, index }))
        }
        Stage::Ready {
            baseline: before,
            index,
        } => {
            let case = &CASES[index];
            let focused = match case.outcome {
                Outcome::CycleWindows => {
                    let current = baseline(view, window, cx)?;
                    same_scene(&before, &current)?;
                    same_members(&before, &current)?;
                    let owned = before
                        .cycle
                        .as_ref()
                        .context("native cycle case has no owned target window")?;
                    view.menu.page.is_none()
                        && view.focus.is_focused(window)
                        && owned.inactive(cx)?
                }
                Outcome::SplitEditor => {
                    let current = baseline(view, window, cx)?;
                    same_terminal(&before, &current)?;
                    ensure!(
                        groups::restored(&before.groups, &current.groups),
                        "native group scene changed before READY"
                    );
                    view.menu.page.is_none() && view.focus.is_focused(window)
                }
                Outcome::Settings => {
                    settings_fixture::window(cx).is_none()
                        && view.menu.page.is_none()
                        && view.focus.is_focused(window)
                }
                Outcome::Fullscreen(_)
                | Outcome::NextRegion
                | Outcome::PreviousRegion
                | Outcome::NextTab
                | Outcome::PreviousTab => {
                    same_terminal(&before, &baseline(view, window, cx)?)?;
                    view.chrome_region.is_none() && view.focus.is_focused(window)
                }
                Outcome::Keybindings | Outcome::Palette | Outcome::ResumePalette => {
                    same_terminal(&before, &baseline(view, window, cx)?)?;
                    ensure!(
                        view.menu.page.is_none(),
                        "native popup case opened before READY"
                    );
                    view.focus.is_focused(window)
                }
                Outcome::IncreaseFont | Outcome::DecreaseFont | Outcome::ResetFont => {
                    let current = baseline(view, window, cx)?;
                    same_terminal(&before, &current)?;
                    ensure!(
                        view.config_load.is_none()
                            && current.font_size == before.font_size
                            && current.cell_height == before.cell_height,
                        "native font baseline changed before READY"
                    );
                    view.focus.is_focused(window)
                }
                Outcome::NewTab => view.focus.is_focused(window),
                Outcome::Tab(number) => {
                    let current = baseline(view, window, cx)?;
                    same_scene(&before, &current)?;
                    same_members(&before, &current)?;
                    ensure!(
                        current.focused_tab == before.focused_tab
                            && current.focused_tab != numbered_tab(&before, number)?,
                        "native tab preparation changed before READY"
                    );
                    view.focus.is_focused(window)
                }
                Outcome::PolishSearch => search_ready(view, window, cx),
                Outcome::TerminalBytes(_) => {
                    same_terminal(&before, &baseline(view, window, cx)?)?;
                    let screen = capture::screen(view)?;
                    ensure!(
                        screen.contains(&capture::marker(nonce, case.id, "READY"))
                            && !screen.contains(&capture::marker(nonce, case.id, "DONE")),
                        "native pane capture changed before READY"
                    );
                    view.focus.is_focused(window)
                }
            };
            if !window.is_window_active() || !focused {
                return Ok(Step::Continue(Stage::Ready {
                    baseline: before,
                    index,
                }));
            }
            eprintln!("{}", notice(nonce, Notice::Ready(case.id)));
            Ok(Step::Continue(Stage::Observe {
                baseline: before,
                index,
                started: Instant::now(),
            }))
        }
        Stage::Observe {
            baseline,
            index,
            started,
        } => {
            let case = &CASES[index];
            if (!matches!(case.outcome, Outcome::Settings | Outcome::CycleWindows)
                && !window.is_window_active())
                || !observed(view, &baseline, case.outcome, nonce, case.id, window, cx)?
            {
                return Ok(Step::Continue(Stage::Observe {
                    baseline,
                    index,
                    started,
                }));
            }
            eprintln!("{}", notice(nonce, Notice::Passed(case.id)));
            if index + 1 == CASES.len() {
                eprintln!("{}", notice(nonce, Notice::Complete));
                Ok(Step::Done)
            } else {
                if case.outcome == Outcome::CycleWindows {
                    let mut baseline = baseline;
                    let owned = baseline
                        .cycle
                        .take()
                        .context("native cycle cleanup lost its owned window")?;
                    return Ok(Step::Continue(Stage::CloseWindow {
                        baseline,
                        index: index + 1,
                        owned,
                    }));
                }
                if case.outcome == Outcome::SplitEditor {
                    let group = view
                        .group_slots()
                        .get(1)
                        .map(|slot| slot.id)
                        .context("native created group is missing after success")?;
                    return Ok(Step::Continue(Stage::CloseGroup {
                        baseline,
                        group,
                        index: index + 1,
                    }));
                }
                Ok(Step::Continue(Stage::Prepare(index + 1)))
            }
        }
    }
}

async fn run(handle: WindowHandle<SheprdWindow>, nonce: &str, cx: &mut AsyncApp) -> Result<()> {
    ensure!(cases::valid_nonce(nonce), "native fixture nonce is invalid");
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut stage = Stage::Mapping;
    loop {
        ensure!(
            Instant::now() < deadline,
            "native Linux fixture exceeded its deadline"
        );
        cx.background_executor()
            .timer(Duration::from_millis(50))
            .await;
        let next = handle.update(cx, |view, window, cx| {
            advance(stage, nonce, view, window, cx)
        })??;
        stage = match next {
            Step::Continue(next) => next,
            Step::Paint(next) => {
                // Preparation completes before READY; observed input uses normal native delivery.
                AnyWindowHandle::from(handle).update(cx, |_, window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                })?;
                next
            }
            Step::Done => return Ok(()),
        };
    }
}

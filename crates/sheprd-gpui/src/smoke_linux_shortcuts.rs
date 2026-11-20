// ABOUTME: Observes real Linux shortcut and text outcomes against an isolated Herdr.
// ABOUTME: Prepares each case before READY and never injects measured keyboard input.
use crate::{SheprdWindow, menu::Page, smoke};
use anyhow::{Context as _, Result, ensure};
use gpui::*;
use std::time::{Duration, Instant};

#[path = "../../test-support/linux_shortcuts.rs"]
mod cases;
use cases::{CASES, Notice, Outcome, notice};

struct Baseline {
    boot: String,
    tabs: Vec<String>,
    windows: usize,
    workspace: String,
    workspaces: Vec<String>,
}

enum Stage {
    // keep-sorted start
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
            "real daemon tab creation and Polish search text",
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

fn baseline(view: &SheprdWindow, cx: &App) -> Result<Baseline> {
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
    Ok(Baseline {
        boot: snapshot.boot_id.clone(),
        tabs,
        windows: cx.windows().len(),
        workspace: snapshot
            .focused_workspace_id
            .clone()
            .context("native workspace is missing")?,
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

fn observed(view: &SheprdWindow, before: &Baseline, outcome: Outcome, cx: &App) -> Result<bool> {
    let after = baseline(view, cx)?;
    ensure!(after.boot == before.boot, "native case changed daemon boot");
    ensure!(
        after.workspace == before.workspace && after.workspaces == before.workspaces,
        "native case changed workspace identity"
    );
    ensure!(
        after.windows == before.windows,
        "native case opened an unexpected window"
    );
    match outcome {
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
            if after.tabs.len() != before.tabs.len() + 1 {
                return Ok(false);
            }
            let snapshot = view
                .live
                .snapshot
                .as_ref()
                .context("native tab snapshot is missing")?;
            Ok(snapshot
                .focused_tab_id
                .as_ref()
                .is_some_and(|id| !before.tabs.contains(id)))
        }
        Outcome::PolishSearch => {
            ensure!(
                after.tabs == before.tabs,
                "Polish input changed tab identities"
            );
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
    }
}

fn advance(
    stage: Stage,
    nonce: &str,
    view: &mut SheprdWindow,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> Result<Step> {
    ensure!(
        view.notices.is_none() && view.live.notices.is_none(),
        "native case has a client notice"
    );
    if !coherent(view) || view.bounds.size.width <= px(0.) {
        return Ok(Step::Continue(stage));
    }
    view.live
        .surface
        .as_ref()
        .context("native surface is missing")?
        .frame
        .validate()?;
    match stage {
        Stage::Mapping => {
            window.focus(&view.focus, cx);
            Ok(Step::Paint(Stage::Mapped))
        }
        Stage::Mapped => {
            eprintln!("{}", notice(nonce, Notice::Mapped));
            Ok(Step::Continue(Stage::Prepare(0)))
        }
        Stage::Prepare(index) => {
            if !window.is_window_active() {
                return Ok(Step::Continue(Stage::Prepare(index)));
            }
            let case = &CASES[index];
            let baseline = baseline(view, cx)?;
            match case.outcome {
                Outcome::NewTab => window.focus(&view.focus, cx),
                Outcome::PolishSearch => view.open_theme_picker(window, cx),
            }
            eprintln!(
                "LINUX_KEYS_CASE id={} expected={:?} physical_chords={}",
                case.id,
                case.outcome,
                case.chords.len()
            );
            Ok(Step::Paint(Stage::Ready { baseline, index }))
        }
        Stage::Ready { baseline, index } => {
            let case = &CASES[index];
            let focused = match case.outcome {
                Outcome::NewTab => view.focus.is_focused(window),
                Outcome::PolishSearch => search_ready(view, window, cx),
            };
            if !window.is_window_active() || !focused {
                return Ok(Step::Continue(Stage::Ready { baseline, index }));
            }
            eprintln!("{}", notice(nonce, Notice::Ready(case.id)));
            Ok(Step::Continue(Stage::Observe {
                baseline,
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
            ensure!(
                started.elapsed() < Duration::from_secs(12),
                "native case {} timed out",
                case.id
            );
            if !window.is_window_active() || !observed(view, &baseline, case.outcome, cx)? {
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

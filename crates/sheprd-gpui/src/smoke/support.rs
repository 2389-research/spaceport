// ABOUTME: Shared scaffolding every native smoke fixture builds its checks on.
// ABOUTME: Keeps the fixtures off the UI thread and reporting one labelled verdict.
//! Native fixture scaffolding: the exit-code plumbing, window targeting, input
//! scripting, and paint assertions the `smoke_*` files share.
//!
//! # The UI thread rules every fixture inherits
//!
//! A fixture runs on the app's only UI thread, which also draws and delivers
//! input, so a blocked thread is a frozen window and a failed fixture.
//!
//! - Never sleep, join a thread, wait on a socket or a child process, or read a
//!   file inside `update`, a paint, or an action. Work that can take time
//!   belongs on `cx.background_spawn` or a worker thread, which hands its result
//!   back as a message that the next `update` applies.
//! - Wait by polling: `wait` and each fixture's step loop re-arm from the async
//!   task and touch the window only inside `update`, between executor timers. A
//!   deadline bounds a run that is already failing; it is never a delay guessed
//!   to be long enough for something to settle.
//! - Script input through `window.dispatch_keystroke` and `dispatch_action`,
//!   with `type_text` for text, and assert the fixture's own `InputProbe`
//!   counters after each step, so a key that reached the wrong handler fails the
//!   step it was typed in rather than the check after it.
//! - Report through `conclude` and `require`, and exit explicitly: AppKit may
//!   `exit(0)` inside `cx.quit()`, which would bypass the code `main` returns.
//!
//! A new fixture is one `smoke_*.rs` file exposing `pub(crate) fn start`, plus
//! its row in the driver's `fixtures!` table. It inherits every rule above.
use crate::{
    Command, LiveState, SheprdWindow,
    sidebar::layout_tests::{PaintedProbes, PaintedText, VerifyChildGeometry},
};
use anyhow::{Context as _, Result, anyhow, bail};
use gpui::*;
use sheprd_client::protocol::*;
use std::{
    sync::atomic::{AtomicU8, Ordering},
    time::{Duration, Instant},
};

pub(crate) static EXIT_CODE: AtomicU8 = AtomicU8::new(0);

/// Prints a fixture's verdict under `label` and exits with it.
///
/// AppKit may `exit(0)` inside `cx.quit()`, which would bypass the code `main`
/// returns, so a daemon-free fixture exits explicitly.
pub(crate) fn conclude(label: &str, message: &str, result: Result<()>) -> ! {
    match result {
        Ok(()) => {
            eprintln!("{label} PASS: {message}");
            EXIT_CODE.store(0, Ordering::SeqCst);
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{label} FAIL: {error:#}");
            EXIT_CODE.store(1, Ordering::SeqCst);
            std::process::exit(1);
        }
    }
}

/// One step of a longer fixture, fatal on its own under its own label so the
/// log names the step that failed.
/// Records this run's exit code, for a fixture that quits through the app
/// instead of exiting at the point it failed.
pub(crate) fn set_exit_code(code: u8) {
    EXIT_CODE.store(code, Ordering::SeqCst);
}

pub(crate) fn require<T>(label: &str, result: Result<T>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{label} FAIL: {error:#}");
            EXIT_CODE.store(1, Ordering::SeqCst);
            std::process::exit(1);
        }
    }
}

/// Runs `check` on the fixture window's root view during one `update`.
///
/// This is the only place a check may read state, draw, or dispatch input: an
/// `update` holds the window and the app for its duration, so nothing inside it
/// may block.
pub(crate) fn with_root<C, T>(
    handle: WindowHandle<SheprdWindow>,
    cx: &mut C,
    check: impl FnOnce(&Entity<SheprdWindow>, &mut Window, &mut App) -> Result<T>,
) -> Result<T>
where
    C: AppContext,
{
    AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<T> {
        let view = root
            .downcast::<SheprdWindow>()
            .map_err(|_| anyhow!("unexpected window root"))?;
        check(&view, window, cx)
    })?
}

/// Draws one frame synchronously, so the checks that follow observe what GPUI
/// just painted rather than waiting for the platform to schedule a frame.
pub(crate) fn draw(window: &mut Window, cx: &mut App) {
    window.refresh();
    window.draw(cx).clear(cx);
}

/// Forgets the previous frame's paint probes, so the next draw's are the only
/// ones a check can see.
pub(crate) fn clear_probes(cx: &mut App) {
    cx.default_global::<PaintedProbes>().0.clear();
}

/// The painted text `label` of the frame just drawn.
pub(crate) fn probe<'a>(cx: &'a App, label: &str) -> Result<&'a PaintedText> {
    cx.try_global::<PaintedProbes>()
        .and_then(|probes| probes.0.get(label))
        .with_context(|| format!("missing paint: {label}"))
}

/// Reports the first paint probe that failed while laying out, if any.
pub(crate) fn check_probes(cx: &mut App) -> Result<()> {
    cx.default_global::<PaintedProbes>().check()
}

/// Stops holding every painted label to the fixed-width child geometry, for the
/// fixtures that add rows of their own.
pub(crate) fn verify_child_geometry(cx: &mut App, verify: bool) {
    cx.set_global(VerifyChildGeometry(verify));
}

fn banner_height() -> f32 {
    if env!("HERDR_BUILD_WORKTREE") == "1" {
        22.
    } else {
        0.
    }
}

/// The fixture viewport at `width` by `height` of content.
///
/// Baseline viewport sizes already include the existing 34px macOS titlebar.
/// Add only the optional banner to preserve the tested content area, not mask
/// clipping.
pub(crate) fn fixture_size(width: f32, height: f32) -> Size<Pixels> {
    size(px(width), px(height + banner_height()))
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct InputProbe {
    pub actions: u64,
    pub keys: u64,
    pub text: u64,
    /// Compositions the platform's input method was told to drop.
    pub compositions_discarded: u64,
}

/// The focus changes the window last asked of the daemon, each with the
/// code that asked, so a step that times out names the request that undid it.
#[derive(Clone, Debug, Default)]
pub(crate) struct Navigations(std::collections::VecDeque<String>);

impl Navigations {
    const KEPT: usize = 12;

    pub(crate) fn record(
        &mut self,
        label: &str,
        focus: Option<&crate::navigation::OwnedNavigationTarget>,
        at: &std::panic::Location<'_>,
    ) {
        if self.0.len() == Self::KEPT {
            self.0.pop_front();
        }
        self.0.push_back(format!(
            "{label} {focus:?} from {}:{}",
            at.file(),
            at.line()
        ));
    }
}

/// What a step types: a command through its default shortcut on this platform,
/// or a key the terminal receives.
#[derive(Clone, Copy)]
pub(crate) enum Key {
    Command(Command),
    Terminal(&'static str),
}

/// This platform's first default shortcut for `command`.
pub(crate) fn shortcut(command: Command) -> Result<&'static str> {
    crate::controls::COMMANDS
        .iter()
        .find(|info| info.command == command)
        .and_then(|info| info.default_shortcuts().first().copied())
        .with_context(|| format!("no default shortcut for {command:?}"))
}

/// Types `text` as native text input, asserting that each character reached the
/// window's own text handler.
pub(crate) fn type_text(
    text: &str,
    view: &Entity<SheprdWindow>,
    window: &mut Window,
    cx: &mut App,
) -> Result<()> {
    for ch in text.chars() {
        let before = view.read(cx).input_probe.text;
        if !window.dispatch_keystroke(
            Keystroke {
                modifiers: Modifiers::default(),
                key: ch.to_string(),
                key_char: Some(ch.to_string()),
            },
            cx,
        ) {
            bail!("unhandled text keystroke {ch:?}");
        }
        if view.read(cx).input_probe.text != before + 1 {
            bail!("text keystroke {ch:?} missed native input handler");
        }
    }
    Ok(())
}

/// Dispatches `name` as one keystroke through the window's keymap.
#[cfg(target_os = "macos")]
pub(crate) fn key(name: &str, window: &mut Window, cx: &mut App) -> Result<()> {
    if !window.dispatch_keystroke(crate::keymap::parse_keystroke(name)?, cx) {
        bail!("unhandled CJK fixture key {name}");
    }
    Ok(())
}

/// Repeats `inspect` on the fixture window until it answers, bounded so a
/// window that never does fails the fixture instead of hanging the run.
#[cfg(target_os = "macos")]
pub(crate) async fn wait<T>(
    handle: WindowHandle<SheprdWindow>,
    cx: &mut AsyncApp,
    label: &str,
    mut inspect: impl FnMut(&Entity<SheprdWindow>, &mut Window, &mut App) -> Result<Option<T>>,
) -> Result<T> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let result = with_root(handle, cx, |view, window, cx| -> Result<_> {
            window.focus(&view.read(cx).focus.clone(), cx);
            draw(window, cx);
            let state = view.read(cx);
            if state.notices.is_some()
                || state.live.notices.is_some()
                || !state.live.status.is_connected()
            {
                bail!(
                    "CJK fixture connection failed: {:?} {:?}",
                    state.notices,
                    state.live.notices
                );
            }
            inspect(view, window, cx)
        })?;
        if let Some(result) = result {
            return Ok(result);
        }
        if Instant::now() >= deadline {
            // This harness only connects to the parent's isolated synthetic shell.
            handle.update(cx, |view, _, _| {
                if let Some(surface) = &view.live.surface {
                    for row in surface.frame.cells.chunks(usize::from(surface.frame.width)) {
                        eprintln!(
                            "fixture row: {:?}",
                            row.iter().map(|c| c.symbol.as_str()).collect::<String>()
                        );
                    }
                }
            })?;
            bail!("timed out waiting for {label}");
        }
        cx.background_executor()
            .timer(Duration::from_millis(50))
            .await;
    }
}

/// Whether the frame drew `marker` as a whole row of its own.
pub(crate) fn has_output(frame: &FrameData, marker: &str) -> bool {
    frame.width > 0
        && frame.cells.chunks(usize::from(frame.width)).any(|row| {
            row.iter()
                .map(|cell| cell.symbol.as_str())
                .collect::<String>()
                // The daemon paints a scrollbar after the terminal's last column
                // once this fixture has produced more than a screen of output.
                .trim_end_matches(['▕', '▐'])
                .trim()
                == marker
        })
}

/// The frame's non-blank rows, bounded for a failure message.
pub(crate) fn screen_text(frame: &FrameData) -> String {
    if frame.width == 0 {
        return String::new();
    }
    frame
        .cells
        .chunks(usize::from(frame.width))
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol.as_str())
                .collect::<String>()
        })
        .filter(|row| !row.trim().is_empty())
        .map(|row| row.trim_end().chars().take(160).collect::<String>())
        .take(40)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether the cursor sits after a shell prompt: the sandbox's `LIVE> ` for
/// `sh`, or PowerShell's `PS C:\...> `.
pub(crate) fn shell_ready(frame: &FrameData) -> bool {
    let Some(cursor) = &frame.cursor else {
        return false;
    };
    let width = usize::from(frame.width);
    let start = usize::from(cursor.y) * width;
    let end = start + usize::from(cursor.x).min(width);
    frame.cells.get(start..end).is_some_and(|row| {
        row.iter()
            .map(|cell| cell.symbol.as_str())
            .collect::<String>()
            .trim_end()
            .ends_with('>')
    })
}

/// The input handoff's decisive state, in one bounded line.
pub(crate) fn handoff_diagnostic(live: &LiveState, workspace: &str, tab: &str) -> String {
    let snapshot = live.snapshot.as_deref();
    let surface = live.surface.as_deref();
    let focused_workspace = snapshot.and_then(|s| s.focused_workspace_id.as_deref());
    let focused_tab = snapshot.and_then(|s| s.focused_tab_id.as_deref());
    let label = |id: &str| sheprd_client::text::safe_text(id, 40);
    format!(
        "workspace_match={} tab_match={} shell_ready={} panes={:?} cursor={:?} focused_workspace={} focused_tab={} expected_workspace={} expected_tab={}",
        focused_workspace == Some(workspace),
        focused_tab == Some(tab),
        surface.is_some_and(|s| shell_ready(&s.frame)),
        surface.map(|s| s.panes.len()),
        surface.and_then(|s| s.frame.cursor.as_ref()),
        label(focused_workspace.unwrap_or("none")),
        label(focused_tab.unwrap_or("none")),
        label(workspace),
        label(tab),
    )
}

/// Keeps the decisive state in the harness's bounded log tail even for a full
/// screen.
pub(crate) fn timeout_diagnostic(diagnostic: &str, screen: &str) -> String {
    format!("screen:\n{screen}\ntimeout: {diagnostic}")
}

#[cfg(test)]
mod tests;

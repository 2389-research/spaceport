// ABOUTME: Freezes the Core opt-in for one app session across all windows.
// ABOUTME: Keeps saved preference edits pending until the next application launch.
use gpui::{App, Global};

#[derive(Clone, Copy)]
struct CoreFeature(bool);
impl Global for CoreFeature {}

pub(crate) fn install(enabled: bool, cx: &mut App) {
    cx.set_global(CoreFeature(enabled));
}

pub(crate) fn enabled(cx: &App) -> bool {
    cx.try_global::<CoreFeature>()
        .is_some_and(|feature| feature.0)
}

pub(crate) fn restart_required(saved: bool, cx: &App) -> bool {
    saved != enabled(cx)
}

pub(crate) fn command_requires_core(command: crate::controls::Command) -> bool {
    matches!(
        command,
        crate::controls::Command::NewCoreTerminal | crate::controls::Command::MirrorCorePane
    )
}

pub(crate) fn disabled_message() -> String {
    crate::i18n::fl!("core-feature-disabled")
}

#[cfg(test)]
mod tests;

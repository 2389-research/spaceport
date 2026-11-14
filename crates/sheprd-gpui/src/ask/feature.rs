// ABOUTME: Captures Ask Spaceport opt-in for the current application session.
// ABOUTME: Separates saved preference edits from helper startup and window visibility.
use gpui::{App, Global};

#[derive(Clone, Copy, Default)]
pub(crate) struct Feature(bool);
impl Global for Feature {}

impl Feature {
    pub(super) fn install(enabled: bool, cx: &mut App) {
        cx.set_global(Self(enabled));
    }

    pub(crate) fn current(cx: &App) -> Self {
        cx.try_global::<Self>().copied().unwrap_or_default()
    }

    pub(crate) fn enabled(self) -> bool {
        self.0
    }

    pub(crate) fn restart_required(saved: bool, cx: &App) -> bool {
        saved != Self::current(cx).enabled()
    }
}

#[cfg(test)]
mod tests;

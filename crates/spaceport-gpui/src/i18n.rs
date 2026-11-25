//! The text the app shows, from Fluent messages (docs/i18n.md). The words and
//! the lookup are `spaceport-i18n`'s: [`fl!`] formats a message in the locale
//! shown and fails to compile when its ID or arguments do not match the
//! English file.
//!
//! What stays here is the app's own: the locale every window shows, as a GPUI
//! global, [`CurrentLocale`], chosen by the `language` setting or a QA
//! pseudo-locale, and the one thing loading a catalog does to the chrome
//! besides the text: it publishes the direction the locale lays the chrome out
//! in to `spaceport-widgets`, which mirrors every element by it. Changing the
//! locale reinstalls the menus and redraws every window; text a view already
//! built keeps its language until it is built again.

mod current;
pub(crate) mod errors;
#[cfg(test)]
pub(crate) mod leak_tests;
#[cfg(test)]
mod tests;

pub(crate) use errors::Localize;
// The check `fl!` runs while compiling, for a `const fn` that names a message
// by constant rather than by literal.
pub(crate) use spaceport_i18n::__private::check;
pub(crate) use spaceport_i18n::FluentValue;
#[cfg(test)]
pub(crate) use spaceport_i18n::takes;
use spaceport_i18n::{Direction, Language, Locale, Pseudo};
pub(crate) use spaceport_i18n::{available, endonym, english, fl, message, shown_locale};

/// A catalog command's name in the current locale.
pub(crate) fn command(info: &crate::controls::CommandInfo) -> String {
    message(info.message, &command_args(info.command))
}

/// A catalog command's English name.
pub(crate) fn command_english(info: &crate::controls::CommandInfo) -> String {
    english(info.message, &command_args(info.command))
}

fn command_args(command: crate::Command) -> Vec<(&'static str, FluentValue<'static>)> {
    command
        .number()
        .map(|number| ("number", FluentValue::from(number)))
        .into_iter()
        .collect()
}

/// The locale every window shows, and what chose it.
#[derive(Clone, Debug)]
pub(crate) struct CurrentLocale {
    configured: Language,
    /// QA > Pseudo-Locale, which wins over the setting until turned off.
    pseudo: Option<Pseudo>,
    locale: Locale,
}

impl Default for CurrentLocale {
    fn default() -> Self {
        Self {
            configured: Language::default(),
            pseudo: None,
            locale: Locale::FALLBACK,
        }
    }
}

impl gpui::Global for CurrentLocale {}

impl CurrentLocale {
    pub(crate) fn configured(&self) -> &Language {
        &self.configured
    }

    pub(crate) fn pseudo(&self) -> Option<Pseudo> {
        self.pseudo
    }

    pub(crate) fn locale(&self) -> &Locale {
        &self.locale
    }

    pub(crate) fn direction(&self) -> Direction {
        self.locale.direction()
    }

    fn resolve(configured: Language, pseudo: Option<Pseudo>) -> Self {
        let requested = pseudo.map_or_else(
            || configured.clone(),
            |pseudo| Language::Locale(pseudo.locale()),
        );
        Self {
            locale: requested.resolve(&available(), &spaceport_i18n::system_languages()),
            configured,
            pseudo,
        }
    }
}

/// The pseudo-locale the windows show, if they show one, for the QA menu
/// to check. Like Slow Animations, it is read where there is no app context.
pub(crate) fn shown_pseudo() -> Option<Pseudo> {
    shown_locale()?.pseudo()
}

/// Shows `locale` on this test's thread with no app, for tests of text and
/// layout outside a window.
#[cfg(test)]
pub(crate) fn show(locale: &Locale) {
    current::replace(locale);
}

/// Picks the locale the setting asks for when the app starts, before any
/// window or menu is built.
pub(crate) fn install(language: &Language, cx: &mut gpui::App) {
    let state = CurrentLocale::resolve(language.clone(), None);
    current::replace(&state.locale);
    cx.set_global(state);
}

/// Follows a reloaded `language` setting. Each window applies the config it
/// reloads, so only the first that sees a change switches the locale.
pub(crate) fn follow_config(language: &Language, cx: &mut gpui::App) {
    let current = cx
        .try_global::<CurrentLocale>()
        .cloned()
        .unwrap_or_default();
    if current.configured == *language {
        return;
    }
    switch(CurrentLocale::resolve(language.clone(), current.pseudo), cx);
}

/// QA > Pseudo-Locale: shows `pseudo`, or the setting's locale again when
/// `pseudo` is the one shown.
pub(crate) fn toggle_pseudo(pseudo: Pseudo, cx: &mut gpui::App) {
    let current = cx
        .try_global::<CurrentLocale>()
        .cloned()
        .unwrap_or_default();
    let pseudo = (shown_pseudo() != Some(pseudo)).then_some(pseudo);
    switch(CurrentLocale::resolve(current.configured, pseudo), cx);
}

fn switch(state: CurrentLocale, cx: &mut gpui::App) {
    let changed = cx
        .try_global::<CurrentLocale>()
        .is_none_or(|current| current.locale != state.locale);
    let menus = cx
        .try_global::<CurrentLocale>()
        .is_none_or(|current| current.pseudo != state.pseudo);
    if changed {
        current::replace(&state.locale);
    }
    cx.set_global(state);
    if changed || menus {
        // Menus are built in the locale, and the QA menu checks the one shown.
        crate::menus::install(cx);
        cx.refresh_windows();
    }
}

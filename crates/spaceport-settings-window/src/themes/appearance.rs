//! The Theme page's browser: the search box's frame, the count of themes
//! listed, the cards a row of the grid draws, and the retries under it.

use super::Choice;
use crate::ui;
use gpui::{prelude::*, *};
use spaceport_i18n::fl;
use spaceport_settings::theme::Scope;
use spaceport_settings::{Theme, corners};
use spaceport_widgets::direction::Logical;

/// Every id this module names, so a test can check each is a message.
#[cfg(test)]
pub(crate) const MESSAGE_IDS: &[&str] = &[
    "settings-theme-card-draft",
    "settings-theme-card-saved",
    "settings-theme-catalog-error",
    "settings-theme-count",
    "settings-theme-count-discovering",
    "settings-theme-empty",
    "settings-theme-kind-built-in",
    "settings-theme-kind-file",
    "settings-theme-kind-ghostty",
    "settings-theme-kind-herdr",
    "settings-theme-preview-error",
    "settings-theme-retry-catalog",
    "settings-theme-retry-preview",
    "settings-theme-thumbnail-loading",
    "settings-theme-thumbnail-unavailable",
];

/// A card's palette: the theme it samples, or why it has none to show yet.
pub enum Palette<'a> {
    /// The theme has not loaded.
    Loading,
    /// The theme could not be loaded, so the card says so.
    Unavailable,
    /// The theme's own colors.
    Loaded(&'a Theme),
}

/// What one card of the grid shows.
pub struct Card<'a> {
    /// The theme the page paints with, which the card's ground is drawn on.
    pub theme: &'a Theme,
    /// The row this card is. It both names the theme and says where the name
    /// comes from.
    pub choice: &'a Choice,
    pub index: usize,
    pub selected: bool,
    /// This card's theme is the one the window is drafting.
    pub draft: bool,
    /// This card's theme is the one saved.
    pub saved: bool,
    pub palette: Palette<'a>,
}

/// The search box's frame. `input` is the window's own field, which keeps the
/// text, the caret and the composition this crate knows nothing about.
pub fn search(
    on_key_down: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    input: AnyElement,
) -> Div {
    div()
        .debug_selector(|| "settings-theme-search".into())
        .h(px(32.))
        .flex_none()
        .on_key_down(on_key_down)
        .child(input)
}

/// What the grid and the lines under it are drawn from. The window answers all
/// of it: which rows matched, whether the catalogue is still being read, the
/// scrolled grid itself, and the two failures a load left behind.
pub struct Results<'a> {
    pub theme: &'a Theme,
    /// How many rows the query left, of how many the listed libraries hold.
    pub shown: usize,
    pub total: usize,
    /// Whether the theme directories are still being read.
    pub discovering: bool,
    /// The scrolled grid, or none when no row matched.
    pub body: Option<AnyElement>,
    pub catalog_error: Option<&'a str>,
    pub preview_error: Option<&'a str>,
}

/// The count, the grid in its frame, and the retries under it.
pub fn results(
    results: Results<'_>,
    on_retry_catalog: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_retry_preview: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_leave: impl Fn(&bool, &mut Window, &mut App) + 'static,
) -> Div {
    let Results {
        theme,
        shown,
        total,
        discovering,
        body,
        catalog_error,
        preview_error,
    } = results;
    let count = if discovering {
        fl!(
            "settings-theme-count-discovering",
            shown = shown,
            total = total
        )
    } else {
        fl!("settings-theme-count", shown = shown, total = total)
    };
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .debug_selector(|| "settings-theme-count".into())
                .text_size(px(11.))
                .text_color(rgb(theme.subtext()))
                .child(count),
        )
        .child(
            div()
                .id("settings-theme-list")
                .debug_selector(|| "settings-theme-list".into())
                // The child list scrolls first; keep wheel events inside the
                // grid even at its edges or when the results are empty.
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .flex_1()
                .min_h(px(240.))
                // This height comes from `min_h` in a column of auto height,
                // so it is indefinite and a percentage on the list would
                // resolve to nothing: the list has to flex into it instead.
                .flex()
                .flex_col()
                // Leaving is answered here rather than on a card, so moving
                // between cards never flashes the saved theme.
                .on_hover(on_leave)
                .overflow_hidden()
                .border_1()
                .border_color(rgb(theme.active))
                .rounded(px(corners::CONTROL))
                .child(body.unwrap_or_else(|| {
                    div()
                        .debug_selector(|| "settings-theme-empty".into())
                        .p(px(12.))
                        .child(fl!("settings-theme-empty"))
                        .into_any_element()
                })),
        )
        .when_some(catalog_error, |results, error| {
            results.child(failure(
                "settings-theme-catalog-error",
                fl!("settings-theme-catalog-error", error = error),
                ui::button(
                    theme,
                    "theme-retry-catalog",
                    fl!("settings-theme-retry-catalog"),
                    true,
                )
                .on_click(on_retry_catalog),
            ))
        })
        .when_some(preview_error, |results, error| {
            results.child(failure(
                "settings-theme-preview-error",
                fl!("settings-theme-preview-error", error = error),
                ui::button(
                    theme,
                    "theme-retry-preview",
                    fl!("settings-theme-retry-preview"),
                    true,
                )
                .on_click(on_retry_preview),
            ))
        })
}

/// A failed load, said under the grid, with the button that asks again.
fn failure(selector: &'static str, message: String, retry: Stateful<Div>) -> Div {
    div()
        .debug_selector(move || selector.into())
        .child(message)
        .child(retry)
}

/// One card of the grid: its palette sample, its name, and where the name
/// comes from. `probe` marks the card for a native test that reads where it
/// painted; a build without those passes none.
pub fn card(
    card: Card<'_>,
    probe: Option<AnyElement>,
    on_hover: impl Fn(&bool, &mut Window, &mut App) + 'static,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let Card {
        theme,
        choice,
        index,
        selected,
        draft,
        saved,
        palette,
    } = card;
    let source = source(choice);
    div()
        .id(index)
        .relative()
        .children(probe)
        .debug_selector(move || format!("settings-theme-row-{index}"))
        .h(px(76.))
        .flex_1()
        .min_w_0()
        .overflow_hidden()
        .px(px(7.))
        .py(px(4.))
        .flex()
        .flex_col()
        .rounded(px(corners::CONTROL))
        .border_2()
        .border_color(rgb(if selected {
            theme.primary()
        } else {
            theme.active
        }))
        .when(choice.scope.editable(), |card| card.cursor_pointer())
        .hover(|card| card.bg(rgb(theme.active)))
        .on_hover(on_hover)
        .child(thumbnail(theme, index, palette))
        .child(
            div()
                .debug_selector(move || format!("settings-theme-name-{index}"))
                .min_w_0()
                .truncate()
                .child(choice.name.clone()),
        )
        .child(
            div()
                .debug_selector(move || format!("settings-theme-source-{index}"))
                .text_size(px(10.))
                .text_color(rgb(theme.subtext()))
                .truncate()
                .child(if draft {
                    fl!("settings-theme-card-draft", source = source)
                } else if saved {
                    fl!("settings-theme-card-saved", source = source)
                } else {
                    source
                }),
        )
        .on_click(on_click)
        .into_any_element()
}

/// Where a listed theme comes from, as the card names it.
fn source(choice: &Choice) -> String {
    match choice.scope {
        Scope::Herdr => fl!("settings-theme-kind-herdr"),
        Scope::App if Theme::BUILTIN_NAMES.contains(&choice.name.as_str()) => {
            fl!("settings-theme-kind-built-in")
        }
        Scope::App
            if std::path::Path::new(&choice.name).is_absolute()
                || choice.name.starts_with("~/") =>
        {
            fl!("settings-theme-kind-file")
        }
        Scope::App => fl!("settings-theme-kind-ghostty"),
    }
}

/// The card's sample of its theme: the theme's own ground, a glyph and the
/// first of its palette, or what the card has to say while it has no colors.
fn thumbnail(theme: &Theme, index: usize, palette: Palette<'_>) -> Div {
    let thumbnail = div()
        .debug_selector(move || format!("settings-theme-thumbnail-{index}"))
        .h(px(26.))
        .flex_none()
        .w_full()
        .rounded(px(3.))
        .overflow_hidden();
    match palette {
        Palette::Loaded(colours) => thumbnail
            .bg(rgb(colours.background))
            .text_color(rgb(colours.foreground))
            .px(px(4.))
            .row()
            .items_center()
            .gap(px(3.))
            // A glyph sample, the same in every language.
            .child(div().text_size(px(10.)).child("Aa")) // i18n: data
            .children(
                colours.palette[..8]
                    .iter()
                    .map(|color| div().flex_1().min_w_0().h(px(9.)).bg(rgb(*color))),
            ),
        Palette::Unavailable => unavailable(theme, fl!("settings-theme-thumbnail-unavailable")),
        Palette::Loading => unavailable(theme, fl!("settings-theme-thumbnail-loading")),
    }
}

/// What a card says in place of a palette.
fn unavailable(theme: &Theme, words: String) -> Div {
    div()
        .bg(rgb(theme.surface))
        .text_color(rgb(theme.subtext()))
        .text_size(px(10.))
        .child(words)
}

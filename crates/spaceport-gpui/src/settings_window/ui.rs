//! Shared row and control builders for the Settings pages, at the paths the
//! pages already use: `spaceport-settings-window`'s kit, with this window's
//! words. Pure rendering: they take the theme and return elements; state and
//! saves stay on the view.
// i18n: migrated
use super::catalog::Entry;
use super::inputs::NumberEdit;
use crate::i18n::{self, fl};
use crate::search_input::SearchInput;
use gpui::{prelude::*, *};
use spaceport_settings::{FontConfig, Theme};
use spaceport_settings_window::ui::{
    Editing, Labels, VALUE_MARK, data_row as kit_data_row, dropdown as kit_dropdown,
    number as kit_number, row as kit_row, section as kit_section, swatch_row as kit_swatch_row,
};

/// The row height only the native check and the font tests name. The
/// native module is macOS-gated, so this re-export follows it; the font
/// test reads the crate path directly.
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(in crate::settings_window) use spaceport_settings_window::ui::ROW_HEIGHT;
pub(in crate::settings_window) use spaceport_settings_window::ui::{
    Dropdown, HostColorChange, Look, Number, NumberChange, Row, Source, Unit, button,
    parse_hex_input, parse_number, segmented, toggle,
};

/// The kit's words, worded in this window's language: the messages the kit
/// names, the band a number takes, and the label that follows Herdr's value.
pub(in crate::settings_window) fn labels() -> Labels {
    Labels {
        message: |id| i18n::message(id, &[]),
        number_range: |low, high| fl!("settings-number-range", min = low as i32, max = high as i32),
        follow_herdr: follow_herdr_template,
    }
}

/// A label that follows Herdr's value, with [`VALUE_MARK`] where it goes.
fn follow_herdr_template() -> String {
    fl!("settings-use-herdr", value = VALUE_MARK)
}

/// A catalogued row, its title and description worded here.
pub(in crate::settings_window) fn row(theme: &Theme, entry: &Entry) -> Row {
    kit_row(theme, entry, &labels())
}

/// A row the catalog does not list because it is built from data.
pub(in crate::settings_window) fn data_row(
    theme: &Theme,
    id: impl Into<SharedString>,
    title: impl Into<SharedString>,
) -> Row {
    kit_data_row(theme, id, title, &labels())
}

/// A section heading, tagged with the app that holds its settings.
pub(in crate::settings_window) fn section(
    theme: &Theme,
    title: &str,
    tag_source: Option<Source>,
) -> Div {
    let labels = labels();
    let tag = tag_source.map(|source| {
        let label: SharedString = (labels.message)(source.label_id()).into();
        (source, label)
    });
    kit_section(theme, title, tag)
}

pub(in crate::settings_window) fn number(
    theme: &Theme,
    spec: Number,
    editing: Option<&NumberEdit>,
    on_change: impl Fn(NumberChange, &mut Window, &mut App) + 'static,
    on_edit: impl Fn(&mut Window, &mut App) + 'static,
) -> Div {
    let editing = editing.map(|edit| Editing {
        input: edit.input.clone().into_any_element(),
        invalid: edit.invalid,
    });
    kit_number(theme, spec, editing, &labels(), on_change, on_edit)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::settings_window) fn dropdown(
    theme: &Theme,
    id: &'static str,
    label: impl Into<SharedString>,
    shared_prefix: bool,
    enabled: bool,
    open: Option<&Dropdown>,
    on_open: impl Fn(&mut Window, &mut App) + 'static,
    on_pick: impl Fn(usize, &mut Window, &mut App) + 'static,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    kit_dropdown(
        theme,
        id,
        label,
        shared_prefix,
        enabled,
        open,
        &labels(),
        on_open,
        on_pick,
        on_dismiss,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::settings_window) fn swatch_row(
    theme: &Theme,
    host: &str,
    index: usize,
    current: Option<u32>,
    mono: &FontConfig,
    hint: SharedString,
    hex_input: Option<(Entity<SearchInput>, bool)>,
    on_change: impl Fn(HostColorChange, &mut Window, &mut App) + 'static,
    on_edit_hex: impl Fn(&mut Window, &mut App) + 'static,
) -> Div {
    let hex_input = hex_input.map(|(input, invalid)| (input.into_any_element(), invalid));
    kit_swatch_row(
        theme,
        host,
        index,
        current,
        mono,
        hint,
        hex_input,
        &labels(),
        on_change,
        on_edit_hex,
    )
}

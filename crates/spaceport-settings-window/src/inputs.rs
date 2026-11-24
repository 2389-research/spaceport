//! The choices the window's transient input stands for: which preference a
//! dropdown entry picks, and the value a slider shows before it writes.
//!
//! The text fields, the saves and the window's own drafts stay with the app;
//! what lives here is the table an index is read through and the preview a
//! drag keeps.

use spaceport_i18n::Language;
use spaceport_settings::LayoutMode;
use spaceport_settings::fields::{
    CLIPBOARD_POSITIONS, DropdownField, NumberField, TOAST_POSITIONS,
};
use spaceport_settings::preferences::Preference;

/// What an entry of an override dropdown stands for.
#[derive(Clone, Debug, PartialEq)]
pub enum Choice {
    /// The layout to switch to, which the window's own draft owns.
    Layout(LayoutMode),
    /// The preference to save through the window's queue.
    Preference(Preference),
}

/// What picking `index` of `field`'s list stands for, or none when the list is
/// shorter than the index says. Index 0 of an override list follows Herdr; the
/// rest are local choices. `languages` is the picker's list, which the app
/// words.
pub fn picked(field: DropdownField, index: usize, languages: &[Language]) -> Option<Choice> {
    let local = index.checked_sub(1);
    let flag = local.map(|choice| choice == 0);
    Some(match field {
        DropdownField::Layout => Choice::Layout(*LayoutMode::ALL.get(index)?),
        DropdownField::Language => {
            Choice::Preference(Preference::Language(languages.get(index)?.clone()))
        }
        DropdownField::ClipboardEnabled => Choice::Preference(Preference::ClipboardEnabled(flag)),
        DropdownField::ClipboardPosition => {
            Choice::Preference(Preference::ClipboardPosition(local.and_then(|index| {
                CLIPBOARD_POSITIONS
                    .get(index)
                    .map(|(position, _)| *position)
            })))
        }
        DropdownField::ToastEnabled => Choice::Preference(Preference::NotificationEnabled(flag)),
        DropdownField::ToastPosition => {
            Choice::Preference(Preference::NotificationPosition(local.and_then(|index| {
                TOAST_POSITIONS.get(index).map(|(position, _)| *position)
            })))
        }
    })
}

/// A slider's value while it is dragged, and after release until the save's
/// reload shows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumberPreview {
    pub field: NumberField,
    pub value: f32,
}

impl NumberPreview {
    /// The preview a drag of `field` to `value` shows: none for a field that
    /// writes only when released, or a value outside the field's range.
    pub fn dragged(field: NumberField, value: f32) -> Option<Self> {
        let previewing = matches!(
            field,
            NumberField::ChatMaxWidth | NumberField::SoundVolume | NumberField::TerminalPadding(..)
        );
        (previewing && field.range().contains(&value)).then_some(Self { field, value })
    }

    /// The preview to keep once `value` is committed: it stays until the save's
    /// reload shows the written value, and goes when the write never started.
    pub fn after_commit(self, value: f32, started: bool, saving: bool) -> Option<Self> {
        (started && saving).then_some(Self {
            field: self.field,
            value,
        })
    }
}

/// What `field` shows: a live drag's value, else `saved`.
pub fn number_value(preview: Option<NumberPreview>, field: NumberField, saved: f32) -> f32 {
    preview
        .filter(|preview| preview.field == field)
        .map_or(saved, |preview| preview.value)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;

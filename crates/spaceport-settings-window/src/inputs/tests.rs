//! Which preference an entry picks, and the value a slider shows before and
//! after its write.
use super::*;
use core::prelude::v1::test;

fn languages() -> Vec<Language> {
    vec![Language::System, Language::Locale("fr".parse().unwrap())]
}

#[test]
fn an_override_list_follows_herdr_first_and_offers_its_choices_after() {
    assert_eq!(
        picked(DropdownField::ClipboardEnabled, 0, &languages()),
        Some(Choice::Preference(Preference::ClipboardEnabled(None)))
    );
    assert_eq!(
        picked(DropdownField::ClipboardEnabled, 1, &languages()),
        Some(Choice::Preference(Preference::ClipboardEnabled(Some(true))))
    );
    assert_eq!(
        picked(DropdownField::ClipboardEnabled, 2, &languages()),
        Some(Choice::Preference(Preference::ClipboardEnabled(Some(
            false
        ))))
    );
    // The list's own positions, so the test checks the index, not the names.
    let (toast, _) = TOAST_POSITIONS[1];
    assert_eq!(
        picked(DropdownField::ToastPosition, 2, &languages()),
        Some(Choice::Preference(Preference::NotificationPosition(Some(
            toast
        ))))
    );
    let (clipboard, _) = CLIPBOARD_POSITIONS[5];
    assert_eq!(
        picked(DropdownField::ClipboardPosition, 6, &languages()),
        Some(Choice::Preference(Preference::ClipboardPosition(Some(
            clipboard
        ))))
    );
}

#[test]
fn a_local_list_picks_by_index_and_none_past_its_end() {
    let languages = languages();
    assert_eq!(
        picked(DropdownField::Language, 1, &languages),
        Some(Choice::Preference(Preference::Language(
            languages[1].clone()
        )))
    );
    assert_eq!(picked(DropdownField::Language, 2, &languages), None);
    assert_eq!(
        picked(DropdownField::Layout, 0, &languages),
        Some(Choice::Layout(LayoutMode::ALL[0]))
    );
    assert_eq!(
        picked(DropdownField::Layout, LayoutMode::ALL.len(), &languages),
        None
    );
}

#[test]
fn only_a_slider_field_previews_and_only_inside_its_range() {
    assert_eq!(
        NumberPreview::dragged(NumberField::SoundVolume, 40.),
        Some(NumberPreview {
            field: NumberField::SoundVolume,
            value: 40.
        })
    );
    // A field typed into, and a value outside its range, show nothing early.
    assert_eq!(NumberPreview::dragged(NumberField::ToastDelay, 3.), None);
    assert_eq!(
        NumberPreview::dragged(NumberField::SoundVolume, 1000.),
        None
    );
}

#[test]
fn a_released_value_stays_until_the_save_reloads() {
    let preview = NumberPreview {
        field: NumberField::SoundVolume,
        value: 40.,
    };
    assert_eq!(
        preview.after_commit(50., true, true),
        Some(NumberPreview {
            field: NumberField::SoundVolume,
            value: 50.
        })
    );
    // A write that never started, or one already reloaded, shows the file's.
    assert_eq!(preview.after_commit(50., false, true), None);
    assert_eq!(preview.after_commit(50., true, false), None);
    assert_eq!(
        number_value(Some(preview), NumberField::SoundVolume, 30.),
        40.
    );
    assert_eq!(
        number_value(Some(preview), NumberField::ChatMaxWidth, 30.),
        30.
    );
    assert_eq!(number_value(None, NumberField::SoundVolume, 30.), 30.);
}

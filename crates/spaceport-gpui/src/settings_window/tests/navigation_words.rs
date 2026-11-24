//! The section list and the footer name their words as data, and the icons the
//! list draws are in the registry. Both stay app-side: the registry is the
//! app's, and the words are the app's.
use super::*;

#[test]
fn every_section_icon_loads_through_the_registry() {
    use gpui::AssetSource;
    for section in Section::ALL {
        let loaded = spaceport_icons::Icons.load(section.icon());
        assert!(
            matches!(loaded, Ok(Some(_))),
            "{section:?} icon {} is not in the registry",
            section.icon()
        );
    }
}

#[test]
fn every_id_the_list_and_footer_name_is_an_english_message() {
    for id in spaceport_settings_window::navigation::MESSAGE_IDS {
        let takes = crate::i18n::takes(id);
        assert!(takes.is_some(), "{id} is not an English message");
        let takes = takes.unwrap_or_default();
        // The drafts a save waits on are the one message that takes anything.
        assert!(
            takes.is_empty() || takes == ["drafts"],
            "{id} takes {takes:?}"
        );
    }
}

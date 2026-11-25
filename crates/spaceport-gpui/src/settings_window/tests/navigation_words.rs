//! The icons the section list draws are in the app's registry, which stays
//! app-side; the words the list and footer show are the kit's own.
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

// ABOUTME: Checks the sidebar directory color in dark and light themes.
// ABOUTME: Keeps the blue subdued and readable on both resting and selected rows.
use super::*;
use spaceport_settings::contrast::ratio;

#[::std::prelude::v1::test]
fn directory_has_a_muted_blue_and_readable_contrast() -> Result<()> {
    for theme in [
        Theme::default(),
        spaceport_settings::Config::default().theme_named("Catppuccin Latte", true)?,
    ] {
        let ink = directory_ink(&theme);
        let blue = theme.palette[4];
        let saturation = |color: u32| Hsla::from(rgb(color)).s;
        assert!(
            saturation(ink) < saturation(blue) * 0.65,
            "directory {ink:06x}, palette blue {blue:06x}"
        );
        assert!(ink & 255 > ink >> 16, "directory should retain a blue tint");
        for background in [
            theme.sidebar_background(),
            theme.active,
            crate::layout::agent_selection_background(&theme),
        ] {
            assert!(ratio(ink, background) >= 4.5);
        }
    }
    Ok(())
}

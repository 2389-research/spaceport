//! Applying a configured face to elements: `spaceport_widgets::fonts`' trait
//! and font, at the paths the app already names them by.

pub(crate) use spaceport_widgets::fonts::StyledFont;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    /// gpui-pre-platform enables no features by default. Without `font-kit` the
    /// macOS platform uses a no-op text system, and without a Linux backend so
    /// does Linux: windows open but draw no text at all. The platform can only be
    /// built on the main thread, so check the manifest rather than a live one.
    #[test]
    fn the_platform_crate_keeps_its_text_features() {
        let manifest: toml::Table = toml::from_str(include_str!("../Cargo.toml")).unwrap();
        let features = manifest["dependencies"]["gpui_platform"]["features"]
            .as_array()
            .unwrap();
        for feature in ["font-kit", "wayland", "x11"] {
            assert!(
                features.iter().any(|value| value.as_str() == Some(feature)),
                "gpui_platform is missing the {feature} feature"
            );
        }
    }
}

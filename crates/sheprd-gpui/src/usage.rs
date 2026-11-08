//! Plan usage presentation in the status bar, panel, and Usage section.
//! Core has no usage feed yet; shared host and shell types remain.
// i18n: migrated

mod details;
mod icons;
mod panel;
mod render;
mod section;
pub(crate) mod text;
mod ui;

#[cfg(test)]
mod tests;

pub(crate) use panel::PANEL_WIDTH;
pub(crate) use render::Hint;
#[cfg(test)]
pub(crate) use sheprd_usage::registry;
pub(crate) use sheprd_usage::{Host, Provider, Reading, Shell, Usage, UsageConfig, model};

pub(crate) fn icon(path: &str) -> Option<&'static [u8]> {
    icons::PROVIDER_ICONS
        .iter()
        .find(|(name, _)| *name == path)
        .map(|(_, bytes)| *bytes)
}

pub(crate) fn icon_paths() -> impl Iterator<Item = &'static str> {
    icons::PROVIDER_ICONS.iter().map(|(name, _)| *name)
}

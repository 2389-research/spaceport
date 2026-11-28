//! Plan usage presentation in the status bar, panel, and Usage section.
//! No authoritative usage feed is connected; shared host and shell types remain.
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
pub(crate) use spaceport_usage::registry;
pub(crate) use spaceport_usage::{Host, Provider, Reading, Shell, Usage, model};

/// Every embedded logo's asset path.
pub(crate) fn icon_paths() -> impl Iterator<Item = &'static str> {
    spaceport_icons::provider_paths()
}

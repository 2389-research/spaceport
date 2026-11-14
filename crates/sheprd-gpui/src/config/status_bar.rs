//! Status bar visibility and detail for retained controls.
use serde::{Deserialize, Serialize};

/// How much of a reading an item spells out.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Detail {
    /// Meters and every figure.
    #[default]
    Detailed,
    /// One number per item.
    Compact,
}

/// What the cup keeps awake.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AwakeScope {
    /// The display and the system, so the screen never locks.
    #[default]
    Display,
    /// Only the system, like `caffeinate -i`; the display may sleep and lock.
    System,
}

/// How a status bar button appears.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Button {
    /// Icon and name.
    #[default]
    Label,
    /// Icon alone, named by its tooltip.
    Icon,
    Hidden,
}

impl Button {
    pub(crate) fn shown(self) -> bool {
        self != Self::Hidden
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct StatusBar {
    /// The bar at all; `toggle_status_bar` hides or shows it for the session.
    pub show: bool,
    pub usage: Detail,
    pub keep_awake: bool,
    pub keep_awake_scope: AwakeScope,
    pub theme: Button,
    pub shortcuts: Button,
    pub report_issue: Button,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            show: true,
            usage: Detail::default(),
            keep_awake: true,
            keep_awake_scope: AwakeScope::default(),
            theme: Button::default(),
            shortcuts: Button::default(),
            report_issue: Button::default(),
        }
    }
}

/// One `[status_bar]` key, as the Settings window saves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edit {
    Usage(Detail),
    KeepAwake(bool),
    KeepAwakeScope(AwakeScope),
    Theme(Button),
    Shortcuts(Button),
    ReportIssue(Button),
}

impl Edit {
    pub(super) fn entry(self) -> crate::Result<(&'static str, toml_edit::Value)> {
        use super::edit::spelled;
        Ok(match self {
            Self::Usage(detail) => ("usage", spelled(detail)?),
            Self::KeepAwake(shown) => ("keep_awake", shown.into()),
            Self::KeepAwakeScope(scope) => ("keep_awake_scope", spelled(scope)?),
            Self::Theme(button) => ("theme", spelled(button)?),
            Self::Shortcuts(button) => ("shortcuts", spelled(button)?),
            Self::ReportIssue(button) => ("report_issue", spelled(button)?),
        })
    }
}

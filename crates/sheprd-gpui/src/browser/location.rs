//! The native content a tab shows. Tab kinds retired by earlier builds exist
//! only in the saved-file schema, which drops them before a tab is live.
use crate::i18n::fl;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Location {
    /// Every device this window connects to, drawn by the app, never a page.
    Devices,
    /// herdr-core (Herdr 2) terminals, emulated and drawn by the app, split
    /// into panes. Tabs saved before splits named one `terminal`.
    CoreTerminal {
        #[serde(alias = "terminal")]
        panes: crate::core_terminals::panes::Panes,
        /// A watch-only mirror: it follows the terminal's size and never
        /// types into it.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        observe: bool,
    },
}

impl Location {
    /// A tab's title before its content names itself, as it is saved and the
    /// command line lists it: in English. The window shows [`Self::drawn_title`].
    pub(crate) fn default_title(&self) -> String {
        match self {
            Self::Devices => "Devices".to_owned(), // i18n: data
            Self::CoreTerminal { .. } => "Terminal".to_owned(), // i18n: data
        }
    }

    /// The title of a tab the app draws, in the window's language.
    pub(crate) fn drawn_title(&self) -> String {
        match self {
            Self::Devices => fl!("browser-tab-devices"),
            Self::CoreTerminal { .. } => fl!("browser-tab-terminal"),
        }
    }
}

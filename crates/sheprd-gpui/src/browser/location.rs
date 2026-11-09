//! Validated persisted tab references and native content locations.
//! Retired page payloads are read only so the store can discard them safely.
use super::WebUrl;
use crate::i18n::fl;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Location {
    Web {
        url: WebUrl,
    },
    Local {
        file: LocalFile,
    },
    /// Accepted only while reading an old tab file; the store discards it.
    #[serde(rename = "review")]
    RetiredReview {
        checkout: RetiredReviewCheckout,
    },
    /// Accepted only while reading an old tab file; the store discards it.
    #[serde(rename = "orchestrator")]
    RetiredOrchestrator {
        repo: RetiredOrchestratorRepo,
    },
    /// Accepted only while reading an old tab file; the store discards it.
    #[serde(rename = "code")]
    RetiredSource {
        file: RetiredSourceFile,
    },
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

/// The old source-tab payload is validated before the tab is discarded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SavedSourceFile")]
pub(crate) struct RetiredSourceFile {
    path: String,
    line: Option<u32>,
}

#[derive(Deserialize)]
struct SavedSourceFile {
    path: String,
    line: Option<u32>,
}

impl TryFrom<SavedSourceFile> for RetiredSourceFile {
    type Error = crate::Error;

    fn try_from(saved: SavedSourceFile) -> crate::Result<Self> {
        if saved.path.is_empty()
            || saved.path.len() > 4096
            || saved.path.chars().any(char::is_control)
            || !Path::new(&saved.path).is_absolute()
        {
            return Err(crate::Error::InvalidCodeFile);
        }
        Ok(Self {
            path: saved.path,
            line: saved.line,
        })
    }
}

/// The checkout saved by an old Orchestrator tab, validated before discard.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SavedOrchestratorRepo")]
pub(crate) struct RetiredOrchestratorRepo {
    pub(crate) checkout: String,
}

#[derive(Deserialize)]
struct SavedOrchestratorRepo {
    checkout: String,
}

impl TryFrom<SavedOrchestratorRepo> for RetiredOrchestratorRepo {
    type Error = crate::Error;

    fn try_from(saved: SavedOrchestratorRepo) -> crate::Result<Self> {
        Self::new(saved.checkout)
    }
}

impl RetiredOrchestratorRepo {
    /// An absolute, bounded path without control characters; a remote one
    /// is a POSIX path whatever this machine's own convention.
    pub(crate) fn new(checkout: String) -> crate::Result<Self> {
        let valid = !checkout.is_empty()
            && checkout.len() <= 4096
            && !checkout.chars().any(char::is_control)
            && (checkout.starts_with('/') || Path::new(&checkout).is_absolute());
        if !valid {
            return Err(crate::Error::InvalidBrowserTabs);
        }
        Ok(Self { checkout })
    }
}

/// The local checkout an old review tab saved. It is validated before the
/// store discards the tab; `repo_key` was the daemon's Git common directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SavedReviewCheckout")]
pub(crate) struct RetiredReviewCheckout {
    repo_key: Option<String>,
    branch: String,
    checkout: Option<String>,
}

#[derive(Deserialize)]
struct SavedReviewCheckout {
    #[serde(default)]
    repo_key: Option<String>,
    branch: String,
    checkout: Option<String>,
}

impl TryFrom<SavedReviewCheckout> for RetiredReviewCheckout {
    type Error = crate::Error;

    fn try_from(saved: SavedReviewCheckout) -> crate::Result<Self> {
        let path = |text: &str| {
            !text.is_empty()
                && text.len() <= 4096
                && !text.chars().any(char::is_control)
                && Path::new(text).is_absolute()
        };
        let branch = !saved.branch.is_empty()
            && saved.branch.len() <= 1024
            && !saved.branch.chars().any(char::is_control);
        let named = saved.repo_key.is_some() || saved.checkout.is_some();
        if !named
            || saved.repo_key.as_deref().is_some_and(|key| !path(key))
            || !branch
            || saved.checkout.as_deref().is_some_and(|c| !path(c))
        {
            return Err(crate::Error::InvalidReviewCheckout);
        }
        Ok(Self {
            repo_key: saved.repo_key,
            branch: saved.branch,
            checkout: saved.checkout,
        })
    }
}

impl Location {
    /// A tab's title before its page names itself, as it is saved and the
    /// command line lists it: in English. The window shows [`Self::drawn_title`]
    /// for a tab the app draws.
    pub(crate) fn default_title(&self) -> String {
        match self {
            Self::Web { url } => url.host().to_owned(),
            Self::Local { file } => file.entry.rsplit('/').next().unwrap_or_default().to_owned(),
            Self::RetiredReview { .. } => String::new(),
            Self::RetiredSource { .. } => String::new(),
            // i18n: data
            Self::RetiredOrchestrator { .. } => String::new(),
            Self::Devices => "Devices".to_owned(), // i18n: data
            Self::CoreTerminal { .. } => "Terminal".to_owned(), // i18n: data
        }
    }

    /// The title of a tab the app draws, in the window's language; `None`
    /// for a page or a file, which name themselves.
    pub(crate) fn drawn_title(&self) -> Option<String> {
        match self {
            Self::RetiredOrchestrator { .. } => None,
            Self::Devices => Some(fl!("browser-tab-devices")),
            Self::CoreTerminal { .. } => Some(fl!("browser-tab-terminal")),
            Self::Web { .. }
            | Self::Local { .. }
            | Self::RetiredSource { .. }
            | Self::RetiredReview { .. } => None,
        }
    }
}

/// A file inside a served folder. Only the folder's own files are served,
/// never a hidden one, and never from a folder holding the home directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SavedLocalFile")]
pub(crate) struct LocalFile {
    root: PathBuf,
    /// Relative to `root`, `/`-separated.
    entry: String,
}

#[derive(Deserialize)]
struct SavedLocalFile {
    root: PathBuf,
    entry: String,
}

impl TryFrom<SavedLocalFile> for LocalFile {
    type Error = crate::Error;

    fn try_from(saved: SavedLocalFile) -> crate::Result<Self> {
        let file = Self {
            root: saved.root,
            entry: saved.entry,
        };
        let home = crate::config::home().ok();
        file.check(home.as_deref())?;
        Ok(file)
    }
}

/// Plain names only: no `..`, no hidden files or folders, nothing empty.
fn plain_component(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && !name.contains(['/', '\\', '\0'])
        && !name.chars().any(char::is_control)
}

impl LocalFile {
    /// The file at `path`, served from its folder. `path` must already be
    /// absolute and resolved; the caller's own process does that, so the UI
    /// never touches the disk here.
    #[cfg(test)]
    pub(crate) fn new(path: &Path, home: Option<&Path>) -> crate::Result<Self> {
        let (Some(root), Some(name)) = (path.parent(), path.file_name().and_then(|n| n.to_str()))
        else {
            return Err(crate::Error::InvalidLocalPage);
        };
        let file = Self {
            root: root.to_owned(),
            entry: name.to_owned(),
        };
        file.check(home)?;
        Ok(file)
    }

    fn check(&self, home: Option<&Path>) -> crate::Result<()> {
        let absolute = self.root.is_absolute()
            && self.root.components().all(|part| {
                matches!(
                    part,
                    Component::RootDir | Component::Prefix(_) | Component::Normal(_)
                )
            })
            && self
                .root
                .components()
                .any(|part| matches!(part, Component::Normal(_)));
        let normal = |part: Component<'_>| match part {
            Component::Normal(name) => name.to_str().is_some_and(|name| !name.starts_with('.')),
            _ => true,
        };
        // A folder holding the home directory would expose all of it.
        let exposes_home = home.is_some_and(|home| home.starts_with(&self.root));
        if !absolute
            || exposes_home
            || !self.root.components().all(normal)
            || !self.entry.split('/').all(plain_component)
        {
            return Err(crate::Error::InvalidLocalPage);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

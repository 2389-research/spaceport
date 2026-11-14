// ABOUTME: Reads the saved tab file, including tab kinds the app no longer shows.
// ABOUTME: Checks retired payloads as before, then yields only the tabs that restore.
//! The schema of `browser-tabs.json` as any earlier build may have written
//! it. Only this module can name a retired tab kind: it checks the whole
//! file, reserves every ID in it, and hands the store the tabs that restore.
use super::{Location, MAX_TABS, MAX_TITLE_CHARS, Saved, Scope, Tab, TabId};
use crate::{browser::WebUrl, core_terminals::panes::Panes};
use serde::Deserialize;
use std::path::{Component, Path, PathBuf};

#[derive(Deserialize)]
struct SavedFile {
    tabs: Vec<SavedTab>,
    #[serde(default)]
    next: u64,
}

#[derive(Deserialize)]
struct SavedTab {
    id: TabId,
    scope: Scope,
    workspace_id: String,
    location: Option<SavedLocation>,
    title: String,
    #[serde(default)]
    origin: Option<String>,
    /// Tabs saved before panels existed are all `Group` tabs.
    #[serde(default)]
    place: SavedPlace,
}

/// Retired payloads are still checked, so a file an earlier build refused
/// is refused now, but nothing of them is kept.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[expect(
    dead_code,
    reason = "retired payloads are parsed only to refuse the files they made invalid"
)]
enum SavedLocation {
    Web {
        url: WebUrl,
    },
    Local {
        file: LocalFile,
    },
    Review {
        checkout: ReviewCheckout,
    },
    Orchestrator {
        repo: OrchestratorRepo,
    },
    Code {
        file: SourceFile,
    },
    Devices,
    /// Tabs saved before splits named one `terminal`.
    CoreTerminal {
        #[serde(alias = "terminal")]
        panes: Panes,
        #[serde(default)]
        observe: bool,
    },
}

/// Where a tab was shown. Retired editor names remain parseable so a saved
/// file can retain its other tabs.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SavedPlace {
    #[default]
    Group,
    Px0,
    VsCode,
}

impl<'de> Deserialize<'de> for SavedPlace {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        match name.as_str() {
            "group" => Ok(Self::Group),
            "code" | "code_group" => Ok(Self::VsCode),
            "px0" => Ok(Self::Px0),
            _ => Err(serde::de::Error::unknown_variant(
                &name,
                &["group", "code", "code_group", "px0"],
            )),
        }
    }
}

impl SavedTab {
    fn valid(&self) -> bool {
        self.scope.valid_workspace(&self.workspace_id)
            && self.title.chars().count() <= MAX_TITLE_CHARS
            && self
                .origin
                .as_ref()
                .is_none_or(|pane| !pane.is_empty() && pane.len() <= 256)
            // Editor tabs did not have a terminal agent to own them.
            && (self.place == SavedPlace::Group || self.origin.is_none())
    }

    /// The live tab, or `None` for a kind the app no longer shows: an
    /// editor tab, a page, or a blank tab outside a Core group.
    fn restore(self) -> Option<Tab> {
        match self.place {
            SavedPlace::Group => {}
            SavedPlace::Px0 | SavedPlace::VsCode => return None,
        }
        let location = match (self.location, &self.scope) {
            (Some(SavedLocation::Devices), _) => Some(Location::Devices),
            (Some(SavedLocation::CoreTerminal { panes, observe }), _) => {
                Some(Location::CoreTerminal { panes, observe })
            }
            // A Core tab still waiting for its terminal reference.
            (None, Scope::Core(_)) => None,
            (None, Scope::Legacy(_))
            | (
                Some(
                    SavedLocation::Web { .. }
                    | SavedLocation::Local { .. }
                    | SavedLocation::Review { .. }
                    | SavedLocation::Orchestrator { .. }
                    | SavedLocation::Code { .. },
                ),
                _,
            ) => return None,
        };
        Some(Tab {
            id: self.id,
            scope: self.scope,
            workspace_id: self.workspace_id,
            location,
            title: self.title,
            origin: self.origin,
        })
    }
}

/// The tabs of a saved file that restore, and the next free ID.
pub(super) fn parse(bytes: &[u8]) -> crate::Result<Saved> {
    let file: SavedFile = serde_json::from_slice(bytes)?;
    if file.tabs.len() > MAX_TABS || !file.tabs.iter().all(SavedTab::valid) {
        return Err(crate::Error::InvalidBrowserTabs);
    }
    // Saved group picks may still name a discarded tab. Never assign its ID
    // to a new tab, or that old pick could show unrelated content.
    let next = file
        .tabs
        .iter()
        .map(|tab| tab.id.0.saturating_add(1))
        .max()
        .unwrap_or(0)
        .max(file.next);
    Ok(Saved {
        tabs: file
            .tabs
            .into_iter()
            .filter_map(SavedTab::restore)
            .collect(),
        next,
    })
}

fn bounded_path(text: &str) -> bool {
    !text.is_empty() && text.len() <= 4096 && !text.chars().any(char::is_control)
}

/// An old source tab's payload, which passed that tab's check.
#[derive(Deserialize)]
#[serde(try_from = "RawSourceFile")]
struct SourceFile;

#[derive(Deserialize)]
struct RawSourceFile {
    path: String,
    #[expect(dead_code, reason = "only its type is checked")]
    line: Option<u32>,
}

impl TryFrom<RawSourceFile> for SourceFile {
    type Error = crate::Error;

    fn try_from(raw: RawSourceFile) -> crate::Result<Self> {
        if !bounded_path(&raw.path) || !Path::new(&raw.path).is_absolute() {
            return Err(crate::Error::InvalidCodeFile);
        }
        Ok(Self)
    }
}

/// The checkout an old Orchestrator tab saved: an absolute, bounded path
/// without control characters; a remote one is a POSIX path whatever this
/// machine's own convention.
#[derive(Deserialize)]
#[serde(try_from = "RawOrchestratorRepo")]
struct OrchestratorRepo;

#[derive(Deserialize)]
struct RawOrchestratorRepo {
    checkout: String,
}

impl TryFrom<RawOrchestratorRepo> for OrchestratorRepo {
    type Error = crate::Error;

    fn try_from(raw: RawOrchestratorRepo) -> crate::Result<Self> {
        let checkout = raw.checkout;
        if !bounded_path(&checkout)
            || !(checkout.starts_with('/') || Path::new(&checkout).is_absolute())
        {
            return Err(crate::Error::InvalidBrowserTabs);
        }
        Ok(Self)
    }
}

/// The local checkout an old review tab saved; `repo_key` was the daemon's
/// Git common directory.
#[derive(Deserialize)]
#[serde(try_from = "RawReviewCheckout")]
struct ReviewCheckout;

#[derive(Deserialize)]
struct RawReviewCheckout {
    #[serde(default)]
    repo_key: Option<String>,
    branch: String,
    checkout: Option<String>,
}

impl TryFrom<RawReviewCheckout> for ReviewCheckout {
    type Error = crate::Error;

    fn try_from(raw: RawReviewCheckout) -> crate::Result<Self> {
        let path = |text: &str| bounded_path(text) && Path::new(text).is_absolute();
        let branch = !raw.branch.is_empty()
            && raw.branch.len() <= 1024
            && !raw.branch.chars().any(char::is_control);
        let named = raw.repo_key.is_some() || raw.checkout.is_some();
        if !named
            || raw.repo_key.as_deref().is_some_and(|key| !path(key))
            || !branch
            || raw.checkout.as_deref().is_some_and(|c| !path(c))
        {
            return Err(crate::Error::InvalidReviewCheckout);
        }
        Ok(Self)
    }
}

/// A file inside a folder an old local page tab served. Only the folder's
/// own files were served, never a hidden one, and never from a folder
/// holding the home directory.
#[derive(Deserialize)]
#[serde(try_from = "RawLocalFile")]
struct LocalFile;

#[derive(Deserialize)]
struct RawLocalFile {
    root: PathBuf,
    /// Relative to `root`, `/`-separated.
    entry: String,
}

impl TryFrom<RawLocalFile> for LocalFile {
    type Error = crate::Error;

    fn try_from(raw: RawLocalFile) -> crate::Result<Self> {
        let home = crate::config::home().ok();
        if !servable(&raw.root, &raw.entry, home.as_deref()) {
            return Err(crate::Error::InvalidLocalPage);
        }
        Ok(Self)
    }
}

/// Plain names only: no `..`, no hidden files or folders, nothing empty.
fn plain_component(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && !name.contains(['/', '\\', '\0'])
        && !name.chars().any(char::is_control)
}

fn servable(root: &Path, entry: &str, home: Option<&Path>) -> bool {
    let absolute = root.is_absolute()
        && root.components().all(|part| {
            matches!(
                part,
                Component::RootDir | Component::Prefix(_) | Component::Normal(_)
            )
        })
        && root
            .components()
            .any(|part| matches!(part, Component::Normal(_)));
    let normal = |part: Component<'_>| match part {
        Component::Normal(name) => name.to_str().is_some_and(|name| !name.starts_with('.')),
        _ => true,
    };
    // A folder holding the home directory would expose all of it.
    let exposes_home = home.is_some_and(|home| home.starts_with(root));
    absolute
        && !exposes_home
        && root.components().all(normal)
        && entry.split('/').all(plain_component)
}

#[cfg(test)]
mod tests;

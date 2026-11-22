//! Creating a new project inside a configured palette root, and opening it.
//!
//! The palette already opens an *existing* directory as a workspace
//! (`palette::project_open`). This supplies the missing verb: making the
//! directory. Nothing here invents a location — the new project goes inside a
//! `palette.project_roots` entry the user configured, which is the same
//! convention project discovery already uses.

use super::{Action, Entry, Error, Palette, projects};
use crate::i18n::fl;
use std::{fs, path::PathBuf};

/// A project folder the palette can create under a configured root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct NewProject {
    pub(super) path: PathBuf,
    pub(super) label: String,
}

/// One path segment that is safe to use as a folder name.
///
/// Deliberately narrow: a leading dot (hidden), a separator, or a colon (a
/// Windows drive or an scp-style remote) is refused rather than guessed at.
pub(super) fn parse_name(text: &str) -> Option<String> {
    let name = text.trim();
    if name.is_empty() || name.len() > 128 || name == "." || name == ".." {
        return None;
    }
    if name.starts_with('.') {
        return None;
    }
    if name
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '/' | '\\' | ':'))
    {
        return None;
    }
    Some(name.to_owned())
}

/// Plan a new project from the palette's input.
///
/// Returns `Ok(None)` when the input is not something we would create, so the
/// palette simply offers no row rather than showing an error while typing.
pub(super) fn plan(roots: &[String], text: &str) -> Result<Option<NewProject>, crate::Error> {
    let Some(root) = roots.first() else {
        return Ok(None);
    };
    let root = projects::expand_root(root)?;
    let label = match parse_name(text) {
        Some(name) => name,
        None => return Ok(None),
    };
    Ok(Some(NewProject {
        path: root.join(&label),
        label,
    }))
}

/// Make the directory, returning the project to open.
/// Blocking: callers run it off the UI thread.
pub(super) fn materialise(project: &NewProject) -> Result<projects::Project, crate::Error> {
    let Some(root) = project.path.parent() else {
        return Err(Error::ProjectRootMissing.into());
    };
    if !root.is_dir() {
        return Err(Error::ProjectRootMissing.into());
    }
    // Claim the destination atomically: whichever attempt creates it owns it.
    fs::create_dir(&project.path).map_err(claim_error)?;
    let path = super::canonical::canonicalize(&project.path)
        .map_err(|error| crate::Error::from(error).at_path(&project.path))?;
    Ok(projects::Project {
        path,
        label: project.label.clone(),
    })
}

/// An existing name is the ordinary case; anything else is a real failure.
fn claim_error(source: std::io::Error) -> Error {
    if source.kind() == std::io::ErrorKind::AlreadyExists {
        Error::ProjectExists
    } else {
        Error::ProjectCreate { source }
    }
}

/// The palette row offering to make the typed folder under the first root.
fn new_project_entry(roots: &[String], query: &str) -> Option<Entry> {
    let plan = plan(roots, query).ok().flatten()?;
    let label = fl!("palette-projects-create-folder", name = plan.label.as_str());
    let detail = fl!(
        "palette-projects-new-in",
        path = plan.path.display().to_string()
    );
    Some(Entry::with_keywords(
        label,
        detail,
        fl!("palette-badge-project"),
        Action::NewProject(plan),
        None,
        // The raw query, so the row is offered for exactly what was typed.
        query,
    ))
}

impl Palette {
    /// The base entries plus the row the current query would create, if any.
    ///
    /// `pending` is rebuilt rather than appended to, so the row can change with
    /// the query without accumulating.
    pub(super) fn rebuild_pending(&mut self, roots: &[String]) {
        let mut entries: Vec<Entry> = self.base.iter().cloned().collect();
        if !self.query.trim().is_empty()
            && let Some(entry) = new_project_entry(roots, &self.query)
        {
            entries.push(entry);
        }
        self.pending = entries.into();
    }
}

#[cfg(test)]
mod tests;

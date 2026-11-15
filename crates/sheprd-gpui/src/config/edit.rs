//! One setting at a time, by its dotted key, for the command line: read the
//! effective value, or change the local overrides file. A change is checked
//! the way loading checks the file before it is written, so a value the app
//! would refuse never reaches the disk. The Settings window's writer
//! (`preferences`) shares the table editing and that check.
use super::Error;
use super::{
    Config, DEFAULT_CONFIG, Daemon, daemon_settings, files::write_config, view_settings_path,
};
use crate::Result;
use std::path::Path;
use toml_edit::{Item, TableLike};

/// Where a setting's effective value comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Layer {
    /// `spaceport/config.local.toml`, the user's overrides.
    Local,
    /// `spaceport/config.toml`, the managed defaults.
    Default,
}

/// A command-line value as TOML: a number, boolean, array, inline table, or
/// quoted string, and anything else as a plain string.
pub(crate) fn parse_value(text: &str) -> toml_edit::Value {
    text.trim()
        .parse::<toml_edit::Value>()
        .unwrap_or_else(|_| toml_edit::Value::from(text))
}

/// A config enum's value as the name serde gives it, so the text a writer
/// saves is the text the parser reads back.
pub(super) fn spelled(value: impl serde::Serialize) -> Result<toml_edit::Value> {
    let name: String = serde_json::from_value(serde_json::to_value(value)?)?;
    Ok(name.into())
}

fn lookup<'a>(mut value: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    for part in key.split('.') {
        value = value.as_table()?.get(part)?;
    }
    Some(value)
}

impl Config {
    /// The effective value of `key`, overrides over defaults, read without
    /// writing anything. `None` when neither file sets it.
    pub(crate) fn effective_value(key: &str) -> Result<Option<(toml::Value, Layer)>> {
        Self::effective_value_at(&Self::path()?, key)
    }

    pub(super) fn effective_value_at(
        path: &Path,
        key: &str,
    ) -> Result<Option<(toml::Value, Layer)>> {
        let (text, source) = Self::overrides_text(path)?;
        let local: toml::Value = text
            .parse::<toml::Table>()
            .map(toml::Value::Table)
            .map_err(|error| crate::Error::from(error).at_path(&source))?;
        let merged =
            Self::merged([DEFAULT_CONFIG, &text]).map_err(|error| error.at_path(&source))?;
        let layer = if lookup(&local, key).is_some() {
            Layer::Local
        } else {
            Layer::Default
        };
        Ok(lookup(&merged, key).map(|value| (value.clone(), layer)))
    }

    /// The settings as the app would load them now, checked as loading checks
    /// them (the overrides alone, then layered), without writing anything.
    /// Fonts are not resolved: only the app's text system knows them.
    pub(crate) fn check_files() -> Result<Self> {
        Self::check_files_at(
            &Self::path()?,
            &view_settings_path(sheprd_client::local_var),
        )
    }

    pub(super) fn check_files_at(path: &Path, daemon: &Path) -> Result<Self> {
        let base = daemon_settings(daemon);
        let (text, source) = Self::overrides_text(path)?;
        Self::parse_over(&text, &base).map_err(|error| error.at_path(&source))?;
        Self::parse_layers([DEFAULT_CONFIG, &text], &base).map_err(|error| error.at_path(&source))
    }

    /// The overrides in `text` as loading checks them: alone, so a bad type
    /// or unknown key cannot hide inside the merge, then over the defaults.
    /// Every writer of the overrides file asks this before it writes.
    pub(super) fn check_overrides(text: &str, base: &Daemon) -> Result<Self> {
        let checked = Self::parse_over(text, base)?;
        Self::parse_layers([DEFAULT_CONFIG, text], base)?;
        Ok(checked)
    }

    /// Writes `edited` over the overrides at `path`, unless that would break
    /// a file that loads. A file the loader already refuses is written as
    /// asked: the edit may be the repair, and refusing would leave the
    /// Settings window unable to change anything. The daemon's own config is
    /// left out of the check: no preference edits what it can conflict with.
    pub(super) fn write_unless_broken(path: &Path, before: &str, edited: &str) -> Result<()> {
        let base = Daemon::default();
        if let Err(error) = Self::check_overrides(edited, &base)
            && Self::check_overrides(before, &base).is_ok()
        {
            return Err(error);
        }
        write_config(path, edited)
    }

    /// Sets `key` in the local overrides file, keeping everything else in it.
    pub(crate) fn set_key(key: &str, value: toml_edit::Value) -> Result<()> {
        let daemon = view_settings_path(sheprd_client::local_var);
        let (_lock, local) = Self::prepare_files(&Self::path()?)?;
        Self::edit_key_at(&local, &daemon, key, Some(value)).map(drop)
    }

    /// Removes `key` from the local overrides file; false when it was not set.
    pub(crate) fn unset_key(key: &str) -> Result<bool> {
        let daemon = view_settings_path(sheprd_client::local_var);
        let (_lock, local) = Self::prepare_files(&Self::path()?)?;
        Self::edit_key_at(&local, &daemon, key, None)
    }

    /// Edits the overrides file at `local`. The edited text must load, and
    /// must not name a key the file did not already get wrong, or the file
    /// is left as it was. Returns whether anything changed.
    pub(super) fn edit_key_at(
        local: &Path,
        daemon: &Path,
        key: &str,
        value: Option<toml_edit::Value>,
    ) -> Result<bool> {
        let parts: Vec<&str> = key.split('.').collect();
        let text = std::fs::read_to_string(local)
            .map_err(|error| crate::Error::from(error).at_path(local))?;
        let mut document = text
            .parse::<toml_edit::DocumentMut>()
            .map_err(|error| crate::Error::from(error).at_path(local))?;
        let changed = match value {
            Some(value) => {
                insert(document.as_table_mut(), &parts, value, key)?;
                true
            }
            None => remove(document.as_table_mut(), &parts),
        };
        if !changed {
            return Ok(false);
        }
        let edited = document.to_string();
        let base = daemon_settings(daemon);
        let checked = Self::check_overrides(&edited, &base)?;
        let before = Self::parse_over(&text, &base)
            .map(|config| config.unknown_keys)
            .unwrap_or_default();
        let unknown: Vec<String> = checked
            .unknown_keys
            .into_iter()
            .filter(|key| !before.contains(key))
            .collect();
        if !unknown.is_empty() {
            return Err(Error::UnknownKeys(unknown).into());
        }
        write_config(local, &edited)?;
        Ok(true)
    }
}

/// How a writer shapes a table it has to create. Each is what its writer's
/// users already have in their files, so neither may drift to the other.
#[derive(Clone, Copy)]
pub(super) enum NewTable {
    /// The command line: no header of its own, only its children's, and an
    /// inline table inside an inline one.
    Implicit,
    /// The Settings window: its own header, and dotted keys inside an
    /// inline table.
    Headed,
}

/// The table `tables` names under `root`, with the missing ones created.
/// The error is how many names lead to the part that holds a value, not a
/// table.
pub(super) fn table_at<'a>(
    root: &'a mut dyn TableLike,
    tables: &[&str],
    new: NewTable,
) -> std::result::Result<&'a mut dyn TableLike, usize> {
    let mut table = root;
    for (depth, name) in tables.iter().enumerate() {
        if !table.contains_key(name) {
            let mut child = toml_edit::Table::new();
            match new {
                NewTable::Implicit => {
                    child.set_implicit(true);
                    table.insert(name, Item::Table(child));
                }
                NewTable::Headed => {
                    table.entry(name).or_insert(Item::Table(child));
                }
            }
        }
        table = table
            .get_mut(name)
            .and_then(Item::as_table_like_mut)
            .ok_or(depth)?;
    }
    Ok(table)
}

/// Sets `key`, keeping the comment beside the value it replaces.
pub(super) fn set(table: &mut dyn TableLike, key: &str, mut value: toml_edit::Value) {
    if let Some(previous) = table.get(key).and_then(Item::as_value) {
        *value.decor_mut() = previous.decor().clone();
    }
    table.insert(key, Item::Value(value));
}

fn insert(
    root: &mut dyn TableLike,
    parts: &[&str],
    value: toml_edit::Value,
    key: &str,
) -> Result<()> {
    let Some((last, tables)) = parts.split_last() else {
        return Ok(());
    };
    let table = table_at(root, tables, NewTable::Implicit).map_err(|_| Error::KeyNotTable {
        key: key.to_owned(),
    })?;
    set(table, last, value);
    Ok(())
}

/// Removes the key and the tables it leaves empty.
fn remove(table: &mut dyn TableLike, parts: &[&str]) -> bool {
    match parts {
        [] => false,
        [last] => table.remove(last).is_some(),
        [first, rest @ ..] => {
            let Some(child) = table.get_mut(first).and_then(Item::as_table_like_mut) else {
                return false;
            };
            let removed = remove(child, rest);
            let empty = child.is_empty();
            if removed && empty {
                table.remove(first);
            }
            removed
        }
    }
}

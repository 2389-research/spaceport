//! Targeted shortcut edits use the same lock, validation, and atomic write as
//! other GUI preferences. Reset removes the override, restoring platform and
//! daemon defaults; an empty list explicitly unbinds the command.
//!
//! This crate cannot resolve a keymap: the keymap crate depends on it. The
//! caller passes the resolver as `validate`, and the save runs it against the
//! merged file under the lock, so two windows editing from older snapshots
//! cannot together save a map that reload rejects.
use super::{
    Binding, Config, Daemon, LOCAL_CONFIG, daemon_settings, view_settings_path, write_config,
};
use crate::{Error, Result};
use spaceport_commands::COMMANDS;
use std::{fs, io::ErrorKind, path::Path};

impl Config {
    /// Saves one command's override. `validate` receives the configuration
    /// reload would read from the merged file; its error is returned as is
    /// and the file is left untouched.
    pub fn save_keybinding<E: From<Error>>(
        name: &'static str,
        binding: Option<Binding>,
        validate: impl FnOnce(&Config) -> Result<(), E>,
    ) -> Result<(), E> {
        Self::save_keybinding_at(
            &Self::path()?,
            &view_settings_path(spaceport_client::local_var),
            name,
            binding,
            validate,
        )
    }

    /// Loads a managed `path` and a `view` settings file of a test's own, as
    /// `load` does.
    #[cfg(any(test, feature = "test-support"))]
    pub fn load_fixture(path: &Path, view: &Path) -> Result<Self> {
        Self::load_path(path, view)
    }

    /// `save_keybinding` against the managed `path` and the `view` settings
    /// file whose `[keys]` reload layers under it. The lock covers the reread,
    /// the merge, `validate` and the write.
    pub fn save_keybinding_at<E: From<Error>>(
        path: &Path,
        view: &Path,
        name: &str,
        binding: Option<Binding>,
        validate: impl FnOnce(&Config) -> Result<(), E>,
    ) -> Result<(), E> {
        let (_lock, local) = Self::prepare_files(path)?;
        let (text, candidate) =
            Self::merge_keybinding(name, binding, &local, &daemon_settings(view))
                .map_err(|error| error.at_path(&local))?;
        validate(&candidate)?;
        Ok(write_config(&local, &text)?)
    }

    /// The file's text with `name`'s override changed, and the configuration
    /// reload reads from that text. Rereading under the lock means another
    /// window's edit is kept rather than overwritten.
    fn merge_keybinding(
        name: &str,
        binding: Option<Binding>,
        path: &Path,
        base: &Daemon,
    ) -> Result<(String, Self)> {
        if !COMMANDS.iter().any(|info| info.name == name) {
            return Err(Error::UnknownKeybinding(name.into()));
        }
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => LOCAL_CONFIG.into(),
            Err(error) => return Err(error.into()),
        };
        let mut document = text.parse::<toml_edit::DocumentMut>()?;
        let table = document
            .entry("keybindings")
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
            .as_table_like_mut()
            .ok_or(crate::herdr_settings::Error::Table("keybindings"))?;
        match binding {
            None => {
                table.remove(name);
            }
            Some(binding) => {
                let mut value = match binding {
                    Binding::One(key) => toml_edit::Value::from(key),
                    Binding::Many(keys) => toml_edit::Value::Array(keys.into_iter().collect()),
                };
                if let Some(previous) = table.get(name).and_then(toml_edit::Item::as_value) {
                    *value.decor_mut() = previous.decor().clone();
                }
                table.insert(name, toml_edit::Item::Value(value));
            }
        }
        if table.is_empty() {
            document.remove("keybindings");
        }
        let text = document.to_string();
        let candidate = Self::parse_loaded(&text, base)?;
        Ok((text, candidate))
    }
}

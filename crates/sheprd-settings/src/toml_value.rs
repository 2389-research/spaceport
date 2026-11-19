//! A settings value as the text a writer saves.

use crate::Result;

/// A config enum's value as the name serde gives it, so the text a writer
/// saves is the text the parser reads back.
pub(crate) fn spelled(value: impl serde::Serialize) -> Result<toml_edit::Value> {
    let name: String = serde_json::from_value(serde_json::to_value(value)?)?;
    Ok(name.into())
}

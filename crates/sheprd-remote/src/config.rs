//! The provider tables of Sheprd's config: `[azure]`, `[container]`,
//! and `[sprites]`. The app owns
//! the config files, their layers, and their maintenance; it hands this crate
//! a [`set_loader`] once at startup, and a provider job reads its table
//! through it when it runs, so an edit since startup applies.

mod azure;
mod container;
mod sprites;

pub use azure::AzureConfig;
#[cfg(feature = "azure")]
pub use azure::AzureFields;
pub use container::ContainerConfig;
#[cfg(feature = "container")]
pub use container::ContainerFields;
pub use sprites::SpritesConfig;
#[cfg(feature = "sprites")]
pub use sprites::SpritesFields;

use std::sync::OnceLock;

/// The provider tables, as the app's config last loaded them.
#[derive(Clone, Debug, Default)]
pub struct Sections {
    pub azure: AzureConfig,
    pub container: ContainerConfig,
    pub sprites: SpritesConfig,
}

/// Why the app could not load its config. The app's own error, kept whole as
/// the source, so its message and chain reach whoever reports the failure.
pub type LoadError = Box<dyn std::error::Error + Send + Sync>;

static LOADER: OnceLock<fn() -> Result<Sections, LoadError>> = OnceLock::new();

/// Sets, once per process, how provider jobs read the config. `load` runs on
/// their background workers, never on the UI thread. Returns `false` when one
/// was already set, which is kept.
pub fn set_loader(load: fn() -> Result<Sections, LoadError>) -> bool {
    LOADER.set(load).is_ok()
}

/// The provider tables now. Fails when the config cannot be read or the app
/// set no loader.
pub fn load() -> crate::Result<Sections> {
    let load = LOADER.get().ok_or(crate::Error::NoConfig)?;
    load().map_err(crate::Error::Config)
}

//! The Daytona account and saved devices shown in Settings.

use super::{Result, Settings, http::Request, store};
use crate::cloud::{self, CloudProvider, SavedDevice};
use crate::credentials::Store;
use secrecy::SecretString;

/// Where the API key in use comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Missing,
    Saved,
    Environment,
}

/// What the Settings tab shows about an account.
#[derive(Debug)]
pub struct Overview {
    pub key: Key,
    /// How many sandboxes the key can see, or why that could not be read.
    pub sandboxes: Option<Result<usize>>,
    /// This account's saved devices.
    pub devices: Vec<SavedDevice>,
}

pub fn overview(settings: &Settings) -> Result<Overview> {
    let devices = cloud::load()?
        .into_iter()
        .filter(|saved| saved.provider == CloudProvider::Daytona && saved.account == settings.base)
        .collect();
    let key = if std::env::var_os(store::VARIABLE).is_some() {
        Key::Environment
    } else if store::saved(settings.store)?.is_some() {
        Key::Saved
    } else {
        Key::Missing
    };
    let sandboxes = (key != Key::Missing).then(|| {
        let key = store::key(settings)?;
        Request {
            settings,
            key: &key,
        }
        .sandboxes()
        .map(|sandboxes| sandboxes.len())
    });
    Ok(Overview {
        key,
        sandboxes,
        devices,
    })
}

/// Save (or with `None`, forget) the API key typed into Settings.
pub fn save_key(store: Store, key: Option<&SecretString>) -> Result<()> {
    store::save(store, key)
}

/// Forget one saved device; its sandbox is left untouched in Daytona.
pub fn forget_device(id: &str) -> Result<()> {
    Ok(cloud::remove(CloudProvider::Daytona, id)?)
}

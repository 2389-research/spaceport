//! Read the boxd account shown in Settings, forget saved devices, and delete
//! a machine when the user explicitly removes it.

use super::{Result, Settings, http::Request, store};
use crate::{
    cloud::{self, CloudProvider, SavedDevice},
    credentials::Store,
};
use secrecy::SecretString;

/// Where the API key in use comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Missing,
    Saved,
    Environment,
}

/// What the boxd tab shows about an account.
#[derive(Debug)]
pub struct Overview {
    pub key: Key,
    /// How many machines this app made that the key sees, or why that could
    /// not be read.
    pub machines: Option<Result<usize>>,
    pub devices: Vec<SavedDevice>,
}

pub fn overview(settings: &Settings) -> Result<Overview> {
    let devices = cloud::load()?
        .into_iter()
        .filter(|saved| saved.provider == CloudProvider::Boxd && saved.account == settings.base)
        .collect();
    let key = if store::from_env() {
        Key::Environment
    } else if store::saved(settings.store)?.is_some() {
        Key::Saved
    } else {
        Key::Missing
    };
    let machines = (key != Key::Missing).then(|| {
        let key = store::key(settings)?;
        Request {
            settings,
            key: &key,
        }
        .machines()
        .map(|machines| machines.len())
    });
    Ok(Overview {
        key,
        machines,
        devices,
    })
}

/// Save (or with `None`, forget) the API key typed into Settings.
pub fn save_key(store: Store, key: Option<&SecretString>) -> Result<()> {
    store::save(store, key)
}

/// Forget one saved device; its machine is left as it is.
pub fn forget_device(id: &str) -> Result<()> {
    Ok(cloud::remove(CloudProvider::Boxd, id)?)
}

/// Delete the saved machine `id`, then forget it. A device no longer saved
/// has nothing left to do. Blocks: run it on a worker.
pub fn destroy_saved(id: &str) -> crate::Result<()> {
    let Some(device) = cloud::load()?
        .into_iter()
        .find(|device| device.provider == CloudProvider::Boxd && device.id == id)
    else {
        return Ok(());
    };
    let settings = super::connect::settings(&device.account)?;
    let key = store::key(&settings)?;
    Request {
        settings: &settings,
        key: &key,
    }
    .delete(id)?;
    Ok(cloud::remove(CloudProvider::Boxd, id)?)
}

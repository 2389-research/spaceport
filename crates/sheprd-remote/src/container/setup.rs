//! The blocking jobs behind the Container tab: read saved devices and their
//! engine state, stop one, or delete one with its home volume.

use super::{Engine, Result, Settings, machine};
pub use super::{
    machine::State,
    settings::{DEFAULT_CPUS, DEFAULT_IMAGE, DEFAULT_MEMORY},
};
use crate::cloud::Value;
use crate::cloud::{self, CloudProvider, SavedDevice};

/// A saved container and what its engine says about it now.
#[derive(Clone, Debug)]
pub struct Device {
    pub saved: SavedDevice,
    /// `None` when the engine could not be asked.
    pub state: Option<State>,
}

/// What the Container tab shows.
#[derive(Clone, Debug)]
pub struct Overview {
    /// The engine for saved containers and its server version, or why there is
    /// none to use.
    pub engine: std::result::Result<(Engine, String), String>,
    pub devices: Vec<Device>,
}

pub fn overview(settings: &Settings, cancelled: &dyn Fn() -> bool) -> Result<Overview> {
    let quick = std::time::Duration::from_secs(10);
    let engine = Engine::pick(settings.engine)
        .and_then(|engine| {
            let version = engine.run(
                "version",
                ["version", "--format", "{{.Server.Version}}"],
                quick,
                cancelled,
            )?;
            Ok((engine, version))
        })
        .map_err(|error| error.to_string());
    let devices: Vec<Device> = cloud::load()?
        .into_iter()
        .filter(|saved| saved.provider == CloudProvider::Container)
        .map(|saved| {
            let state = Engine::from_key(&saved.account)
                .and_then(|engine| machine::state(engine, &saved.id, cancelled).ok());
            Device { saved, state }
        })
        .collect();
    Ok(Overview { engine, devices })
}

/// Delete the saved container `id` with its home volume, then forget it. A
/// device no longer saved has nothing left to do. Blocks: run it on a worker.
pub fn destroy_saved(id: &str) -> crate::Result<()> {
    let Some(device) = cloud::load()?
        .into_iter()
        .find(|device| device.provider == CloudProvider::Container && device.id == id)
    else {
        return Ok(());
    };
    let engine =
        Engine::from_key(&device.account).ok_or(super::Error::Field(Value::ContainerEngine))?;
    machine::delete(engine, id, &|| false)?;
    Ok(cloud::remove(CloudProvider::Container, id)?)
}

/// Stop the saved container `id`, keeping its files for the next connection.
pub fn stop_saved(id: &str) -> crate::Result<()> {
    let Some(device) = cloud::load()?
        .into_iter()
        .find(|device| device.provider == CloudProvider::Container && device.id == id)
    else {
        return Ok(());
    };
    let engine =
        Engine::from_key(&device.account).ok_or(super::Error::Field(Value::ContainerEngine))?;
    Ok(super::lifecycle::stop(engine, id, || {
        machine::stop(engine, id, &|| false)
    })?)
}

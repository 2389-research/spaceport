//! The blocking read behind the Container tab: saved devices and engine state.

use super::{Engine, Result, Settings, machine};
pub use super::{
    machine::State,
    settings::{DEFAULT_CPUS, DEFAULT_IMAGE, DEFAULT_MEMORY},
};
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

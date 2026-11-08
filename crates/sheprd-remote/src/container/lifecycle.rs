//! Serialize starts of the same saved container across connection workers.
//! A second view must not race the first one's engine state and start calls.

use super::{Engine, Result};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, PoisonError},
};

type Devices = HashMap<String, HashMap<String, Arc<Mutex<()>>>>;
static DEVICES: LazyLock<Mutex<Devices>> = LazyLock::new(Mutex::default);

fn device(account: &str, id: &str) -> Arc<Mutex<()>> {
    DEVICES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .entry(account.to_owned())
        .or_default()
        .entry(id.to_owned())
        .or_default()
        .clone()
}

pub(super) fn start(engine: Engine, id: &str, start: impl FnOnce() -> Result<()>) -> Result<()> {
    let device = device(engine.key(), id);
    let _operation = device.lock().unwrap_or_else(PoisonError::into_inner);
    start()
}

#[cfg(test)]
mod tests;

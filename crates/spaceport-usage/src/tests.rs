#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{
    Reading, UsageConfig,
    model::{Account, Kind, Provider, Report, WEEK, Window},
    registry,
};
use crate::shell::Error;
use std::time::{Duration, SystemTime};

mod display_contract;
mod labels;
#[cfg(unix)]
mod linuxbrew_path;
mod pace_exhausted;
mod provider_settings;
#[cfg(unix)]
mod remote_hosts;
mod shown_providers;
mod wsl_hosts;

fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

fn provider(id: &str) -> Provider {
    registry::find(id).unwrap()
}

fn report(provider: Provider, used: f64) -> Report {
    Report::new(
        provider,
        Account::default(),
        vec![Window::new(Kind::Session, used, None, None)],
    )
}

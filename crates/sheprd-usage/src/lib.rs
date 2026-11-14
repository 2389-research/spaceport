// ABOUTME: Usage report presentation models and shared host shell primitives.
// ABOUTME: Holds no provider polling, sign-in, cookie or credential implementation.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

pub mod details;
mod error;
pub mod model;
pub mod registry;
pub mod settings;
pub mod shell;
pub mod text;

#[cfg(all(test, unix))]
#[path = "../../test-support/executable.rs"]
mod test_executable;
#[cfg(test)]
mod tests;

pub use error::{Error, Result};
pub use model::{Host, Provider};
pub use settings::UsageConfig;
pub use shell::Shell;
pub use text::{Arg, Text};

use model::Report;
use std::{collections::HashMap, sync::Arc, time::SystemTime};

/// A report and any error associated with its observation.
#[derive(Clone, Debug)]
pub struct Reading {
    pub provider: Provider,
    pub report: Option<Report>,
    pub error: Option<Arc<Error>>,
}

/// Last observed reports for one host. Missing reports are not zero usage.
#[derive(Default)]
pub struct Entry {
    pub readings: Vec<Reading>,
    pub error: Option<Arc<Error>>,
    pub updated: Option<SystemTime>,
}

impl Entry {
    /// Display filtering is shared by the status bar, panel, section and CLI.
    /// It never deletes or changes an observed report.
    pub fn visible_readings<'a>(
        &'a self,
        config: &'a UsageConfig,
    ) -> impl Iterator<Item = &'a Reading> {
        self.readings
            .iter()
            .filter(|reading| !config.hidden(reading.provider))
    }

    /// The providers the status bar shows: those with numbers to show, the
    /// one chosen in the panel first, then the ones closest to a limit.
    /// The layout decides how many fit. A provider that has no report yet,
    /// or no windows or balances, waits in the panel.
    pub fn headline<'a>(
        &'a self,
        chosen: Option<Provider>,
        config: &'a UsageConfig,
    ) -> Vec<&'a Reading> {
        let mut shown: Vec<&Reading> = self
            .visible_readings(config)
            .filter(|reading| has_numbers(reading))
            .collect();
        // Stable: equally used providers keep the registry's order.
        shown.sort_by(|a, b| urgency(b).total_cmp(&urgency(a)));
        if let Some(index) = shown
            .iter()
            .position(|reading| Some(reading.provider) == chosen)
        {
            let reading = shown.remove(index);
            shown.insert(0, reading);
        }
        shown
    }
}

impl Entry {
    /// The providers the panel offers as tabs: those with numbers to show,
    /// and those explicitly selected by display preferences.
    pub fn tabs<'a>(&'a self, config: &'a UsageConfig) -> Vec<&'a Reading> {
        self.visible_readings(config)
            .filter(|reading| has_numbers(reading) || config.shown(reading.provider))
            .collect()
    }
}

fn has_numbers(reading: &Reading) -> bool {
    reading
        .report
        .as_ref()
        .is_some_and(|report| !report.windows.is_empty() || !report.balances.is_empty())
}

fn urgency(reading: &Reading) -> f32 {
    reading
        .report
        .as_ref()
        .and_then(|report| report.tightest())
        .map_or(0., |window| window.used)
}

/// Presentation state only. No authoritative report feed is connected yet.
#[derive(Default)]
pub struct Usage {
    host: Option<Host>,
    entries: HashMap<Host, Entry>,
    chosen: Option<Provider>,
}

impl Usage {
    /// Tracks `host` with `readings` already read, as a finished read would
    /// leave it: for the app's status bar and panel tests.
    #[cfg(feature = "test-support")]
    pub fn showing(host: Host, readings: Vec<Reading>) -> Self {
        let mut usage = Self::default();
        usage.entries.insert(
            host.clone(),
            Entry {
                readings,
                ..Entry::default()
            },
        );
        usage.host = Some(host);
        usage
    }

    /// What the status bar shows for the tracked host.
    pub fn current(&self) -> Option<&Entry> {
        self.entries.get(self.host.as_ref()?)
    }

    /// What was last read on `host`, tracked or not.
    pub fn entry(&self, host: &Host) -> Option<&Entry> {
        self.entries.get(host)
    }

    /// The provider picked in the panel to lead the status bar.
    pub fn chosen(&self) -> Option<Provider> {
        self.chosen
    }

    /// Leads the status bar with `provider`, without reading anything again.
    pub fn choose(&mut self, provider: Provider) {
        self.chosen = Some(provider);
    }
}

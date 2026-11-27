// ABOUTME: Reads `[usage]`, which picks the daemon-reported usage the window shows.
// ABOUTME: Herdr draws the indicators itself; these switches only show or hide them.
use serde::Deserialize;

/// Whether the window shows the usage indicators the daemon reports.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Indicators {
    /// The daemon's tab bar segments, shown in the title bar.
    pub topbar: bool,
    /// The daemon's configured sidebar rows, and the workspace status they
    /// roll up.
    pub inline: bool,
}

impl Default for Indicators {
    fn default() -> Self {
        Self {
            topbar: true,
            inline: true,
        }
    }
}

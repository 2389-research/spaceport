// ABOUTME: Holds the confirmed idle time before Priority groups an agent under Quiet.
// ABOUTME: Shares the sidebar's three supported durations with the settings editor.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum QuietAfter {
    #[default]
    #[serde(rename = "24h")]
    OneDay,
    #[serde(rename = "48h")]
    TwoDays,
    #[serde(rename = "7d")]
    OneWeek,
}

impl QuietAfter {
    pub fn milliseconds(self) -> u64 {
        let days = match self {
            Self::OneDay => 1,
            Self::TwoDays => 2,
            Self::OneWeek => 7,
        };
        days * 24 * 3_600_000
    }
}

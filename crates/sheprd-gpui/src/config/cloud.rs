//! The retired `[cloud]` setup table stays parseable while saved Container
//! devices still load. Its values cannot launch scripts or install agents.

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CloudConfig {
    // keep-sorted start
    pub agents: Option<Vec<String>>,
    pub container: CloudSetup,
    pub setup: Option<String>,
    // keep-sorted end
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CloudSetup {
    pub agents: Option<Vec<String>>,
    pub setup: Option<String>,
}

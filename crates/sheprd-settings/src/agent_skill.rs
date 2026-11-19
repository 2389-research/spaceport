//! `[agent_skill]`: where the app may put its managed agent skill. It wins
//! over the answer saved in `agent-skill.json`, so it can travel with
//! dotfiles and a fresh machine does not ask again.
use serde::Deserialize;

/// Where the managed skill may go on its own.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillInstall {
    /// Offer it once on this machine; saved SSH hosts are not touched.
    #[default]
    Local,
    /// Never offer, refresh, or remove it here. Installing
    /// from Settings or the palette still works, once.
    Off,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AgentSkillConfig {
    pub install: SkillInstall,
}

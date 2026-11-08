use super::*;
use crate::config::SkillInstall;

#[test]
fn agent_skill_install_is_local_or_off() -> anyhow::Result<()> {
    for config in [
        Config::default(),
        Config::parse("")?,
        Config::parse(DEFAULT_CONFIG)?,
    ] {
        assert_eq!(config.agent_skill.install, SkillInstall::Local);
    }
    for (name, install) in [("local", SkillInstall::Local), ("off", SkillInstall::Off)] {
        let text = format!("[agent_skill]\ninstall = \"{name}\"");
        assert_eq!(Config::parse(&text)?.agent_skill.install, install);
    }
    for bad in ["\"Off\"", "\"never\"", "false"] {
        assert!(Config::parse(&format!("[agent_skill]\ninstall = {bad}")).is_err());
    }
    Ok(())
}

#[test]
fn remote_agent_skill_install_value_is_rejected() {
    assert!(Config::parse("[agent_skill]\ninstall = \"ask\"").is_err());
}

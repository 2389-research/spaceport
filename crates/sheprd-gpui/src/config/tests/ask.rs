// ABOUTME: Verifies Ask agent choice persists without changing ordinary chat defaults.
// ABOUTME: Refuses unsupported agent choices until their restricted role is qualified.
use super::*;
use crate::config::preferences::Preference;

#[test]
fn ask_enablement_and_agent_are_independent_settings() -> anyhow::Result<()> {
    assert!(!Config::default().chat.ask_enabled);
    for enabled in [false, true] {
        let config = Config::parse(&format!(
            "[chat]\nask_enabled = {enabled}\nask_agent = 'pi'"
        ))?;
        assert_eq!(config.chat.ask_enabled, enabled);
        assert_eq!(config.chat.ask_agent, Some(chat::AskAgent::Pi));
    }
    Ok(())
}

#[test]
fn ask_switch_save_keeps_the_agent_and_other_preferences() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("local.toml");
    fs::write(&path, "[chat]\nmax_width = 900 # keep\nask_agent = 'pi'\n")?;
    for enabled in [true, false] {
        Config::save_preference_path(Preference::AskEnabled(enabled), &path)?;
        let text = fs::read_to_string(&path)?;
        let config = Config::parse(&text)?;
        assert!(text.contains("900 # keep"));
        assert_eq!(config.chat.ask_enabled, enabled);
        assert_eq!(config.chat.ask_agent, Some(chat::AskAgent::Pi));
    }
    Config::save_preference_path(Preference::AskAgent(None), &path)?;
    let config = Config::parse(&fs::read_to_string(path)?)?;
    assert!(!config.chat.ask_enabled && config.chat.ask_agent.is_none());
    Ok(())
}

#[test]
fn ask_starts_unconfigured_and_only_accepts_the_supported_agent() -> anyhow::Result<()> {
    assert!(Config::default().chat.ask_agent.is_none());
    let config = Config::parse("[chat]\nask_agent = 'pi'")?;
    assert_eq!(config.chat.ask_agent, Some(chat::AskAgent::Pi));
    assert!(Config::parse("[chat]\nask_agent = 'codex'").is_err());
    assert!(Config::parse("[chat]\nask_agent = 'claude'").is_err());
    Ok(())
}

#[test]
fn ask_agent_save_preserves_other_preferences() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("local.toml");
    fs::write(&path, "[chat]\nmax_width = 900 # keep\n")?;
    Config::save_preference_path(Preference::AskAgent(Some(chat::AskAgent::Pi)), &path)?;
    let text = fs::read_to_string(&path)?;
    assert!(text.contains("900 # keep"));
    assert_eq!(
        Config::parse(&text)?.chat.ask_agent,
        Some(chat::AskAgent::Pi)
    );
    Config::save_preference_path(Preference::AskAgent(None), &path)?;
    assert!(
        Config::parse(&fs::read_to_string(path)?)?
            .chat
            .ask_agent
            .is_none()
    );
    Ok(())
}

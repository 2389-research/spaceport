// ABOUTME: Checks explicit Core opt-in and preservation of personal settings.
// ABOUTME: Exercises the real configuration parser and targeted preference writer.
use super::*;
use crate::config::preferences::Preference;

#[test]
fn core_is_off_until_explicitly_enabled() -> anyhow::Result<()> {
    for text in ["", DEFAULT_CONFIG, "herdr_2_terminals = true"] {
        assert!(!Config::parse(text)?.herdr_core_enabled);
    }
    assert!(!Config::default().herdr_core_enabled);
    assert!(Config::parse("herdr_core_enabled = true")?.herdr_core_enabled);
    assert!(Config::parse("herdr_core_enabled = 'yes'").is_err());
    Ok(())
}

#[test]
fn core_opt_in_round_trips_without_changing_other_preferences() -> anyhow::Result<()> {
    let root = TempDirectory::new()?;
    let path = root.0.join("config.local.toml");
    fs::write(&path, "# personal\nherdr_2_terminals = true # retained\n")?;
    for enabled in [true, false] {
        Config::save_preference_path(Preference::HerdrCoreEnabled(enabled), &path)?;
        let text = fs::read_to_string(&path)?;
        let config = Config::parse(&text)?;
        assert_eq!(config.herdr_core_enabled, enabled);
        assert!(config.herdr_2_terminals);
        assert!(text.contains("# personal") && text.contains("# retained"));
    }
    Ok(())
}

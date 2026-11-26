// ABOUTME: Verifies all Quiet thresholds in saved settings and personal overrides.
// ABOUTME: Checks that the standard settings validator rejects unsupported choices.
use super::*;
use crate::{QuietAfter, preferences::Preference};

#[test]
fn quiet_after_choices_round_trip_through_personal_settings() -> anyhow::Result<()> {
    let temp = TempDirectory::new()?;
    let path = temp.0.join("config.local.toml");
    fs::write(&path, "theme = 'Nord' # keep\n")?;
    assert_eq!(Config::parse("")?.quiet_after, QuietAfter::OneDay);
    assert_eq!(
        Config::parse(DEFAULT_CONFIG)?.quiet_after,
        QuietAfter::OneDay
    );
    for (name, choice) in [
        ("24h", QuietAfter::OneDay),
        ("48h", QuietAfter::TwoDays),
        ("7d", QuietAfter::OneWeek),
    ] {
        Config::save_preference_path(Preference::QuietAfter(choice), &path)?;
        let text = fs::read_to_string(&path)?;
        let parsed = Config::parse(&text)?;
        assert_eq!(parsed.quiet_after, choice);
        assert!(parsed.unknown_keys.is_empty());
        assert!(text.contains(&format!("quiet_after = \"{name}\"")));
        assert!(text.contains("theme = 'Nord' # keep"));
    }
    assert!(Config::parse("quiet_after = '5h'").is_err());
    Ok(())
}

use super::*;

#[test]
fn config_reports_unknown_display_providers() {
    let parse = |text: &str| {
        let mut config: UsageConfig = toml::from_str(text).unwrap();
        let unknown = config.retain_known();
        (config, unknown)
    };
    let (config, unknown) = parse("show_providers = [\"claude\"]\nhide_providers = [\"codex\"]");
    assert!(config.shown(provider("claude")));
    assert!(config.hidden(provider("codex")));
    assert!(config.show);
    assert!(unknown.is_empty());
    // Names a newer build may know are dropped and reported, not fatal.
    let (config, unknown) =
        parse("show_providers = [\"nope\", \"claude\"]\nhide_providers = [\"later\"]");
    assert!(config.shown(provider("claude")));
    assert_eq!(config.show_providers, ["claude"]);
    assert!(config.hide_providers.is_empty());
    assert_eq!(
        unknown,
        ["usage.show_providers.nope", "usage.hide_providers.later",]
    );
}

// ABOUTME: Checks retained usage presentation and rejection of excluded sign-in configuration.
// ABOUTME: Keeps missing observations distinct from observed zero usage.
use super::*;

#[test]
fn sign_in_configuration_is_rejected() {
    for config in [
        "browser_cookies = false",
        "[providers.openai]\napi_key = 'fixture'",
        "[providers]",
        "providers = {}",
    ] {
        assert!(
            toml::from_str::<UsageConfig>(config).is_err(),
            "accepted {config}"
        );
    }
}

#[test]
fn display_preferences_and_zero_usage_remain_distinct_from_unavailable() {
    let mut config: UsageConfig = toml::from_str("show = false\ntopbar = false\ninline = false\nshow_providers = ['codex']\nhide_providers = ['claude']").unwrap();
    assert!(!config.show && !config.topbar && !config.inline);
    assert!(config.retain_known().is_empty());
    assert!(config.shown(provider("codex")));
    assert!(config.hidden(provider("claude")));
    let entry = super::super::Entry {
        readings: vec![
            Reading {
                provider: provider("codex"),
                report: Some(report(provider("codex"), 0.)),
                error: None,
            },
            Reading {
                provider: provider("claude"),
                report: None,
                error: None,
            },
        ],
        ..Default::default()
    };
    assert_eq!(entry.headline(None, &UsageConfig::default()).len(), 1);
    assert_eq!(
        entry.headline(None, &UsageConfig::default())[0]
            .report
            .as_ref()
            .unwrap()
            .tightest()
            .unwrap()
            .percent(),
        0
    );
    assert!(super::super::Usage::default().current().is_none());
}

#[test]
fn display_registry_preserves_identity_order_links_and_icons() {
    let providers: Vec<_> = registry::all().collect();
    let ids: std::collections::HashSet<_> =
        providers.iter().map(|provider| provider.id()).collect();
    assert_eq!(ids.len(), providers.len());
    for (index, provider) in providers.iter().enumerate() {
        assert_eq!(registry::find(provider.id()), Some(*provider));
        assert_eq!(registry::position(*provider), index);
        assert!(!provider.name().is_empty());
    }
    assert!(registry::position(provider("codex")) < registry::position(provider("openai")));
    assert!(registry::position(provider("openai")) < registry::position(provider("claude")));
    let openai = provider("openai").meta();
    assert_eq!(openai.name, "OpenAI");
    assert_eq!(openai.icon, Some("icons/agent-codex.svg"));
    assert_eq!(openai.dashboard, Some("https://platform.openai.com/usage"));
    assert_eq!(openai.status_page, Some("https://status.openai.com"));
    assert!(registry::find("unregistered-provider").is_none());
}

#[test]
fn report_details_survive_without_provider_implementations() {
    use crate::details::{ClaudeAccount, ClaudeAccountStatus, OpenaiDaily};
    let daily = report(provider("openai"), 30.).with_detail(OpenaiDaily {
        days: vec![("Oct 28".into(), 1.5)],
    });
    assert_eq!(
        daily.detail::<OpenaiDaily>().unwrap().days,
        [("Oct 28".into(), 1.5)]
    );
    let other = ClaudeAccount {
        email: "fixture@example.test".into(),
        alias: Some("Fixture".into()),
        disabled: false,
        status: ClaudeAccountStatus::Ok,
        stale: true,
        windows: vec![Window::new(Kind::Weekly, 12., None, None)],
    };
    let claude = report(provider("claude"), 0.).with_detail(vec![other]);
    let accounts = claude.detail::<Vec<ClaudeAccount>>().unwrap();
    assert_eq!(accounts[0].name(), "Fixture");
    assert_eq!(accounts[0].note(), Some("last known"));
    assert_eq!(accounts[0].windows[0].percent(), 12);
}

#[test]
fn hidden_reports_stay_out_of_panel_tabs_even_when_explicitly_shown() {
    let entry = super::super::Entry {
        readings: vec![Reading {
            provider: provider("claude"),
            report: Some(report(provider("claude"), 25.)),
            error: None,
        }],
        ..Default::default()
    };
    let config: UsageConfig =
        toml::from_str("show_providers = ['claude']\nhide_providers = ['claude']").unwrap();
    assert!(entry.tabs(&config).is_empty());
    assert!(entry.headline(Some(provider("claude")), &config).is_empty());
    assert!(entry.visible_readings(&config).next().is_none());
    assert_eq!(
        entry.readings[0]
            .report
            .as_ref()
            .unwrap()
            .tightest()
            .unwrap()
            .percent(),
        25
    );
}

#[test]
fn ordered_metadata_matches_original_runtime_getters() {
    let baseline: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/provider_metadata.json")).unwrap();
    let expected = baseline["providers"].as_array().unwrap();
    let actual: Vec<_> = registry::all()
        .map(|provider| {
            let meta = provider.meta();
            serde_json::json!({
                "id": meta.id, "name": meta.name, "icon": meta.icon,
                "dashboard": meta.dashboard, "status_page": meta.status_page,
            })
        })
        .collect();
    assert_eq!(actual.len(), expected.len());
    let mismatches: Vec<_> = actual
        .iter()
        .zip(expected)
        .enumerate()
        .filter(|(_, (actual, expected))| actual != expected)
        .map(|(index, (actual, expected))| {
            format!("position {index}: actual {actual}, expected {expected}")
        })
        .collect();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

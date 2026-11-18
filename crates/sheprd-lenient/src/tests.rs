use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Style {
    #[default]
    Dots,
    Symbols,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Table {
    #[serde(deserialize_with = "super::or_default")]
    style: Style,
    #[serde(deserialize_with = "super::or_true")]
    enabled: bool,
    #[serde(deserialize_with = "super::or_default")]
    name: Option<String>,
    #[serde(deserialize_with = "super::entries")]
    agents: HashMap<String, Style>,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            style: Style::Dots,
            enabled: true,
            name: None,
            agents: HashMap::new(),
        }
    }
}

#[test]
fn each_unreadable_value_falls_back_on_its_own() {
    let table: Table = toml::from_str(
        "style = 'bars'\nenabled = 'yes'\nname = 7\n[agents]\na = 'symbols'\nb = 'later'",
    )
    .unwrap();
    assert_eq!(table.style, Style::Dots);
    assert!(table.enabled);
    assert_eq!(table.name, None);
    assert_eq!(table.agents, HashMap::from([("a".into(), Style::Symbols)]));

    let table: Table =
        toml::from_str("style = 'symbols'\nenabled = false\nname = 'x'\nagents = 3").unwrap();
    assert_eq!(table.style, Style::Symbols);
    assert!(!table.enabled);
    assert_eq!(table.name.as_deref(), Some("x"));
    assert!(table.agents.is_empty());

    assert!(toml::from_str::<Table>("style = [").is_err());
}

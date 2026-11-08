use super::*;

#[test]
fn old_auto_stop_values_are_preserved_without_affecting_connection_settings() {
    let expected = Settings::resolve(&ContainerConfig::default()).unwrap();
    for value in ["30m", "1m"] {
        let config = ContainerConfig {
            auto_stop: Some(value.into()),
            ..Default::default()
        };
        assert_eq!(Settings::resolve(&config).unwrap(), expected);
        assert_eq!(config.auto_stop.as_deref(), Some(value));
    }
}

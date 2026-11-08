#![allow(clippy::unwrap_used)]
use super::*;
use std::fs;

#[test]
fn saving_writes_only_edited_keys_and_validates_first() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config-gpui.local.toml");
    fs::write(
        &path,
        "theme = 'Nord'\n\n[container]\nimage = 'debian:12' # mine\ncpus = 2\npids_limit = 512\nmemory = '4g'\nauto_stop = '30m'\n\n[cloud]\nagents = ['future-agent'] # keep\nsetup = 'echo old'\n\n[cloud.container]\nsetup = 'echo saved'\n",
    )
    .unwrap();
    let fields = ContainerFields {
        engine: "podman".into(),
    };
    Config::save_container_path(&fields, &path).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("theme = 'Nord'"));
    assert!(text.contains("image = 'debian:12' # mine"));
    assert!(text.contains("agents = ['future-agent'] # keep"));
    assert!(text.contains("setup = 'echo saved'"));
    assert!(
        text.contains("pids_limit = 512"),
        "keys Settings does not edit stay"
    );
    let parsed: toml::Value = toml::from_str(&text).unwrap();
    assert_eq!(parsed["container"]["memory"].as_str(), Some("4g"));
    assert_eq!(parsed["container"]["cpus"].as_integer(), Some(2));
    assert_eq!(parsed["container"]["engine"].as_str(), Some("podman"));
    assert_eq!(parsed["container"]["auto_stop"].as_str(), Some("30m"));
    let broken = ContainerFields {
        engine: "lxc".into(),
    };
    assert!(broken.validate(&ContainerConfig::default()).is_err());
}

#[test]
fn loading_reads_the_table_and_refuses_bad_values() {
    let config = Config::parse("[container]\nengine = \"podman\"\ncpus = 2\n").unwrap();
    assert_eq!(config.container.engine.as_deref(), Some("podman"));
    assert_eq!(config.container.cpus.as_deref(), Some("2"));
    let old = Config::parse("[container]\nauto_stop = \"1m\"\n").unwrap();
    assert_eq!(old.container.auto_stop.as_deref(), Some("1m"));
    assert!(Config::parse("[container]\nengine = \"lxc\"\n").is_err());
    assert!(Config::parse("[container]\nunknown = 1\n").is_err());
}

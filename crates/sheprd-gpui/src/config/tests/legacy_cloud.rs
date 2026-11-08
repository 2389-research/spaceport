use super::*;

#[test]
fn retired_cloud_setup_values_load_as_inert_data() {
    let config = Config::parse(
        "[cloud]\nagents = ['claude', 'future-agent']\nsetup = 'echo old'\n\
         [cloud.container]\nagents = ['codex']\nsetup = 'echo container'\n",
    )
    .unwrap();

    assert_eq!(
        config.cloud.agents.as_deref(),
        Some(&["claude".into(), "future-agent".into()][..])
    );
    assert_eq!(config.cloud.setup.as_deref(), Some("echo old"));
    assert_eq!(
        config.cloud.container.agents.as_deref(),
        Some(&["codex".into()][..])
    );
    assert_eq!(
        config.cloud.container.setup.as_deref(),
        Some("echo container")
    );
}

use super::*;

#[test]
fn retired_cloud_tables_do_not_block_other_settings() {
    let config = Config::parse(
        "show_device_icons = false\n[cloud]\nagents = ['claude']\nsetup = 'echo old'\n\
         [container]\nengine = 'podman'\n",
    )
    .unwrap();

    assert!(!config.show_device_icons);
    assert!(
        config
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("cloud"))
    );
    assert!(
        config
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("container"))
    );
}

// ABOUTME: Reports removed dashboard settings rather than applying them.
// ABOUTME: Keeps retained service and network settings usable alongside old keys.
use super::*;

#[test]
fn resource_dashboard_settings_are_unknown_and_service_settings_still_load() -> anyhow::Result<()> {
    let config = Config::parse(
        "show_system_load = true\nshow_gpu_load = true\nshow_sidebar_gpu_load = true\n\
         show_listening_ports = false\nshow_network_quality = true\n\
         [status_bar]\nsystem_load = 'compact'\ngpu_detail = 'compact'\n",
    )?;
    for key in [
        "show_system_load",
        "show_gpu_load",
        "show_sidebar_gpu_load",
        "status_bar.system_load",
        "status_bar.gpu_detail",
    ] {
        assert!(
            config.unknown_keys.iter().any(|unknown| unknown == key),
            "{key}"
        );
    }
    assert!(!config.show_listening_ports);
    assert!(config.show_network_quality);
    Ok(())
}

use super::*;
use crate::config::DeviceFooter;

#[test]
fn device_footer_is_named_and_defaults_to_always() -> anyhow::Result<()> {
    for config in [Config::default(), Config::parse(DEFAULT_CONFIG)?] {
        assert_eq!(config.sidebar_style.device_footer, DeviceFooter::Always);
    }
    for (name, mode) in [
        ("always", DeviceFooter::Always),
        ("auto", DeviceFooter::Auto),
        ("never", DeviceFooter::Never),
    ] {
        assert_eq!(
            Config::parse(&format!("[sidebar]\ndevice_footer = \"{name}\""))?
                .sidebar_style
                .device_footer,
            mode
        );
    }
    for bad in ["\"hidden\"", "\"Auto\"", "true"] {
        assert!(Config::parse(&format!("[sidebar]\ndevice_footer = {bad}")).is_err());
    }
    Ok(())
}

#[test]
fn auto_device_footer_needs_a_second_device() {
    for (devices, shown) in [(1, false), (2, true), (0, false)] {
        assert_eq!(
            DeviceFooter::Auto.shown(devices),
            shown,
            "{devices} devices"
        );
        assert!(DeviceFooter::Always.shown(devices));
        assert!(!DeviceFooter::Never.shown(devices));
    }
}

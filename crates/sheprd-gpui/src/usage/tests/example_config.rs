// ABOUTME: Keeps the shipped example restricted to supported usage display preferences.
// ABOUTME: Ensures excluded sign-in setup cannot reappear in generated config content.

#[test]
fn example_config_has_no_provider_sign_in_setup() {
    let example = include_str!("../../../config-gpui.example.toml");
    for key in ["usage.providers", "browser_cookies", "OPENAI_ADMIN_KEY"] {
        assert!(!example.contains(key), "excluded key: {key}");
    }
    for key in [
        "show = true",
        "topbar = true",
        "inline = true",
        "show_providers",
        "hide_providers",
    ] {
        assert!(example.contains(key), "missing display preference: {key}");
    }
}

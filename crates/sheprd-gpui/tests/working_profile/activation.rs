// ABOUTME: Refuses foreground profile claims from inactive or hidden native observations.
// ABOUTME: Exercises the typed observation consumer without launching or mocking a window.
use super::*;

#[test]
fn foreground_requires_active_visible_key_window_and_active_unhidden_app() -> Result<()> {
    let ready = json!({"request_id":1,"gui_pid":1,"release":true,"window_id":1,
        "window_active":true,"window_visible":true,"native_key":true,"app_active":true,
        "app_hidden":false,"native_activation":"synthetic activation contract fixture",
        "workspace":null,"synthetic_working_panes":[],"groups":[]});
    assert!(foreground(&serde_json::from_value(ready.clone())?));
    for (field, value) in [
        ("window_active", false),
        ("window_visible", false),
        ("native_key", false),
        ("app_active", false),
        ("app_hidden", true),
    ] {
        let mut blocked = ready.clone();
        blocked[field] = json!(value);
        assert!(
            !foreground(&serde_json::from_value(blocked)?),
            "qualified {field}={value}"
        );
    }
    Ok(())
}

// ABOUTME: Refuses foreground profile claims from inactive or hidden native observations.
// ABOUTME: Exercises the typed observation consumer without launching or mocking a window.
use super::*;

#[test]
fn foreground_requires_active_visible_key_window_and_active_unhidden_app() -> Result<()> {
    let ready = json!({"request_id":1,"gui_pid":1,"release":true,"window_id":1,
        "reduce_motion":false,"mark_layer_render_calls":0,
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

#[test]
fn windows_foreground_requires_the_exact_owned_hwnd_and_its_actual_thread() -> Result<()> {
    let ready = json!({"request_id":1,"gui_pid":42,"release":true,"window_id":1,
        "reduce_motion":false,"mark_layer_render_calls":0,
        "window_active":true,"window_visible":true,"native_key":null,"app_active":null,
        "app_hidden":null,"native_activation":"Windows contract fixture","windows_hwnd":1234,
        "windows_native":{"hwnd":1234,"gui_pid":42,"thread_id":7,"foreground_hwnd":1234,
            "visible":true,"minimized":false},
        "workspace":null,"synthetic_working_panes":[],"groups":[]});
    assert!(foreground(&serde_json::from_value(ready.clone())?));
    for (field, value) in [
        ("hwnd", json!(999)),
        ("gui_pid", json!(999)),
        ("thread_id", json!(0)),
        ("foreground_hwnd", json!(999)),
        ("visible", json!(false)),
        ("minimized", json!(true)),
    ] {
        let mut blocked = ready.clone();
        blocked["windows_native"][field] = value;
        assert!(
            !foreground(&serde_json::from_value(blocked)?),
            "qualified invalid {field}"
        );
    }
    let mut absent = ready;
    absent["windows_native"] = json!(null);
    assert!(!foreground(&serde_json::from_value(absent)?));
    Ok(())
}

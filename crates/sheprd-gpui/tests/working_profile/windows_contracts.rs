// ABOUTME: Requires Windows CPU clocks and native facts through the real profile schema.
// ABOUTME: Keeps platform qualification distinct from macOS AppKit observations.
#[path = "measurements.rs"]
mod measurements;
#[path = "../../src/working_profile/report.rs"]
mod report;

use anyhow::Result;
use serde_json::json;
use std::time::Duration;

#[test]
fn windows_cpu_clock_preserves_100ns_ticks() -> Result<()> {
    assert_eq!(
        measurements::cpu_ticks("12500001\r\n")?,
        Duration::new(1, 250_000_100)
    );
    Ok(())
}

#[test]
fn windows_cpu_ticks_refuse_invalid_or_overflowing_timespan_values() {
    for value in ["", "-1", "+1", "1.5", "1 2", "9223372036854775808"] {
        assert!(
            measurements::cpu_ticks(value).is_err(),
            "accepted {value:?}"
        );
    }
}

#[test]
fn windows_observation_does_not_invent_appkit_facts() -> Result<()> {
    let observation: report::Observation = serde_json::from_value(json!({
        "request_id":1,"gui_pid":42,"release":true,"window_id":1,
        "window_active":true,"window_visible":true,
        "native_key":null,"app_active":null,"app_hidden":null,
        "native_activation":"Win32 HWND supplied by the real window",
        "windows_hwnd":1234,"workspace":null,"synthetic_working_panes":[],"groups":[]
    }))?;
    let actual = serde_json::to_value(observation)?;
    assert!(actual["native_key"].is_null());
    assert_eq!(actual["windows_hwnd"], 1234);
    Ok(())
}

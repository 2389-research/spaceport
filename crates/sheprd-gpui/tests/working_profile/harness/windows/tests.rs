// ABOUTME: Exercises actual Windows profile helper processes and read-only recorder qualification.
// ABOUTME: Keeps GUI and recording prerequisites explicit while ordinary controls use no daemon.
use super::*;

#[test]
fn windows_cpu_helper_reads_the_actual_test_process_clock() -> Result<()> {
    let fixture = Fixture {
        sandbox: Some(Sandbox::new()),
        gui: None,
        server: None,
        request: 0,
    };
    let started = Instant::now();
    let before = fixture.cpu("cpu-before", std::process::id())?;
    let after = fixture.cpu("cpu-after", std::process::id())?;
    assert!(measurements::process_cpu(before, after, started.elapsed())? >= 0.);
    Ok(())
}

#[test]
fn windows_native_helper_rejects_an_invalid_window_without_stealing_focus() -> Result<()> {
    let fixture = Fixture {
        sandbox: Some(Sandbox::new()),
        gui: None,
        server: None,
        request: 0,
    };
    let observation = serde_json::from_value(json!({
        "request_id":1,"gui_pid":std::process::id(),"release":true,"window_id":1,
        "reduce_motion":false,"mark_layer_render_calls":0,
        "window_active":true,"window_visible":true,"native_activation":"invalid real HWND control",
        "windows_hwnd":0,"workspace":null,"synthetic_working_panes":[],"groups":[]
    }))?;
    let error = fixture
        .native_observation(observation)
        .err()
        .context("invalid HWND unexpectedly passed native observation")?;
    eprintln!("WORKING PROFILE invalid HWND control: {error:#}");
    let diagnostic = fs::read_to_string(fixture.sandbox()?.dir.join("native-1.stderr.log"))?;
    assert!(
        diagnostic.contains("Profile HWND no longer belongs to the exact GUI process"),
        "native helper failed before HWND validation: {diagnostic}; full failure: {error:#}"
    );
    let stages = fs::read_to_string(fixture.sandbox()?.dir.join("native-1.stages.txt"))?;
    assert!(
        stages.contains("utility_import_complete")
            && stages.contains("add_type_complete")
            && stages.contains("ownership_complete thread=0 owner=0"),
        "native helper did not complete expected stages: {stages}"
    );
    Ok(())
}

#[test]
#[ignore = "requires explicit installed xperf; only queries WPR status and active loggers"]
fn read_only_windows_recorder_inventory() -> Result<()> {
    let fixture = Fixture {
        sandbox: Some(Sandbox::new()),
        gui: None,
        server: None,
        request: 0,
    };
    fixture.recorder_available("inventory").map(|_| ())
}

#[test]
fn completed_measurement_survives_a_real_stop_process_failure() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("measurement.json");
    let measurement = json!({"cpu_before_seconds":1.25,"cpu_after_seconds":1.75,"wall_seconds":5.5,
        "stop_outcome":"pending; CPU interval completed","fixture":"serialization contract"});
    let result = retain_before_stop(&path, &measurement, || {
        match sheprd_client::new_command(directory.path().join("missing-stop-helper.exe")).spawn() {
            Err(error) => Err(error.into()),
            Ok(mut child) => {
                let _ = child.kill();
                let _ = child.wait();
                anyhow::bail!("missing stop process unexpectedly started")
            }
        }
    });
    assert!(result.is_err());
    let actual: Value = serde_json::from_slice(&fs::read(path)?)?;
    assert_eq!(actual, measurement);
    Ok(())
}

// ABOUTME: Qualifies real Herdr images through an owned native Windows terminal.
// ABOUTME: Keeps producer commands, wheel input, captures, and cleanup inside one private sandbox.
use super::{CAPTURE_LIMIT, Duration, GuiTarget, Instant, Read, Running, fs, thread};
use std::{
    collections::BTreeSet,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    time::{SystemTime, UNIX_EPOCH},
};

fn deadline_argument(limit: Duration) -> String {
    (SystemTime::now() + limit)
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

fn helper_log(running: &Running) -> String {
    let mut bytes = Vec::new();
    fs::File::open(running.sandbox.dir.join("uia-images.log"))
        .unwrap()
        .take((CAPTURE_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .unwrap();
    assert!(
        bytes.len() <= CAPTURE_LIMIT,
        "native image helper exceeds log bound"
    );
    String::from_utf8(bytes).unwrap()
}

fn panes(running: &mut Running, workspace: &str) -> BTreeSet<String> {
    let result = running.daemon_result(&["pane", "list", "--workspace", workspace]);
    assert_eq!(result["type"], "pane_list");
    result["panes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pane| pane["pane_id"].as_str().unwrap().to_owned())
        .collect()
}

fn wait_view(running: &mut Running, deadline: Instant) -> serde_json::Value {
    loop {
        let log = running.gui_log();
        assert!(!log.contains("IMAGE_NATIVE_FAIL "), "{log}");
        if let Some(line) = log
            .lines()
            .find_map(|line| line.strip_prefix("IMAGE_NATIVE_VIEW "))
        {
            return serde_json::from_str(line).unwrap();
        }
        if let Some(status) = running.observer.as_mut().unwrap().try_wait().unwrap() {
            panic!(
                "native image input helper exited before producer readiness: {status}: {}",
                helper_log(running)
            );
        }
        assert!(
            running.gui.as_mut().unwrap().try_wait().unwrap().is_none(),
            "owned image GUI exited"
        );
        assert!(
            Instant::now() < deadline,
            "native image pane readiness missing: {log}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore = "requires active Windows desktop and HERDR_TEST_BINARY; uses only an authored private image pane"]
fn native_images_preserve_mapping_layers_clipping_and_cached_cell_replay() {
    let mut running = Running::prepared();
    let token = uuid::Uuid::new_v4();
    fs::write(
        running
            .sandbox
            .dir
            .join("config/spaceport/config.local.toml"),
        "language = \"en-US\"\nsmooth_scroll = true\n",
    )
    .unwrap();
    for (name, source) in [
        (
            "image_scene.ps1",
            include_str!("../support/image_scene.ps1"),
        ),
        (
            "native_image_input.cs",
            include_str!("../support/native_image_input.cs"),
        ),
        ("uia_images.ps1", include_str!("../support/uia_images.ps1")),
    ] {
        fs::write(running.sandbox.dir.join(name), source).unwrap();
    }
    running.start_daemon();
    running.start_gui(GuiTarget::Images(token));
    running.wait_tree("private image terminal", |tree| {
        tree.contains("Document 'Terminal' [Value, Text]")
    });

    // The first helper proves restoration after failure without sending any wheel input.
    let capture = running.sandbox.dir.join("uia-images-failure.log");
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_images.ps1"),
        "uia-images-failure.log",
    );
    command.args([
        "-PointerFailure",
        "-DeadlineEpochMs",
        &deadline_argument(Duration::from_secs(25)),
    ]);
    let failure = running
        .command_output(command, &capture, Duration::from_secs(25))
        .unwrap_err();
    let failure = failure.to_string();
    assert!(
        failure.contains("Intentional native image pointer failure"),
        "{failure}"
    );
    assert!(
        failure.contains("IMAGE_INPUT_POINTER_RELEASE restored=True"),
        "{failure}"
    );

    let limit = Duration::from_secs(45);
    let deadline = Instant::now() + limit;
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_images.ps1"),
        "uia-images.log",
    );
    command.args(["-DeadlineEpochMs", &deadline_argument(limit)]);
    running.observer = Some(command.spawn().unwrap());
    let result = catch_unwind(AssertUnwindSafe(|| {
        let view = wait_view(&mut running, deadline);
        let rows = view["rows"]
            .as_u64()
            .filter(|rows| (22..=100).contains(rows))
            .unwrap();
        let pane = view["pane"].as_str().unwrap();
        let tabs = running.daemon_result(&["tab", "list"]);
        assert_eq!(tabs["type"], "tab_list");
        let tabs = tabs["tabs"].as_array().unwrap();
        assert_eq!(tabs.len(), 1);
        let workspace = tabs[0]["workspace_id"].as_str().unwrap();
        let before = panes(&mut running, workspace);
        assert_eq!(before, BTreeSet::from([pane.to_owned()]));
        let producer = running.sandbox.dir.join("image_scene.ps1");
        let producer = producer.to_str().unwrap().replace('\'', "''");
        let invocation = format!("& '{producer}' -Rows {rows} -Token '{token}'");
        // Queue success is not image delivery; the observer waits for exact authored pixels.
        running.daemon_result(&["pane", "run", pane, &invocation]);
        loop {
            if let Some(status) = running.observer.as_mut().unwrap().try_wait().unwrap() {
                let output = helper_log(&running);
                assert!(
                    status.success()
                        && output.contains(
                            "IMAGE_INPUT native=true wheel=1 pointer_restored=true released=true"
                        ),
                    "{status}\n{output}"
                );
                running.observer.take();
                break;
            }
            let log = running.gui_log();
            assert!(!log.contains("IMAGE_NATIVE_FAIL "), "{log}");
            assert!(
                Instant::now() < deadline,
                "native image helper exceeded its parent deadline: {log}"
            );
            thread::sleep(Duration::from_millis(10));
        }
        let log = running.gui_log();
        assert!(log.contains("IMAGE_NATIVE_PASS assets=true cached_cells=true fractional=true clipping=true layers=true"), "{log}");
        for phase in ["stationary", "replay", "fractional", "settled"] {
            let prefix = format!("IMAGE_NATIVE_PNG {phase} ");
            let lines: Vec<_> = log
                .lines()
                .filter(|line| line.starts_with(&prefix))
                .collect();
            assert_eq!(lines.len(), 1, "expected one bounded native {phase} image");
            eprintln!("{}", lines[0]);
        }
        for line in log
            .lines()
            .filter(|line| line.starts_with("IMAGE_NATIVE_") && !line.contains("_PNG "))
        {
            eprintln!("{line}");
        }
        assert_eq!(
            panes(&mut running, workspace),
            before,
            "native image test changed pane ownership"
        );
    }));
    let released = fs::write(running.sandbox.dir.join("image-release.request"), b"1");
    let producer_done = fs::write(running.sandbox.dir.join("image-done.request"), b"1");
    // Keep the helper alive for its finally block or independent pointer watchdog before dropping Running.
    let mut wait_error = None;
    while let Some(helper) = &mut running.observer {
        match helper.try_wait() {
            Ok(Some(_)) => {
                running.observer.take();
                break;
            }
            Ok(None) => {}
            Err(error) => wait_error = Some(error),
        }
        if Instant::now() >= deadline {
            crate::stop_children([&mut running.observer]);
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    if let Err(panic) = result {
        if let Some(error) = wait_error {
            eprintln!(
                "Native image helper observation failed during original failure cleanup: {error}"
            );
        }
        resume_unwind(panic);
    }
    released.unwrap();
    producer_done.unwrap();
    assert!(
        wait_error.is_none(),
        "native image helper cleanup failed: {wait_error:?}"
    );
    eprintln!(
        "IMAGE_NATIVE native PASS: real authored daemon assets, cached cells, fractional wheel, native pixels, layer order, clipping, pointer failure cleanup and stable pane IDs"
    );
}

// ABOUTME: Observes real pane-console input and native glyph geometry for Windows shortcuts.
// ABOUTME: Uses trusted preparation commands before sending each measured scan-code case.
use super::{
    Case, Duration, Instant, Outcome, Running, cases, diagnostics, fixture, native,
    native_with_font, same_scene, snapshot, window_state,
};
use std::{collections::BTreeSet, fs, thread};

pub(super) fn prepare_input(running: &mut Running, command: Option<&str>) {
    let capture = running.sandbox.dir.join("shell-prompt.log");
    let mut probe = running.powershell(
        &running.sandbox.dir.join("uia_shortcut_matrix.ps1"),
        "shell-prompt.log",
    );
    probe.env("SPACEPORT_DISPOSABLE_LAYOUT_TEST", "github-hosted-windows");
    let deadline = (std::time::SystemTime::now() + Duration::from_secs(40))
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    probe.args([
        "-CaseId",
        "console-attachment",
        "-DeadlineEpochMs",
        &deadline,
        "-WaitPrompt",
    ]);
    if let Some(command) = command {
        probe.args(["-PrepareInput", command]);
    }
    let output = match running.command_output(probe, &capture, Duration::from_secs(40)) {
        Ok(output) => output,
        Err(error) => {
            diagnostics(running, "trusted pane preparation");
            panic!("trusted pane preparation failed: {error}");
        }
    };
    assert!(
        output.contains("WINDOWS_KEYS shell_prompt=true input=false"),
        "{output}"
    );
    if command.is_some() {
        assert!(
            output.contains("WINDOWS_KEYS pane_preparation=true actual_gui_input=true"),
            "{output}"
        );
    }
}

fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

fn pane_state(running: &mut Running) -> serde_json::Value {
    let capture = running.sandbox.dir.join("pane-state.log");
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_shortcut_matrix.ps1"),
        "pane-state.log",
    );
    let deadline = (std::time::SystemTime::now() + Duration::from_secs(40))
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    command.args([
        "-CaseId",
        "console-attachment",
        "-DeadlineEpochMs",
        &deadline,
        "-ObserveState",
    ]);
    let output = running
        .command_output(command, &capture, Duration::from_secs(40))
        .unwrap();
    serde_json::from_str(output.lines().find(|line| line.starts_with('{')).unwrap()).unwrap()
}

fn receipt(running: &mut Running, kind: &str, nonce: &str, id: &str) -> serde_json::Value {
    let path = running
        .sandbox
        .dir
        .join(format!("console-{kind}-{nonce}-{id}.json"));
    let deadline = Instant::now() + Duration::from_secs(40);
    loop {
        match fs::read(&path) {
            Ok(bytes) => {
                assert!(bytes.len() <= 4096, "console receipt exceeds bound");
                let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(value["nonce"].as_str(), Some(nonce));
                assert_eq!(value["id"].as_str(), Some(id));
                assert_eq!(value["attached"].as_bool(), Some(true));
                return value;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("private console receipt failed: {error}"),
        }
        if Instant::now() >= deadline {
            diagnostics(running, &format!("console {kind} receipt {id}"));
            panic!("private console receipt did not arrive: {id}");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
#[ignore = "requires disposable Windows desktop and explicit HERDR_TEST_BINARY; reads actual pane-console keys"]
fn native_pane_program_keeps_bare_function_and_control_page_keys() {
    let cases = cases();
    let mut running = fixture();
    let before = snapshot(&running);
    let before_windows = window_state(&mut running);
    let before_ui = pane_state(&mut running);
    assert_eq!(before_ui["status"].as_u64(), Some(1));
    assert_eq!(before_ui["sidebar"].as_u64(), Some(1));
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let script = running.sandbox.dir.join("console_shortcuts.ps1");
    prepare_input(
        &mut running,
        Some(&format!(
            "& {} -Nonce {}",
            quote(script.to_str().unwrap()),
            quote(&nonce)
        )),
    );
    let mut producer = None;
    for case in cases
        .iter()
        .filter(|case| matches!(case.outcome, Outcome::PaneKey { .. }))
    {
        let ready = receipt(&mut running, "ready", &nonce, &case.id);
        let actual = ready["producer"].as_u64().unwrap();
        assert_ne!(actual, u64::from(running.gui.as_ref().unwrap().id()));
        if let Some(expected) = producer {
            assert_eq!(actual, expected);
        } else {
            producer = Some(actual);
        }
        // The sentinel must return from ReadKey before any disputed pane key is sent.
        native(&mut running, case);
        let received = receipt(&mut running, "result", &nonce, &case.id);
        assert_eq!(received["producer"].as_u64(), Some(actual));
        let Outcome::PaneKey { key, modifiers } = &case.outcome else {
            unreachable!()
        };
        assert_eq!(received["key"].as_str(), Some(key.as_str()));
        let actual: BTreeSet<_> = received["modifiers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();
        let expected: BTreeSet<_> = modifiers.iter().map(String::as_str).collect();
        assert_eq!(
            actual, expected,
            "pane program received different modifiers"
        );
        let after = snapshot(&running);
        same_scene(&before, &after);
        assert_eq!(after.focused_tab_id, before.focused_tab_id);
        assert_eq!(after.focused_pane_id, before.focused_pane_id);
        assert_eq!(window_state(&mut running), before_windows);
        assert_eq!(pane_state(&mut running), before_ui);
        eprintln!(
            "WINDOWS_KEYS pane_receipt id={} key={key} modifiers={modifiers:?} attached=true",
            case.id
        );
    }
}

pub(super) fn fonts(running: &mut Running, cases: &[Case]) {
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let marker = format!("SP_FONT_{nonce}");
    prepare_input(
        running,
        Some(&format!("Write-Output ('SP_FONT_' + {})", quote(&nonce))),
    );
    let reset = cases
        .iter()
        .find(|case| case.outcome == Outcome::FontReset)
        .unwrap();
    let output = native_with_font(running, reset, Some(&marker), None, None, true);
    let configured = font_width(&output);
    let mut previous = configured;
    for (outcome, points) in [
        (Outcome::FontLarger, "20"),
        (Outcome::FontSmaller, "24"),
        (Outcome::FontReset, "28"),
    ] {
        let case = cases.iter().find(|case| case.outcome == outcome).unwrap();
        running.control(&["ui", "font-size", points]);
        let expected = (outcome == Outcome::FontReset).then_some(configured);
        let output = native_with_font(
            running,
            case,
            Some(&marker),
            expected,
            Some(previous),
            false,
        );
        previous = font_width(&output);
    }
}

fn font_width(output: &str) -> f64 {
    let value: serde_json::Value =
        serde_json::from_str(output.lines().find(|line| line.starts_with('{')).unwrap()).unwrap();
    value["width"].as_f64().unwrap()
}

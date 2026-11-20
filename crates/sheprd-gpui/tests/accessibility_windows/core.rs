// ABOUTME: Qualifies a real isolated Core terminal through native Windows accessibility.
// ABOUTME: Owns the exact runtime, pipe, GUI, and state without installing or discovering a daemon.
use super::{CAPTURE_LIMIT, Duration, GuiTarget, Instant, Running, Sandbox, fs, thread};
use sheprd_core_client::{Attachment, Core, wire::Attach};
use std::{io, process::Command};

#[path = "../../../test-support/core_windows.rs"]
mod runtime;
use runtime::Runtime;

#[path = "core/resize.rs"]
mod resize;

#[derive(Clone, Copy)]
pub(super) struct Target(runtime::Target);

impl Target {
    pub(super) fn apply(&self, command: &mut Command) {
        self.0.apply(command);
        command.env(
            "SPACEPORT_TEST_CORE_FRAME_OBSERVER",
            self.0.nonce().simple().to_string(),
        );
    }
}

struct Fixture {
    core: Option<Core>,
    dir: Option<tempfile::TempDir>,
    running: Running,
    runtime: Option<Runtime>,
    target: Target,
}

impl Fixture {
    fn start() -> Self {
        let binary = runtime::binary();
        let target = Target(runtime::Target::new());
        let mut fixture = Self {
            core: None,
            dir: Some(
                tempfile::Builder::new()
                    .prefix("sp-core-")
                    .tempdir()
                    .unwrap(),
            ),
            running: Running::prepared(),
            runtime: None,
            target,
        };
        fixture.runtime = Some(Runtime::start(
            &fixture.running.sandbox,
            &binary,
            fixture.dir.as_ref().unwrap().path(),
            target.0,
        ));
        fixture.core = Some(fixture.runtime.as_mut().unwrap().connect());
        fs::write(
            fixture
                .running
                .sandbox
                .dir
                .join("config/spaceport/config.local.toml"),
            "language = \"en-US\"\nherdr_core_enabled = true\n",
        )
        .unwrap();
        fixture.running.start_daemon();
        fixture.running.start_gui(GuiTarget::Core(fixture.target));
        fixture
            .running
            .wait_tree("private startup before Core command", |tree| {
                tree.contains("Document 'Terminal' [Value, Text]")
            });
        fixture
    }

    fn api(&self) -> &Core {
        self.core.as_ref().unwrap()
    }

    fn open_terminal(&mut self) -> String {
        self.running
            .control(&["run", "--json", "new_core_terminal"]);
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let terminals = self.api().list(false).unwrap().terminals;
            if let [terminal] = terminals.as_slice() {
                let address = terminal.address.clone();
                if let [tab] = self.tab_ids(&address).as_slice() {
                    self.running
                        .control(&["tab", "--json", "focus", &tab.to_string()]);
                    return address;
                }
            }
            assert!(
                Instant::now() < deadline,
                "Core command did not create exactly one app-owned terminal"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn tab_ids(&mut self, terminal: &str) -> Vec<u64> {
        let tabs: serde_json::Value =
            serde_json::from_str(&self.running.control_output(&["tab", "--json", "list"])).unwrap();
        tabs["tabs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|tab| tab["kind"] == "core_terminal" && tab["location"] == terminal)
            .map(|tab| tab["id"].as_u64().unwrap())
            .collect()
    }

    fn terminal_helper(&self, action: &str, capture: &str) -> Command {
        for (name, source) in [
            (
                "uia_terminal.ps1",
                include_str!("../support/uia_terminal.ps1"),
            ),
            ("uia_live.cs", include_str!("../support/uia_live.cs")),
            (
                "native_tab_capture.cs",
                include_str!("../support/native_tab_capture.cs"),
            ),
        ] {
            fs::write(self.running.sandbox.dir.join(name), source).unwrap();
        }
        let mut command = self
            .running
            .powershell(&self.running.sandbox.dir.join("uia_terminal.ps1"), capture);
        command.args(["-Action", action]);
        if action == "Text" {
            command
                .args(["-CapturePixels", "-CoreToken"])
                .arg(self.dir.as_ref().unwrap().path().file_name().unwrap());
        }
        command
    }

    fn frame_logs(&mut self) -> io::Result<String> {
        let capture = self.running.sandbox.dir.join("core-frame-log-capture.json");
        let mut command = self.running.sandbox.command(
            env!("CARGO_BIN_EXE_spaceport"),
            "core-frame-log-capture.json",
        );
        command.args(["logs", "--json", "--lines", "128"]);
        self.running
            .command_output(command, &capture, Duration::from_secs(5))
    }

    fn cleanup(&mut self) -> io::Result<()> {
        super::stop_children([
            &mut self.running.command,
            &mut self.running.observer,
            &mut self.running.gui,
        ]);
        self.core.take();
        let Some(runtime) = self.runtime.take() else {
            return Ok(());
        };
        runtime.shutdown(|command, capture, limit| {
            self.running
                .command_output(command, capture, limit)
                .map(|_| ())
        })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            let preserved = self.dir.take().map(tempfile::TempDir::keep);
            eprintln!("CORE_NATIVE cleanup failed; preserved private state {preserved:?}: {error}");
            if !thread::panicking() {
                panic!("private Core cleanup failed: {error}");
            }
        }
    }
}

fn runtime_output_seen(
    observed: &Attachment,
    marker: &[u8],
    limit: Duration,
) -> anyhow::Result<bool> {
    let deadline = Instant::now() + limit;
    let mut seen = Vec::new();
    while !seen.windows(marker.len()).any(|bytes| bytes == marker) {
        if Instant::now() >= deadline {
            return Ok(false);
        }
        let chunk = observed.read(Duration::from_millis(100))?;
        anyhow::ensure!(
            chunk.end.is_none(),
            "owned Core terminal ended during native input check"
        );
        anyhow::ensure!(
            seen.len() + chunk.bytes.len() <= CAPTURE_LIMIT,
            "Core fixture output exceeds 1 MiB"
        );
        seen.extend(chunk.bytes);
    }
    Ok(true)
}

fn require_runtime_output(observed: &Attachment, marker: &[u8]) {
    assert!(
        runtime_output_seen(observed, marker, Duration::from_secs(10)).unwrap(),
        "native input did not reach this exact Core terminal"
    );
}

fn runtime_diagnostic(observed: &Attachment, limit: Duration) -> anyhow::Result<(bool, String)> {
    let deadline = Instant::now() + limit;
    let mut bytes = Vec::new();
    while Instant::now() < deadline {
        let chunk = observed.read(Duration::from_millis(100))?;
        anyhow::ensure!(
            chunk.end.is_none(),
            "owned Core terminal ended during native input check"
        );
        anyhow::ensure!(
            bytes.len() + chunk.bytes.len() <= CAPTURE_LIMIT,
            "Core fixture diagnostic exceeds 1 MiB"
        );
        bytes.extend(chunk.bytes);
    }
    let marker = b"SP_A11Y_7";
    let present = bytes.windows(marker.len()).any(|part| part == marker);
    let shown = &bytes[..bytes.len().min(8192)];
    let diagnostic = format!(
        "marker={present} bytes={} shown={} text={:?} hex={shown:02x?}",
        bytes.len(),
        shown.len(),
        String::from_utf8_lossy(shown)
    );
    Ok((present, diagnostic))
}

#[test]
#[ignore = "requires Windows desktop plus explicitly selected Herdr and Core fixture binaries"]
fn native_core_text_caret_bounds_and_output_changes_match_the_real_runtime() {
    let mut fixture = Fixture::start();
    let terminal = fixture.open_terminal();
    fixture
        .running
        .wait_tree("native Core terminal text provider", |tree| {
            tree.contains("Document 'Terminal' [Value, Text]")
        });
    let info = fixture.api().terminal_get(&terminal).unwrap();
    let observed = fixture
        .api()
        .attach(&Attach {
            cols: info.cols,
            rows: info.rows,
            terminal: &terminal,
            observe: true,
            ..Attach::default()
        })
        .unwrap();
    let capture = fixture
        .running
        .sandbox
        .dir
        .join("core-terminal-capture.log");
    let command = fixture.terminal_helper("Text", "core-terminal-capture.log");
    let output = fixture
        .running
        .command_output(command, &capture, Duration::from_secs(30))
        .unwrap_or_else(|error| {
            let runtime = runtime_diagnostic(&observed, Duration::from_secs(1));
            let frames = fixture.frame_logs();
            panic!(
                "{error}; exact Core runtime on helper failure: cols={} rows={} {runtime:?}; app frames: {frames:?}",
                info.cols, info.rows
            );
        });
    for receipt in [
        "TERMINAL_A11Y text=true native_text_changed=true unselected_caret=true caret_line_bounds=true",
        "TERMINAL_A11Y observer_removed=true",
    ] {
        assert!(output.contains(receipt), "{output}");
    }
    let pixels = output
        .lines()
        .find(|line| line.starts_with("CORE_TERMINAL_PNG "))
        .expect("private Core helper did not capture native pixels");
    eprintln!("{pixels}");
    let (present, runtime) = runtime_diagnostic(&observed, Duration::from_secs(10)).unwrap();
    assert!(
        present,
        "native input did not reach this exact Core terminal"
    );
    eprintln!(
        "CORE_RUNTIME_CAPTURE cols={} rows={} {runtime}",
        info.cols, info.rows
    );
    let frames = fixture.frame_logs().unwrap();
    assert!(
        frames.contains("CORE_FRAME_PROBE"),
        "private app frame observer did not produce records: {frames}"
    );
    eprintln!("CORE_APP_FRAMES {frames}");
    assert_eq!(fixture.api().list(false).unwrap().terminals.len(), 1);
    eprintln!(
        "CORE_NATIVE text=true native_text_changed=true caret_line_bounds=true exact_runtime_output=true observer_removed=true"
    );
}

#[test]
#[ignore = "requires Windows desktop plus explicitly selected Herdr and Core fixture binaries"]
fn native_core_observing_pane_exposes_fresh_text_caret_and_change_events() {
    let mut fixture = Fixture::start();
    let terminal = fixture.open_terminal();
    let before = fixture.tab_ids(&terminal);
    assert_eq!(before.len(), 1);
    fixture
        .running
        .control(&["run", "--json", "mirror_core_pane"]);
    let deadline = Instant::now() + Duration::from_secs(15);
    let mirror = loop {
        let tabs = fixture.tab_ids(&terminal);
        if tabs.len() == 2 && tabs.contains(&before[0]) {
            break *tabs.iter().find(|tab| **tab != before[0]).unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "Core mirror did not preserve the exact source tab"
        );
        thread::sleep(Duration::from_millis(20));
    };
    fixture
        .running
        .control(&["tab", "--json", "focus", &mirror.to_string()]);
    let info = fixture.api().terminal_get(&terminal).unwrap();
    let writer = fixture
        .api()
        .attach(&Attach {
            cols: info.cols,
            rows: info.rows,
            terminal: &terminal,
            ..Attach::default()
        })
        .unwrap();
    let ready = fixture.running.sandbox.dir.join("core-observe.ready");
    assert!(!ready.exists());
    let capture = fixture.running.sandbox.dir.join("core-observe.log");
    let command = fixture.terminal_helper("Observe", "core-observe.log");
    let (output, produced) = thread::scope(|scope| {
        let producer = scope.spawn(|| -> anyhow::Result<()> {
            let deadline = Instant::now() + Duration::from_secs(15);
            while !ready.exists() {
                anyhow::ensure!(
                    Instant::now() < deadline,
                    "native Core observer did not register"
                );
                thread::sleep(Duration::from_millis(10));
            }
            writer.write(b"echo SP_A11Y_OBS_7\r")?;
            Ok(())
        });
        let output = fixture
            .running
            .command_output(command, &capture, Duration::from_secs(30));
        (output, producer.join())
    });
    let output =
        output.unwrap_or_else(|error| panic!("{error}; Core producer result: {produced:?}"));
    produced.unwrap().unwrap();
    assert!(output.contains("TERMINAL_A11Y text=true native_text_changed=true unselected_caret=true caret_line_bounds=true"), "{output}");
    assert!(
        output.contains("TERMINAL_A11Y observer_removed=true"),
        "{output}"
    );
    let terminals = fixture.api().list(false).unwrap().terminals;
    assert_eq!(
        terminals
            .iter()
            .map(|terminal| terminal.address.as_str())
            .collect::<Vec<_>>(),
        vec![terminal.as_str()]
    );
    let after = fixture.tab_ids(&terminal);
    assert_eq!(after.len(), 2);
    assert!(
        after.contains(&before[0]) && after.contains(&mirror),
        "Core mirror changed app tab identities"
    );
    eprintln!(
        "CORE_NATIVE observing=true text=true native_text_changed=true caret_line_bounds=true same_terminal=true observer_removed=true"
    );
}

// ABOUTME: Profiles a real private Core terminal beside controlled Working TUI content.
// ABOUTME: Owns holder-aware runtime cleanup and keeps projected content pending capture review.
use super::*;
use readiness::CoreTarget;
use sheprd_core_client::{
    Core,
    wire::{Attach, Terminal, TerminalState},
};
use std::io;

#[path = "../../../../test-support/core_windows.rs"]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "shared native Core fixture validates explicit isolated runtime preconditions"
)]
mod runtime;

const OUTPUT_LIMIT: usize = 1024 * 1024;

struct MixedFixture {
    tui: Fixture,
    api: Option<Core>,
    runtime: Option<runtime::Runtime>,
    directory: PathBuf,
}

impl MixedFixture {
    fn api(&self) -> Result<&Core> {
        self.api.as_ref().context("private Core API is absent")
    }

    fn cleanup(&mut self) -> Result<()> {
        let mut gui = self.tui.gui.take();
        stop_children([&mut gui]);
        self.api.take();
        let Some(runtime) = self.runtime.take() else {
            return Ok(());
        };
        let receipt = self.directory.join("core-cleanup.json");
        let prepared = fs::write(
            &receipt,
            json!({
                "stop":"prepared private command; submission not yet acknowledged",
                "holder_cleanup_qualified":false,
            })
            .to_string(),
        );
        // Stop streams were prepared before serve. Never reopen them before spawn.
        let stopped = runtime.shutdown(|mut command, capture, limit| {
            let child = command.spawn()?;
            wait_for_output(child, capture, limit, report::MAX_LINE)
                .map(|_| ())
                .map_err(io::Error::other)
        });
        let retained = fs::write(
            receipt,
            json!({
                "stop":if stopped.is_ok() {"acknowledged; owned serve exited successfully"} else {"failed; private state retained"},
                "holder_cleanup_qualified":stopped.is_ok(),
                "failure":stopped.as_ref().err().map(|error| format!("{error:#}")),
            }).to_string(),
        );
        if let Err(error) = &prepared {
            eprintln!("WORKING PROFILE Core cleanup preparation receipt failed: {error}");
        }
        if let Err(error) = &retained {
            eprintln!("WORKING PROFILE Core cleanup outcome receipt failed: {error}");
        }
        stopped.context("private Core stop or owned serve exit failed; retain all state")?;
        prepared.context("retain Core cleanup preparation receipt")?;
        retained.context("retain Core cleanup outcome receipt")?;
        Ok(())
    }

    fn created(&mut self, window: &str) -> Result<(u64, Terminal)> {
        let deadline = Instant::now() + STARTUP;
        loop {
            ensure!(
                !self.api()?.is_lost(),
                "private Core connection dropped after creation submission"
            );
            let terminals = self.api()?.list(false)?.terminals;
            ensure!(
                terminals.len() <= 1,
                "private Core created unexpected additional terminals"
            );
            if let [terminal] = terminals.as_slice() {
                ensure!(
                    terminal.state == TerminalState::Live,
                    "created Core terminal ended"
                );
                let tabs = self
                    .tui
                    .cli("core-tabs", &["tab", "list", "--window", window])?;
                let tabs = tabs["tabs"].as_array().context("Core app tab list")?;
                let matching: Vec<_> = tabs
                    .iter()
                    .filter(|tab| {
                        tab["kind"] == "core_terminal" && tab["location"] == terminal.address
                    })
                    .collect();
                ensure!(
                    matching.len() <= 1,
                    "private Core terminal appeared in multiple app tabs"
                );
                if let [tab] = matching.as_slice() {
                    let id = tab["id"].as_u64().context("actual Core app tab ID")?;
                    let actual = self.api()?.terminal_get(&terminal.address)?;
                    ensure!(
                        actual.address == terminal.address && actual.state == TerminalState::Live,
                        "Core terminal identity or liveness changed after app tab creation"
                    );
                    return Ok((id, actual));
                }
            }
            ensure!(
                Instant::now() < deadline,
                "one submitted Core creation did not yield its exact app tab"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn seed(&self, terminal: &Terminal, token: &str) -> Result<Value> {
        ensure!(
            terminal.state == TerminalState::Live,
            "Core terminal ended before output generation"
        );
        fs::write(
            self.directory.join("core-terminal.json"),
            serde_json::to_vec_pretty(&json!({
                "terminal":terminal.address,"argv":terminal.argv,"pid":terminal.pid,
                "columns":terminal.cols,"rows":terminal.rows,"state":"live",
            }))?,
        )?;
        let program = terminal
            .argv
            .first()
            .context("Core did not report its shell argv")?;
        let program = Path::new(program)
            .file_name()
            .and_then(|name| name.to_str())
            .context("Core shell argv has no valid program name")?;
        ensure!(
            program.eq_ignore_ascii_case("powershell.exe")
                || program.eq_ignore_ascii_case("pwsh.exe"),
            "Core profile supports an observed PowerShell shell only; actual argv={:?}",
            terminal.argv
        );
        ensure!(
            terminal.cols > 0 && terminal.rows > 0,
            "Core terminal has no current grid"
        );
        let marker = format!("SP_CORE_{token}");
        let command = format!(
            "{}; 'SP_CORE_{{0}}' -f '{token}'",
            peer::generated_command()
        );
        ensure!(
            !command.contains(report::MARKER) && !command.contains(&marker),
            "Core command echo must not contain either complete output marker"
        );
        let attached = self.api()?.attach(&Attach {
            cols: terminal.cols,
            rows: terminal.rows,
            terminal: &terminal.address,
            ..Attach::default()
        })?;
        ensure!(
            attached.terminal.address == terminal.address,
            "Core writer attached to another terminal"
        );
        let mut bytes = Vec::new();
        let produced = (|| -> Result<()> {
            attached.write(format!("{command}\r").as_bytes())?;
            let deadline = Instant::now() + STARTUP;
            let last_row = format!("synthetic profile row {:03}", peer::GENERATED_LINES - 1);
            loop {
                let chunk = attached.read(Duration::from_millis(100))?;
                ensure!(
                    chunk.end.is_none(),
                    "Core terminal ended while generating profile content"
                );
                ensure!(
                    bytes.len() + chunk.bytes.len() <= OUTPUT_LIMIT,
                    "Core profile output exceeds 1 MiB"
                );
                bytes.extend(chunk.bytes);
                let contains = |marker: &str| {
                    bytes
                        .windows(marker.len())
                        .any(|part| part == marker.as_bytes())
                };
                if contains(report::MARKER) && contains(&marker) && contains(&last_row) {
                    return Ok(());
                }
                ensure!(
                    Instant::now() < deadline,
                    "Core did not produce its authored marker and final row"
                );
            }
        })();
        let retained = fs::write(self.directory.join("core-output.bin"), &bytes);
        if let Err(error) = &retained {
            eprintln!("WORKING PROFILE Core output retention failed: {error}");
        }
        produced?;
        retained?;
        // The attachment drops here, before any observation wait or measured phase.
        Ok(json!({
            "authored_lines":peer::GENERATED_LINES,"unique_marker":marker,
            "observed_raw_bytes":bytes.len(),"output_limit_bytes":OUTPUT_LIMIT,
            "input_echo_contains_complete_marker":false,
            "output_scope":"settled bounded shell output; no streaming workload during measured intervals",
        }))
    }
}

impl Drop for MixedFixture {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            eprintln!("WORKING PROFILE private Core cleanup failed; state retained: {error:#}");
        }
        // Base Fixture drops next and preserves the entire private parent.
    }
}

pub(crate) fn run() -> Result<()> {
    check_environment()?;
    let binary = runtime::binary();
    let target = runtime::Target::new();
    let tui = Fixture::prepare()?;
    let directory = tui.sandbox()?.dir.clone();
    let mut fixture = MixedFixture {
        tui,
        api: None,
        runtime: None,
        directory: directory.clone(),
    };
    let core_root = directory.join("core");
    fs::create_dir(&core_root)?;
    // Register exact child ownership before connecting or retaining any further result.
    fixture.runtime = Some(runtime::Runtime::start(
        fixture.tui.sandbox()?,
        &binary,
        &core_root,
        target,
    ));
    fixture.api = Some(
        fixture
            .runtime
            .as_mut()
            .context("private Core runtime")?
            .connect(),
    );
    let core_id = fixture.api()?.hello().core.id.clone();
    ensure!(
        !core_id.is_empty(),
        "private Core hello omitted its identity"
    );
    fs::write(
        directory.join("core-runtime.json"),
        serde_json::to_vec_pretty(&json!({
            "binary":binary,"nonce":target.nonce().simple().to_string(),"core_id":core_id,
            "generation":fixture.api()?.hello().generation,"version":fixture.api()?.hello().version,
            "state_root":core_root,"frame_probe_enabled":false,
        }))?,
    )?;
    fs::write(
        directory.join("config/spaceport/config.local.toml"),
        "language = \"en-US\"\nherdr_core_enabled = true\n",
    )?;
    let targets = fixture.tui.start_tui()?;
    fixture.tui.launch_gui(|command| target.apply(command))?;
    let window = fixture.tui.split_window(&targets)?;
    fixture.tui.cli(
        "core-create",
        &["run", "new_core_terminal", "--window", &window],
    )?;
    let (tab, terminal) = fixture.created(&window)?;
    fixture.tui.cli(
        "core-focus",
        &["tab", "focus", &tab.to_string(), "--window", &window],
    )?;
    fixture.tui.wait("selected live Core tab", |observation| {
        observation.groups.iter().any(|group| {
            !group.shown_terminal
                && group.core.as_ref().is_some_and(|core| {
                    core.core_id.as_deref() == Some(&core_id)
                        && core.tab == tab
                        && core.terminal == terminal.address
                        && core.connected
                        && !core.observe
                        && !core.ended
                        && !core.failed
                })
        })
    })?;
    let terminal = fixture.api()?.terminal_get(&terminal.address)?;
    let output = fixture.seed(&terminal, &target.nonce().simple().to_string())?;
    let workload = Workload::Mixed {
        tui: &targets,
        core: CoreTarget {
            core_id: &core_id,
            tab,
            terminal: &terminal.address,
        },
    };
    let mut phases = Vec::new();
    phases.push(fixture.tui.phase("idle-before", workload, false)?);
    peer::report_status(&directory.join("a.sock"), &targets.pane, "working", 1)?;
    phases.push(fixture.tui.phase("synthetic-working", workload, true)?);
    peer::report_status(&directory.join("a.sock"), &targets.pane, "idle", 2)?;
    phases.push(fixture.tui.phase("idle-after", workload, false)?);
    fixture.tui.extract_stacks(&mut phases)?;
    ensure!(
        matches!(exchange(&directory, &Request::Stop)?, Reply::Stopped),
        "observer failed to stop"
    );
    Peer::open(&fixture.tui.sandbox()?.socket())?.close_workspace(&targets.workspace)?;
    fixture.cleanup()?;
    fs::write(
        directory.join("report.json"),
        serde_json::to_vec_pretty(&json!({
            "workload":"one generated TUI group with synthetic Working status beside one private single-pane Core terminal",
            "synthetic_status":true,"provider_used":false,"workspace":targets.workspace,"boot":targets.boot,
            "generated_pane":targets.pane,"tui_generated_lines":peer::GENERATED_LINES,
            "core_id":core_id,"core_tab":tab,"core_terminal":terminal.address,
            "core_output":output,"listening_port_poller_enabled":false,
            "core_content_scope":"projected frame metadata plus last-layout bounds; no paint receipt or native presentation metric",
            "visible_content_qualified":false,"capture_review_required":true,
            "capture_scope":"native GPU scene at endpoints; inspect unique Core marker; projected metadata may lead last rendered scene",
            "cpu_scope":"all-thread process CPU; sampled trace includes recorder overhead; no provider ran",
            "limits":"settled content only; no chat, mirror, or streaming qualification; no uninterrupted foreground proof",
            "phases":phases,
        }))?,
    )?;
    Ok(())
}

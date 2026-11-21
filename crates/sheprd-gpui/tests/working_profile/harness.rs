// ABOUTME: Profiles an owned release GUI against explicit private terminal workloads.
// ABOUTME: Retains workload observations, native captures, CPU clocks, and sampled UI stacks.
use super::{
    measurements,
    peer::{self, Peer, Targets},
    report::{self, Observation, Reply, Request},
};
use anyhow::{Context as _, Result, bail, ensure};
#[path = "../../../test-support/sandbox.rs"]
#[allow(clippy::disallowed_methods)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "shared sandbox validates isolated process preconditions"
)]
mod sandbox;
use sandbox::{Sandbox, daemon_binary, stop_children};
use serde_json::{Value, json};
use sheprd_client::Stream;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command},
    time::{Duration, Instant},
};

const STARTUP: Duration = Duration::from_secs(30);
const SAMPLE_SECONDS: u64 = 5;

mod readiness;
use readiness::Workload;

#[cfg(windows)]
mod core;
#[cfg(windows)]
pub(crate) use core::run as run_core;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

struct Sample {
    before: Duration,
    after: Duration,
    wall: Duration,
    path: PathBuf,
    details: Value,
    interval_ms: Option<u64>,
}

#[cfg(test)]
#[path = "before_sample.rs"]
mod before_sample;
#[cfg(test)]
mod configuration;
#[cfg(test)]
#[path = "activation.rs"]
mod tests;

struct Fixture {
    sandbox: Option<Sandbox>,
    gui: Option<Child>,
    server: Option<Child>,
    request: u64,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        stop_children([&mut self.gui, &mut self.server]);
        if let Some(sandbox) = self.sandbox.take() {
            eprintln!(
                "WORKING PROFILE retained private artifacts: {}",
                sandbox.dir.display()
            );
            // Raw profiles and partial failure evidence must survive the shared Sandbox cleanup.
            std::mem::forget(sandbox);
        }
    }
}

impl Fixture {
    fn prepare(core_enabled: bool) -> Result<Self> {
        let fixture = Self {
            sandbox: Some(Sandbox::new()),
            gui: None,
            server: None,
            request: 0,
        };
        let dir = &fixture.sandbox()?.dir;
        eprintln!(
            "WORKING PROFILE synthetic workload; private artifacts={}",
            dir.display()
        );
        fs::create_dir_all(dir.join("config/spaceport"))?;
        fs::write(
            dir.join("config/spaceport/config.local.toml"),
            format!(
                "language = \"en-US\"\nherdr_core_enabled = {core_enabled}\nshow_listening_ports = false\n[agent_skill]\ninstall=\"off\"\n"
            ),
        )?;
        fs::write(
            dir.join("config/spaceport/view.toml"),
            "[ui]\nprompt_new_tab_name=false\nprompt_new_workspace_name=false\n",
        )?;
        let path = dir.join("config/spaceport/config.toml");
        let (_lock, _) = sheprd_settings::Config::prepare_files(&path)?;
        let config =
            sheprd_settings::Config::load_fixture(&path, &dir.join("config/spaceport/view.toml"))?;
        ensure!(
            !config.show_listening_ports
                && config.agent_skill.install == sheprd_settings::SkillInstall::Off
                && config.herdr_core_enabled == core_enabled,
            "private profile settings did not preserve disabled ports/skill and Core={core_enabled}"
        );
        Ok(fixture)
    }

    fn sandbox(&self) -> Result<&Sandbox> {
        self.sandbox.as_ref().context("missing profile sandbox")
    }

    fn alive(&mut self) -> Result<u32> {
        let child = self.gui.as_mut().context("missing profile GUI")?;
        ensure!(
            child.try_wait()?.is_none(),
            "profile GUI {} exited",
            child.id()
        );
        Ok(child.id())
    }

    fn observe(&mut self, capture: bool) -> Result<Observation> {
        self.alive()?;
        self.request += 1;
        let request = self.request;
        match exchange(
            &self.sandbox()?.dir,
            &Request::Observe {
                id: request,
                capture,
            },
        )? {
            Reply::Observation(observation) => {
                let observation = *observation;
                ensure!(
                    observation.request_id == request,
                    "uncorrelated profile observation"
                );
                ensure!(
                    observation.gui_pid == self.alive()?,
                    "observer reports another GUI PID"
                );
                ensure!(observation.release, "observer must run in release mode");
                #[cfg(windows)]
                let observation = self.native_observation(observation)?;
                if capture {
                    let (width, height) = image::image_dimensions(
                        self.sandbox()?.dir.join(format!("capture-{request}.png")),
                    )?;
                    ensure!(width > 0 && height > 0, "empty native capture");
                }
                Ok(observation)
            }
            Reply::Stopped => bail!("profile observer stopped unexpectedly"),
        }
    }

    fn wait(&mut self, stage: &str, accept: impl Fn(&Observation) -> bool) -> Result<Observation> {
        let deadline = Instant::now() + STARTUP;
        loop {
            let observation = self
                .observe(false)
                .with_context(|| format!("profile {stage}"))?;
            if accept(&observation) {
                return Ok(observation);
            }
            ensure!(
                Instant::now() < deadline,
                "profile {stage} timed out: {observation:?}"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn cli(&self, name: &str, args: &[&str]) -> Result<Value> {
        let mut command = self.sandbox()?.command(
            env!("CARGO_BIN_EXE_spaceport"),
            format!("{name}.stderr.log"),
        );
        #[cfg(windows)]
        windows::hide(&mut command);
        command.args(args).arg("--json");
        let bytes = command_output(
            command,
            &self.sandbox()?.dir.join(format!("{name}.json")),
            Duration::from_secs(15),
        )?;
        serde_json::from_slice(&bytes).with_context(|| format!("profile CLI {name} output"))
    }

    fn start_tui(&mut self) -> Result<Targets> {
        self.server = Some(
            self.sandbox()?
                .command(daemon_binary(), "daemon.log")
                .arg("server")
                .spawn()?,
        );
        let sandbox = self.sandbox.as_ref().context("profile sandbox")?;
        sandbox.wait_for_daemon(self.server.as_mut().context("profile server")?, STARTUP);
        let mut peer = Peer::open(&sandbox.socket())?;
        peer.seed(&sandbox.dir)
    }

    fn launch_gui(&mut self, configure: impl FnOnce(&mut Command)) -> Result<()> {
        let dir = self.sandbox()?.dir.clone();
        let mut command = self
            .sandbox()?
            .command(env!("CARGO_BIN_EXE_spaceport"), "gui.log");
        command
            .arg("--socket")
            .arg(self.sandbox()?.socket())
            .env(report::OBSERVER_ENV, &dir);
        configure(&mut command);
        self.gui = Some(command.spawn()?);
        let ready = dir.join(report::READY);
        let deadline = Instant::now() + STARTUP;
        while !ready.exists() {
            self.alive()?;
            ensure!(
                Instant::now() < deadline,
                "profile observer startup timed out"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }

    fn split_window(&mut self, targets: &Targets) -> Result<String> {
        let windows = self.cli("windows", &["window", "list"])?;
        let windows = windows["windows"]
            .as_array()
            .context("window list must contain windows")?;
        ensure!(
            windows.len() == 1,
            "private profile requires exactly one window: {windows:?}"
        );
        let window = windows[0]["id"]
            .as_u64()
            .context("actual window ID")?
            .to_string();
        self.cli("size", &["window", "size", "1200x780", "--window", &window])?;
        self.cli("focus", &["window", "focus", &window])?;
        self.wait("initial presented terminal", |observation| {
            observation.workspace.as_deref() == Some(&targets.workspace)
                && observation.groups.iter().any(|group| {
                    group
                        .presented
                        .as_ref()
                        .is_some_and(|frame| frame.marker_visible)
                })
        })?;
        self.cli("split", &["run", "split_editor", "--window", &window])?;
        self.wait("split groups", |observation| observation.groups.len() == 2)?;
        Ok(window)
    }

    fn phase(&mut self, name: &str, workload: Workload<'_>, working: bool) -> Result<Value> {
        let targets = match workload {
            Workload::TwoTui(targets) | Workload::Mixed { tui: targets, .. } => targets,
        };
        self.wait(name, |observation| {
            workload.populated(observation)
                && observation.synthetic_working_panes.contains(&targets.pane) == working
        })?;
        let activation_deadline = Instant::now() + Duration::from_secs(3);
        while !foreground(&self.observe(false)?) && Instant::now() < activation_deadline {
            std::thread::sleep(Duration::from_millis(100));
        }
        let before_image = self.observe(true)?;
        workload
            .validate(&before_image, working)
            .with_context(|| format!("profile {name} before sample"))?;
        let pid = self.alive()?;
        let Sample {
            before,
            after,
            wall,
            path,
            details,
            interval_ms,
        } = self.sample(name, pid, &before_image)?;
        self.alive()?;
        let after_image = self.observe(true)?;
        let foreground_qualified = foreground(&after_image);
        let percentage = measurements::process_cpu(before, after, wall)?;
        #[cfg(windows)]
        let activation_scope = "native checks before recorder setup and after owned trace stop; no monitoring during sampling";
        #[cfg(target_os = "macos")]
        let activation_scope =
            "native state at interval endpoints; no activation monitoring during sampling";
        let phase = json!({"phase":name,"synthetic_working":working,"pid":pid,"sample_seconds":SAMPLE_SECONDS,
            "sampling":details,"sample_interval_ms":interval_ms,"wall_seconds":wall.as_secs_f64(),"cpu_before_seconds":before.as_secs_f64(),
            "cpu_after_seconds":after.as_secs_f64(),"process_cpu_percent":percentage,"sample_path":path,
            "window_active_before":before_image.window_active,"window_active_after":after_image.window_active,
            "listening_ports_configured":false,
            "foreground_qualified":foreground_qualified,"activation_scope":activation_scope,
            "before":before_image,"after":after_image});
        fs::write(
            self.sandbox()?.dir.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&phase)?,
        )?;
        workload.validate(&after_image, working).with_context(|| {
            format!("profile {name} after sample; partial phase metrics retained")
        })?;
        eprintln!(
            "WORKING PROFILE phase={name} pid={pid} process_cpu_percent={percentage:.2} wall_seconds={:.3} sample={}",
            wall.as_secs_f64(),
            path.display()
        );
        Ok(phase)
    }
}

fn foreground(observation: &Observation) -> bool {
    if !observation.window_active || !observation.window_visible {
        return false;
    }
    if let Some(hwnd) = observation.windows_hwnd {
        return observation.windows_native.as_ref().is_some_and(|native| {
            native.hwnd == hwnd
                && native.gui_pid == observation.gui_pid
                && native.thread_id != 0
                && native.foreground_hwnd == hwnd
                && native.visible
                && !native.minimized
        });
    }
    observation.native_key == Some(true)
        && observation.app_active == Some(true)
        && observation.app_hidden == Some(false)
}

fn validate_before_sample(
    observation: &Observation,
    targets: &Targets,
    working: bool,
) -> Result<()> {
    Workload::TwoTui(targets).validate(observation, working)
}

fn populated(observation: &Observation, targets: &Targets) -> bool {
    observation.workspace.as_deref() == Some(&targets.workspace)
        && observation.groups.len() == 2
        && observation.groups.iter().all(|group| {
            group.shown_terminal
                && group.connected
                && group.bounds[2] > 0.
                && group.bounds[3] > 0.
                && group.tab.is_some()
                && group.presented.as_ref().is_some_and(|frame| {
                    frame.current
                        && frame.boot == targets.boot
                        && frame.columns > 0
                        && frame.rows > 0
                        && !frame.pane_ids.is_empty()
                        && frame.nonblank_cells > 0
                })
        })
        && observation.groups[0].tab != observation.groups[1].tab
        && observation.groups.iter().any(|group| {
            group
                .presented
                .as_ref()
                .is_some_and(|frame| frame.marker_visible)
        })
}

fn check_environment() -> Result<()> {
    ensure!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "run working_terminal_groups_profile with --release"
    );
    #[cfg(unix)]
    ensure!(
        std::env::var_os("HERDR_TEST_TMPDIR").as_deref() == Some(std::ffi::OsStr::new("/tmp")),
        "set HERDR_TEST_TMPDIR=/tmp explicitly"
    );
    #[cfg(windows)]
    ensure!(
        std::env::var_os("HERDR_TEST_TMPDIR").is_some_and(|path| Path::new(&path).is_dir()),
        "set HERDR_TEST_TMPDIR to an existing private temporary parent explicitly"
    );
    Ok(())
}

pub(crate) fn run() -> Result<()> {
    check_environment()?;
    let mut fixture = Fixture::prepare(false)?;
    let dir = fixture.sandbox()?.dir.clone();
    let targets = fixture.start_tui()?;
    fixture.launch_gui(|_| {})?;
    let window = fixture.split_window(&targets)?;
    fixture.cli("new-tab", &["run", "new_tab", "--window", &window])?;
    let mut phases = Vec::new();
    phases.push(fixture.phase("idle-before", Workload::TwoTui(&targets), false)?);
    peer::report_status(&dir.join("a.sock"), &targets.pane, "working", 1)?;
    phases.push(fixture.phase("synthetic-working", Workload::TwoTui(&targets), true)?);
    peer::report_status(&dir.join("a.sock"), &targets.pane, "idle", 2)?;
    phases.push(fixture.phase("idle-after", Workload::TwoTui(&targets), false)?);
    #[cfg(windows)]
    fixture.extract_stacks(&mut phases)?;
    fs::write(
        dir.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"workload":"synthetic status with generated terminal output and idle second terminal",
        "generated_lines":peer::GENERATED_LINES,"generated_bytes":(0..peer::GENERATED_LINES).map(|line| format!("synthetic profile row {line:03}\n").len()).sum::<usize>() + report::MARKER.len() + 1,
        "generated_bytes_scope":"authored UTF-8 text payload with LF separators; Windows console transport/control bytes are not measured",
        "marker":report::MARKER,"workspace":targets.workspace,"boot":targets.boot,
        "generated_pane":targets.pane,"listening_ports_configured":false,
        "configuration_scope":"private settings parsed before launch; no live poller-state probe",
        "cpu_scope":"all-thread process CPU; sample itself perturbs execution", "phases":phases}),
        )?,
    )?;
    ensure!(
        matches!(exchange(&dir, &Request::Stop)?, Reply::Stopped),
        "observer failed to stop"
    );
    // Use the same verified workspace.close contract; never stop an unrelated server.
    Peer::open(&fixture.sandbox()?.socket())?.close_workspace(&targets.workspace)?;
    Ok(())
}

fn exchange(directory: &Path, request: &Request) -> Result<Reply> {
    #[cfg(unix)]
    let mut stream = Stream::connect(report::endpoint(directory)?)?;
    #[cfg(windows)]
    let mut stream = Stream::connect_owned(report::endpoint(directory)?)?;
    stream.set_read_timeout(Some(Duration::from_secs(15)))?;
    stream.set_write_timeout(Some(Duration::from_secs(15)))?;
    writeln!(stream, "{}", serde_json::to_string(request)?)?;
    super::wire::read_reply(stream)
}

fn command_output(command: Command, output: &Path, limit: Duration) -> Result<Vec<u8>> {
    command_output_bounded(command, output, limit, report::MAX_LINE)
}

fn command_output_bounded(
    mut command: Command,
    output: &Path,
    limit: Duration,
    bound: u64,
) -> Result<Vec<u8>> {
    command.stdout(File::create(output)?);
    let child = command
        .spawn()
        .with_context(|| format!("spawn profile command {command:?}"))?;
    wait_for_output(child, output, limit, bound)
}

fn wait_for_output(child: Child, output: &Path, limit: Duration, bound: u64) -> Result<Vec<u8>> {
    let mut child = Some(child);
    let result = (|| {
        let deadline = Instant::now() + limit;
        loop {
            let process = child.as_mut().context("missing profile command")?;
            if let Some(status) = process.try_wait()? {
                ensure!(
                    status.success(),
                    "profile command {} failed: {status}; output={}",
                    process.id(),
                    output.display()
                );
                ensure!(
                    fs::metadata(output)?.len() <= bound,
                    "profile command output exceeds limit"
                );
                return fs::read(output).context("read profile command output");
            }
            ensure!(
                Instant::now() < deadline,
                "profile command {} timed out; output={}",
                process.id(),
                output.display()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    })();
    stop_children([&mut child]);
    result
}

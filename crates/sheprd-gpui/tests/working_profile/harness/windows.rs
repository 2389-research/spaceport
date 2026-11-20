// ABOUTME: Measures the exact Windows GUI process and observes its owned native window.
// ABOUTME: Keeps helper commands hidden and all blocking native inspection outside UI updates.
use super::*;
use std::os::windows::process::CommandExt as _;

mod recording;

pub(super) fn hide(command: &mut Command) {
    command.creation_flags(sheprd_client::CREATE_NO_WINDOW);
}

impl Fixture {
    fn recorder_available(&self, name: &str) -> Result<recording::Tools> {
        let directory = &self.sandbox()?.dir;
        let tools = recording::Tools::installed()?;
        let mut status = self
            .sandbox()?
            .command(&tools.wpr, format!("{name}-wpr.stderr.log"));
        hide(&mut status);
        status.arg("-status");
        let status = command_output(
            status,
            &directory.join(format!("{name}-wpr.txt")),
            Duration::from_secs(15),
        )?;
        let mut loggers = self
            .sandbox()?
            .command(&tools.xperf, format!("{name}-loggers.stderr.log"));
        hide(&mut loggers);
        loggers.arg("-loggers");
        let loggers = command_output_bounded(
            loggers,
            &directory.join(format!("{name}-loggers.txt")),
            Duration::from_secs(15),
            2 * 1024 * 1024,
        )?;
        recording::ensure_available(
            std::str::from_utf8(&status)?,
            std::str::from_utf8(&loggers)?,
        )
        .with_context(|| {
            format!(
                "recorder preflight refused; private queries retained at {}",
                directory.display()
            )
        })?;
        eprintln!(
            "WORKING PROFILE read-only recorder preflight passed; status_bytes={} logger_bytes={}; no trace started",
            status.len(),
            loggers.len()
        );
        Ok(tools)
    }

    pub(super) fn native_observation(&self, mut observation: Observation) -> Result<Observation> {
        let hwnd = observation
            .windows_hwnd
            .context("Windows observer omitted actual HWND")?;
        let mut command = self.sandbox()?.command(
            "powershell.exe",
            format!("native-{}.stderr.log", observation.request_id),
        );
        // These system paths support Windows PowerShell's cmdlet loading.
        for name in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        hide(&mut command);
        command
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"])
            .arg(format!(
                "& {{\n{}\n}} -WindowHandle {hwnd} -ProcessId {} -ReceiptFile native-{}.stages.txt",
                include_str!("windows_native.ps1"),
                observation.gui_pid,
                observation.request_id,
            ));
        let directory = &self.sandbox()?.dir;
        let output = directory.join(format!("native-{}.json", observation.request_id));
        let bytes =
            command_output(command, &output, Duration::from_secs(15)).with_context(|| {
                format!(
                    "native helper request={} gui_pid={}; stdout={}; stderr={}; stages={}",
                    observation.request_id,
                    observation.gui_pid,
                    bounded_diagnostic(&output),
                    bounded_diagnostic(
                        &directory.join(format!("native-{}.stderr.log", observation.request_id))
                    ),
                    bounded_diagnostic(
                        &directory.join(format!("native-{}.stages.txt", observation.request_id))
                    ),
                )
            })?;
        observation.windows_native = Some(serde_json::from_slice(&bytes)?);
        Ok(observation)
    }

    fn cpu(&self, name: &str, pid: u32) -> Result<Duration> {
        let mut command = self
            .sandbox()?
            .command("powershell.exe", format!("{name}.stderr.log"));
        hide(&mut command);
        command.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"])
            .arg(format!("$ErrorActionPreference='Stop'; $taskProcess=[System.Diagnostics.Process]::GetProcessById({pid}); $taskProcess.TotalProcessorTime.Ticks.ToString([System.Globalization.CultureInfo]::InvariantCulture)"));
        let bytes = command_output(
            command,
            &self.sandbox()?.dir.join(format!("{name}.txt")),
            Duration::from_secs(10),
        )?;
        measurements::cpu_ticks(std::str::from_utf8(&bytes)?)
    }

    pub(super) fn sample(&self, name: &str, pid: u32, observation: &Observation) -> Result<Sample> {
        let prepared = Instant::now();
        let mut recording = recording::Recording::start(self, name)?;
        let setup = prepared.elapsed();
        let started = Instant::now();
        let before = self.cpu(&format!("{name}-cpu-before"), pid)?;
        std::thread::sleep(Duration::from_secs(SAMPLE_SECONDS));
        let after = self.cpu(&format!("{name}-cpu-after"), pid)?;
        let wall = started.elapsed();
        let stopped = Instant::now();
        let path = recording.stop()?;
        let shutdown = stopped.elapsed();
        let percentage = measurements::process_cpu(before, after, wall)?;
        // Preserve completed clocks if final capture or later export fails.
        fs::write(
            self.sandbox()?.dir.join(format!("{name}-measurement.json")),
            serde_json::to_vec_pretty(&json!({
                "phase":name,"pid":pid,"cpu_before_seconds":before.as_secs_f64(),"cpu_after_seconds":after.as_secs_f64(),
                "wall_seconds":wall.as_secs_f64(),"process_cpu_percent":percentage,"raw_trace":path,"raw_trace_private":true,
                "qualification":"pending final native observation and stack review","before":observation,
            }))?,
        )?;
        Ok(Sample {
            before,
            after,
            wall,
            path,
            interval_ms: None,
            details: json!({
                "sampler":"Windows Performance Recorder CPU","raw_trace_private":true,"stack_review_required":true,
                "setup_seconds":setup.as_secs_f64(),"stop_seconds":shutdown.as_secs_f64(),
            "trace_scope":"CPU-clock interval is inside the owned trace; trace endpoints also include recorder overhead",
                "symbols":"local executable PDB only; decoding remains unqualified until actual output inspection",
            }),
        })
    }

    pub(super) fn extract_stacks(&self, phases: &mut [Value]) -> Result<()> {
        let tools = recording::Tools::installed()?;
        let symbols = recording::local_symbols()?;
        let directory = &self.sandbox()?.dir;
        let cache = directory.join("symcache");
        fs::create_dir(&cache)?;
        for phase in phases {
            let name = phase["phase"].as_str().context("phase name")?.to_owned();
            let pid = phase["pid"].as_u64().context("phase GUI PID")?;
            let before = &phase["before"]["windows_native"];
            let after = &phase["after"]["windows_native"];
            let thread = before["thread_id"].as_u64().context("actual HWND thread")?;
            ensure!(
                thread > 0
                    && before["gui_pid"] == pid
                    && after["gui_pid"] == pid
                    && after["thread_id"] == thread
                    && before["hwnd"] == after["hwnd"],
                "native owner or HWND thread changed across phase {name}"
            );
            let raw = PathBuf::from(
                phase["sample_path"]
                    .as_str()
                    .context("raw owned trace path")?,
            );
            ensure!(
                raw == directory.join(format!("{name}.etl")),
                "trace is outside its owned phase"
            );
            let output = directory.join(format!("{name}-ui-stacks.txt"));
            ensure!(
                !output.exists(),
                "refusing to replace existing filtered stacks"
            );
            let mut command = self
                .sandbox()?
                .command(&tools.xperf, format!("{name}-xperf.stderr.log"));
            hide(&mut command);
            command
                .arg("-i")
                .arg(&raw)
                .arg("-o")
                .arg(&output)
                .args([
                    "-symbols",
                    "-a",
                    "stack",
                    "-butterfly",
                    "0",
                    "-pid",
                    &pid.to_string(),
                    "-tid",
                    &thread.to_string(),
                ])
                .env("_NT_SYMBOL_PATH", symbols)
                .env("_NT_SYMCACHE_PATH", &cache);
            command_output(
                command,
                &directory.join(format!("{name}-xperf.stdout.log")),
                Duration::from_secs(60),
            )?;
            let length = fs::metadata(&output)?.len();
            ensure!(
                length > 0 && length <= 32 * 1024 * 1024,
                "filtered stack output is empty or exceeds 32 MiB"
            );
            phase["sampling"]["filtered_stacks"] = json!({"path":output,"pid":pid,"thread_id":thread,
                "bytes":length,"review_required":true,"qualified":false,
                "limit":"command success and nonempty output do not prove samples or symbolization"});
            fs::write(
                directory.join(format!("{name}.json")),
                serde_json::to_vec_pretty(phase)?,
            )?;
        }
        Ok(())
    }
}

fn bounded_diagnostic(path: &Path) -> String {
    use std::io::Read as _;
    let mut bytes = Vec::new();
    match File::open(path).and_then(|file| file.take(4096).read_to_end(&mut bytes)) {
        Ok(_) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(error) => format!("unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests;

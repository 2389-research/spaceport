// ABOUTME: Measures the exact Windows GUI process and observes its owned native window.
// ABOUTME: Keeps helper commands hidden and all blocking native inspection outside UI updates.
use super::*;
use std::os::windows::process::CommandExt as _;

mod recording;

pub(super) fn hide(command: &mut Command) {
    command.creation_flags(sheprd_client::CREATE_NO_WINDOW);
}

#[cfg(test)]
mod tests {
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
        Ok(())
    }
}

impl Fixture {
    pub(super) fn native_observation(&self, mut observation: Observation) -> Result<Observation> {
        let hwnd = observation
            .windows_hwnd
            .context("Windows observer omitted actual HWND")?;
        let mut command = self.sandbox()?.command(
            "powershell.exe",
            format!("native-{}.stderr.log", observation.request_id),
        );
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

    pub(super) fn sample(
        &self,
        _name: &str,
        _pid: u32,
        _observation: &Observation,
    ) -> Result<Sample> {
        anyhow::bail!(
            "Windows sampling requires verified recorder ownership and actual tool help before qualification"
        )
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

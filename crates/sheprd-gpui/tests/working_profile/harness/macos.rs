// ABOUTME: Measures exact-process CPU and actual main-thread samples on macOS.
// ABOUTME: Preserves the release profile's native sample command and bounded artifact checks.
use super::*;

impl Fixture {
    fn cpu(&self, name: &str, pid: u32) -> Result<Duration> {
        let mut command = self
            .sandbox()?
            .command("/bin/ps", format!("{name}.stderr.log"));
        command.args(["-p", &pid.to_string(), "-o", "time="]);
        let bytes = command_output(
            command,
            &self.sandbox()?.dir.join(format!("{name}.txt")),
            Duration::from_secs(5),
        )?;
        measurements::cpu_clock(std::str::from_utf8(&bytes)?)
    }

    pub(super) fn sample(
        &self,
        name: &str,
        pid: u32,
        _observation: &Observation,
    ) -> Result<Sample> {
        let start = Instant::now();
        let before = self.cpu(&format!("{name}-cpu-before"), pid)?;
        let path = self.sandbox()?.dir.join(format!("{name}.sample.txt"));
        let mut command = self
            .sandbox()?
            .command("/usr/bin/sample", format!("{name}-sample.stderr.log"));
        command
            .args([&pid.to_string(), &SAMPLE_SECONDS.to_string(), "1", "-file"])
            .arg(&path);
        command_output(
            command,
            &self
                .sandbox()?
                .dir
                .join(format!("{name}-sample.stdout.log")),
            Duration::from_secs(20),
        )?;
        let after = self.cpu(&format!("{name}-cpu-after"), pid)?;
        let wall = start.elapsed();
        ensure!(
            fs::metadata(&path)?.len() <= 32 * 1024 * 1024,
            "native sample exceeds read bound"
        );
        let stacks = fs::read_to_string(&path).context("read actual native sample")?;
        ensure!(
            stacks.contains("com.apple.main-thread"),
            "sample lacks actual UI-thread stacks: {}",
            path.display()
        );
        Ok(Sample {
            before,
            after,
            wall,
            path,
            details: json!({"sampler":"macOS sample","ui_thread_stacks_verified":true}),
            interval_ms: Some(1),
        })
    }
}

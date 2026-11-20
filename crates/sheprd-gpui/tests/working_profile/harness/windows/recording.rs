// ABOUTME: Qualifies Windows recorder ownership before the fixture starts a CPU trace.
// ABOUTME: Refuses existing or unclassified system sampling without stopping any session.
use super::{Child, Command, File, Fixture, command_output, hide, wait_for_output};
use anyhow::{Context as _, Result, ensure};
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::{fs, path::Path, time::Duration};

pub(super) struct Tools {
    pub(super) wpr: PathBuf,
    pub(super) xperf: PathBuf,
}

impl Tools {
    pub(super) fn installed() -> Result<Self> {
        let wpr =
            PathBuf::from(std::env::var_os("SystemRoot").context("missing Windows SystemRoot")?)
                .join("System32/wpr.exe");
        let xperf = PathBuf::from(
            std::env::var_os("SPACEPORT_WORKING_PROFILE_XPERF")
                .context("set SPACEPORT_WORKING_PROFILE_XPERF explicitly")?,
        );
        ensure!(
            wpr.is_absolute() && wpr.is_file() && xperf.is_absolute() && xperf.is_file(),
            "recorder tools must be explicit installed absolute executables"
        );
        Ok(Self { wpr, xperf })
    }
}

pub(super) fn local_symbols() -> Result<&'static Path> {
    let executable = Path::new(env!("CARGO_BIN_EXE_spaceport"));
    ensure!(
        executable.with_extension("pdb").is_file(),
        "release spaceport.pdb is required for local symbol decoding"
    );
    executable.parent().context("profile executable directory")
}

const CPU_PROFILE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/profile/spaceport-cpu.wprp"
));

fn retain_profile(_directory: &Path, _name: &str) -> Result<std::ffi::OsString> {
    anyhow::bail!("sampled CPU profile retention is not implemented")
}

pub(super) struct Recording {
    authority: Option<String>,
    path: PathBuf,
    receipt: PathBuf,
    stop: Command,
    stdout: PathBuf,
}

impl Recording {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn start(fixture: &Fixture, name: &str) -> Result<Self> {
        local_symbols()?;
        let tools = fixture.recorder_available(name)?;
        let directory = &fixture.sandbox()?.dir;
        let path = directory.join(format!("{name}.etl"));
        ensure!(!path.exists(), "refusing to replace an existing trace");
        let temporary = directory.join(format!("{name}-wpr-temp"));
        fs::create_dir(&temporary)?;
        let instance = format!(
            "Spaceport-{}-{name}",
            directory
                .file_name()
                .context("profile directory name")?
                .to_string_lossy()
        );
        let receipt = directory.join(format!("{name}-recording.json"));
        // Prepare both Stop logs before Start can create an owned trace.
        let stdout = directory.join(format!("{name}-wpr-stop.txt"));
        let mut stop = fixture
            .sandbox()?
            .command(&tools.wpr, format!("{name}-wpr-stop.stderr.log"));
        hide(&mut stop);
        stop.stdout(File::create(&stdout)?)
            .arg("-stop")
            .arg(&path)
            .arg("Spaceport controlled Working status")
            .arg("-skipPdbGen")
            .arg("-instancename")
            .arg(&instance);
        fs::write(
            &receipt,
            serde_json::to_vec_pretty(
                &json!({"instance":instance,"start":"submitted; outcome unknown until acknowledgement","raw_trace_private":true}),
            )?,
        )?;
        let mut command = fixture
            .sandbox()?
            .command(&tools.wpr, format!("{name}-wpr-start.stderr.log"));
        hide(&mut command);
        command
            .args(["-start", "CPU", "-filemode", "-recordtempto"])
            .arg(&temporary)
            .arg("-instancename")
            .arg(&instance);
        command_output(
            command,
            &directory.join(format!("{name}-wpr-start.txt")),
            Duration::from_secs(30),
        )
        .context(
            "WPR Start was not acknowledged; never stop or cancel an unacknowledged instance",
        )?;
        let recording = Self {
            authority: Some(instance.clone()),
            path,
            receipt: receipt.clone(),
            stop,
            stdout,
        };
        // The guard already owns Stop if retaining this acknowledgement fails.
        fs::write(
            receipt,
            serde_json::to_vec_pretty(
                &json!({"instance":instance,"start":"acknowledged","raw_trace_private":true}),
            )?,
        )?;
        Ok(recording)
    }

    pub(super) fn stop(&mut self) -> Result<PathBuf> {
        let receipt = &self.receipt;
        let path = &self.path;
        let command = &mut self.stop;
        let stdout = &self.stdout;
        submit_stop(
            &mut self.authority,
            |instance| {
                fs::write(
                    receipt,
                    serde_json::to_vec_pretty(
                        &json!({"instance":instance,"start":"acknowledged","stop":"prepared; helper not yet spawned","raw_trace_private":true}),
                    )?,
                )?;
                Ok(())
            },
            |_| command.spawn().context("spawn prepared owned WPR Stop"),
            |child, instance| {
                let pid = child.id();
                let submitted = (|| -> Result<()> {
                    fs::write(
                        receipt,
                        serde_json::to_vec_pretty(
                            &json!({"instance":instance,"start":"acknowledged","stop":"submitted; outcome unknown until acknowledgement","stop_helper_pid":pid,"raw_trace_private":true}),
                        )?,
                    )?;
                    Ok(())
                })();
                if let Err(error) = &submitted {
                    eprintln!("WORKING PROFILE Stop submission receipt failed: {error:#}");
                }
                // A failed receipt must never skip waiting for this exact child.
                wait_for_output(child, stdout, Duration::from_secs(60), super::report::MAX_LINE)
                    .context("owned WPR Stop was not acknowledged; preserve partial trace and do not retry blindly")?;
                fs::write(
                    receipt,
                    serde_json::to_vec_pretty(
                        &json!({"instance":instance,"start":"acknowledged","stop":"acknowledged","stop_helper_pid":pid,"raw_trace_private":true,"trace":path}),
                    )?,
                )?;
                submitted.context("owned Stop completed but its submission receipt failed")?;
                let length = fs::metadata(path)?.len();
                ensure!(
                    length > 0 && length <= 512 * 1024 * 1024,
                    "raw trace is empty or exceeds 512 MiB"
                );
                Ok(path.clone())
            },
        )
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        if self.authority.is_some()
            && let Err(error) = self.stop()
        {
            eprintln!("WORKING PROFILE owned recording cleanup failed: {error:#}");
        }
    }
}

fn submit_stop(
    authority: &mut Option<String>,
    receipt: impl FnOnce(&str) -> Result<()>,
    spawn: impl FnOnce(&str) -> Result<Child>,
    finish: impl FnOnce(Child, &str) -> Result<PathBuf>,
) -> Result<PathBuf> {
    let instance = authority
        .as_ref()
        .context("owned trace Stop was already submitted")?
        .clone();
    let receipt = receipt(&instance);
    if let Err(error) = &receipt {
        eprintln!("WORKING PROFILE Stop preparation receipt failed: {error:#}");
    }
    let child = spawn(&instance).context("owned Stop helper never started; authority retained")?;
    // A real child now owns submission. Any later failure is never replayed.
    authority.take();
    let path = finish(child, &instance)?;
    receipt.context("owned Stop completed but its preparation receipt failed")?;
    Ok(path)
}

pub(super) fn ensure_available(status: &str, loggers: &str) -> Result<()> {
    ensure!(
        status
            .lines()
            .any(|line| line.trim() == "WPR is not recording")
            && !status.contains("recording is in progress"),
        "WPR does not establish an inactive default recording"
    );
    ensure!(
        loggers.len() <= 2 * 1024 * 1024,
        "logger inventory exceeds bound"
    );
    let mut current: Option<Logger<'_>> = None;
    let mut names = HashSet::new();
    for line in loggers.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "Logger Name" => {
                if let Some(logger) = current.take() {
                    logger.ensure_available()?;
                }
                ensure!(
                    !value.is_empty() && names.insert(value.to_ascii_lowercase()),
                    "empty or duplicate logger identity"
                );
                current = Some(Logger {
                    name: value,
                    mode: None,
                    flags: None,
                });
            }
            "Log File Mode" => {
                let logger = current.as_mut().context("logger mode precedes identity")?;
                ensure!(
                    logger.mode.replace(value).is_none(),
                    "duplicate logger mode"
                );
            }
            "Trace Flags" => {
                let logger = current.as_mut().context("trace flags precede identity")?;
                ensure!(
                    logger.flags.replace(value).is_none(),
                    "duplicate trace flags"
                );
            }
            _ => {}
        }
    }
    current
        .context("no complete logger inventory")?
        .ensure_available()
}

struct Logger<'a> {
    name: &'a str,
    mode: Option<&'a str>,
    flags: Option<&'a str>,
}

impl Logger<'_> {
    fn ensure_available(self) -> Result<()> {
        let name = self.name.to_ascii_lowercase();
        let wpr = name
            .split(|character: char| !character.is_ascii_alphanumeric())
            .any(|part| part == "wpr");
        ensure!(
            !wpr && name != "nt kernel logger",
            "existing WPR or kernel trace: {}",
            self.name
        );
        let mode = self.mode.context("incomplete logger mode")?;
        ensure!(!mode.is_empty(), "unclassified empty logger mode");
        // Observed non-system mode flags. Unknown modes require inspection, never a guess.
        for flag in mode.split_ascii_whitespace() {
            ensure!(
                matches!(
                    flag,
                    "AddToTriageDumps"
                        | "Append"
                        | "Buffered"
                        | "Circular"
                        | "IndependentSession"
                        | "LocalSeqNo"
                        | "NewFile"
                        | "NoPerProcessorBuffering"
                        | "NonStoppable"
                        | "PagedMemory"
                        | "PersistOnHybridShutdown"
                        | "Secure"
                        | "Sequential"
                        | "UseKBytes"
                        | "UseMsecFlushTimer"
                ),
                "potential system sampler or unclassified logger mode {flag}: {}",
                self.name
            );
        }
        let flags = self.flags.context("incomplete logger trace flags")?;
        // Eventlog-Security has a present, empty field in the actual runner inventory.
        if flags.is_empty() {
            return Ok(());
        }
        for provider in flags.split('+') {
            ensure_user_provider(provider)?;
        }
        Ok(())
    }
}

fn ensure_user_provider(provider: &str) -> Result<()> {
    let mut fields = provider.split(':');
    let identity = fields.next().context("missing provider identity")?;
    let normalized = if identity.starts_with('"') && identity.ends_with('"') && identity.len() > 2 {
        let identity = &identity[1..identity.len() - 1];
        ensure!(!identity.contains('"'), "unclassified quoted provider");
        identity
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    } else {
        uuid::Uuid::parse_str(identity)
            .context("unclassified legacy or system trace flags")?
            .to_string()
    };
    // SystemProfileProvider, PerfInfo and SystemTraceControlGuid are documented CPU/system identities.
    ensure!(
        !matches!(
            normalized.as_str(),
            "bfeb0324-1cee-496f-a409-2ac2b48a6322"
                | "ce1dbfb4-137e-4da6-87b0-3f59aa102cbc"
                | "9e814aad-3204-11d2-9a82-006008a86939"
                | "systemprofileprovider"
                | "perfinfo"
                | "systemtracecontrolguid"
        ),
        "existing system profiling provider"
    );
    for (index, field) in fields.enumerate() {
        ensure!(index < 2, "unclassified provider attributes");
        if field.is_empty() {
            continue;
        }
        let value = u64::from_str_radix(
            field
                .strip_prefix("0x")
                .context("unclassified provider attribute")?,
            16,
        )
        .context("invalid provider attribute")?;
        ensure!(
            index == 0 || value <= u8::MAX.into(),
            "provider level exceeds bound"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests;

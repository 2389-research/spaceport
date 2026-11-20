// ABOUTME: Qualifies Windows recorder ownership before the fixture starts a CPU trace.
// ABOUTME: Refuses existing or unclassified system sampling without stopping any session.
use anyhow::{Context as _, Result, ensure};
use std::collections::HashSet;

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
        ensure!(
            !name.contains("wpr") && name != "nt kernel logger",
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

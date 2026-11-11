//! Explicit native opt-in: this launches a real platform window in private state.
#![cfg(any(target_os = "macos", windows))]
#![allow(clippy::disallowed_methods)]
use anyhow::{Context as _, Result, bail};
#[path = "../../test-support/sandbox.rs"]
#[expect(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "shared test sandbox asserts preconditions and supplies daemon-only helpers"
)]
mod sandbox;

use sandbox::{Sandbox, stop_children};
use std::{
    fs,
    process::Child,
    time::{Duration, Instant},
};

struct Fixture {
    sandbox: Sandbox,
    child: Option<Child>,
}

#[derive(Clone, Copy)]
enum Algorithm {
    Cached,
    Reference,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        stop_children([&mut self.child]);
    }
}

#[test]
#[ignore = "requires release mode and a native macOS/Windows desktop"]
fn dense_terminal_sidebar_performance() -> Result<()> {
    run(Algorithm::Cached)
}

#[test]
#[ignore = "opens native reference fixture; run explicitly in release mode"]
fn reference_terminal_sidebar_performance() -> Result<()> {
    run(Algorithm::Reference)
}

fn run(algorithm: Algorithm) -> Result<()> {
    if cfg!(debug_assertions) {
        bail!("run the native benchmark with --release");
    }
    let mut fixture = Fixture {
        sandbox: Sandbox::new(),
        child: None,
    };
    let mut command = fixture
        .sandbox
        .command(env!("CARGO_BIN_EXE_spaceport"), "gui.log");
    if let Some(budget) = std::env::var_os("HERDR_PERF_P95_MS") {
        command.env("HERDR_PERF_P95_MS", budget);
    }
    if matches!(algorithm, Algorithm::Reference) {
        command.env("HERDR_PERF_UNCACHED", "1");
    }
    fixture.child = Some(
        command
            .arg("--performance-test")
            .spawn()
            .context("spawning native performance fixture")?,
    );
    let child = fixture.child.as_mut().context("missing fixture child")?;
    eprintln!(
        "PERF fixture pid={} sandbox={}",
        child.id(),
        fixture.sandbox.dir.display()
    );
    let start = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .context("polling native performance fixture")?
        {
            let log = fs::read_to_string(fixture.sandbox.dir.join("gui.log"))
                .context("reading native performance log")?;
            eprintln!("{log}");
            assert!(
                status.success(),
                "native performance fixture failed: {status}\n{log}"
            );
            assert!(log.contains("PERF native scroll list=0"));
            assert!(log.contains("PERF native scroll list=1"));
            match algorithm {
                Algorithm::Cached => {
                    assert!(log.contains("PERF mode=cached profile=release"));
                    assert!(log.contains("PERF native cached/fresh glyph layouts identical:"));
                    assert!(log.contains(
                        "PERF native changed-cell + centered popup + underline cursor verified"
                    ));
                }
                Algorithm::Reference => {
                    assert!(log.contains("PERF mode=reference profile=release"))
                }
            }
            assert!(log.contains("PERF PASS p95_budget_ms="));
            let dir = fixture.sandbox.dir.clone();
            drop(fixture);
            assert!(
                !dir.exists(),
                "fixture sandbox was not removed: {}",
                dir.display()
            );
            return Ok(());
        }
        if start.elapsed() > Duration::from_secs(180) {
            child
                .kill()
                .context("killing timed-out native performance fixture")?;
            child
                .wait()
                .context("reaping timed-out native performance fixture")?;
            bail!("native performance fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

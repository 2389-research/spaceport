// ABOUTME: Exercises bounded authored output through an owned Windows TUI workspace.
// ABOUTME: Requires observed seed, progress, and completion before accepting the fixture.
use super::{Fixture, check_environment};
use crate::working_profile::peer::{Peer, Targets, output::Producer};
use anyhow::{Context as _, Result, ensure};
use std::{
    fs, io,
    path::Path,
    time::{Duration, Instant},
};

const FINAL_STEP: u8 = 64;
const OUTPUT_LIMIT: Duration = Duration::from_secs(20);

#[derive(Debug, thiserror::Error)]
enum FixtureFailure {
    // keep-sorted start
    #[error("bounded TUI contract failed. See private {log}")]
    Retained { log: &'static str },
    #[error("bounded TUI contract failed and could not retain private {log}")]
    Retention {
        log: &'static str,
        #[source]
        source: io::Error,
    },
    // keep-sorted end
}

pub(crate) fn run() -> Result<()> {
    check_environment()?;
    let mut fixture = Fixture::prepare(false)?;
    let sandbox = fixture.sandbox()?;
    let directory = sandbox.dir.clone();
    let socket = sandbox.socket();

    let mut owned_workspace = None;
    let seeded = fixture.start_tui(|workspace| owned_workspace = Some(workspace.to_owned()));
    let seed_log = if owned_workspace.is_some() {
        "output-contract-seed.log"
    } else {
        "output-contract-seed-no-exact-workspace.log"
    };
    let result = retain_failure(&directory, seed_log, seeded)
        .map_err(anyhow::Error::from)
        .and_then(|targets| {
            eprintln!("BOUNDED TUI OUTPUT CONTRACT generated ready marker observed");
            verify_output(&socket, &targets)
        });

    // A later seed or stream failure still leaves the exact acknowledged workspace to close.
    let cleanup: Result<(), FixtureFailure> =
        owned_workspace.as_deref().map_or(Ok(()), |workspace| {
            let closed = Peer::open(&socket).and_then(|mut peer| peer.close_workspace(workspace));
            retain_failure(&directory, "output-contract-workspace-close.log", closed)?;
            eprintln!("BOUNDED TUI OUTPUT CONTRACT exact workspace close acknowledged");
            Ok(())
        });
    match (result, cleanup) {
        (Err(error), Err(cleanup)) => Err(error.context(cleanup)),
        (Ok(()), Err(cleanup)) => Err(cleanup.into()),
        (result, Ok(())) => result,
    }
}

fn retain_failure<T>(
    directory: &Path,
    log: &'static str,
    result: Result<T>,
) -> Result<T, FixtureFailure> {
    result.map_err(
        |error| match fs::write(directory.join(log), format!("{error:#?}\n")) {
            Ok(()) => FixtureFailure::Retained { log },
            Err(source) => FixtureFailure::Retention { log, source },
        },
    )
}

fn verify_output(socket: &Path, targets: &Targets) -> Result<()> {
    let deadline = Instant::now() + OUTPUT_LIMIT;
    let (mut producer, first) =
        Producer::start(socket, targets, deadline).context("start bounded TUI output")?;
    ensure!(
        first.last_step == 1 && !first.complete,
        "expected the first generated step, observed {first:?}"
    );

    let progress = producer
        .pump(deadline)
        .with_context(|| format!("observe later bounded TUI output after {first:?}"))?;
    ensure!(
        (first.last_step + 1..FINAL_STEP).contains(&progress.last_step) && !progress.complete,
        "expected an intermediate generated step, observed {progress:?}"
    );

    let receipt = producer
        .finish()
        .with_context(|| format!("finish bounded TUI output after {progress:?}"))?;
    ensure!(
        receipt.final_step == FINAL_STEP && receipt.complete,
        "expected natural completion by the deadline at step {FINAL_STEP}, observed {receipt:?}"
    );
    Ok(())
}

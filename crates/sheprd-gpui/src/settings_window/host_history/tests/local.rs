// ABOUTME: Checks local runtime-history cancellation and file conflict guards.
// ABOUTME: Exercises the same guarded writer used by the real Settings control.
use super::*;
use anyhow::Context as _;

#[test]
fn accepted_local_writes_outlive_the_settings_owner() -> anyhow::Result<()> {
    let mut history = HostHistory::default();
    let load = history
        .sync([("Local".into(), Target::Local)], false)
        .remove(0);
    assert!(history.finish(
        &Target::Local,
        load.generation,
        Ok(History::Local(Box::new(Settings::parse_text("")?))),
    ));
    let (_, save) = history
        .begin_save(&Target::Local)
        .context("local history is ready")?;
    drop(history);
    assert!(
        !save.cancel.load(Ordering::Acquire),
        "an accepted local write must finish after the window closes"
    );
    Ok(())
}

#[test]
fn dropping_a_saving_ssh_host_cancels_its_command() -> anyhow::Result<()> {
    let mut hosts = HostHistory::default();
    let target = Target::Ssh("a".into());
    let load = hosts
        .sync([("SSH".into(), target.clone())], false)
        .remove(0);
    assert!(hosts.finish(&target, load.generation, Ok(history(false)?)));
    let (_, save) = hosts.begin_save(&target).context("SSH history is ready")?;
    drop(hosts);
    assert!(save.cancel.load(Ordering::Acquire));
    Ok(())
}

#[cfg(unix)]
#[test]
fn local_runtime_history_refuses_cancelled_or_conflicting_writes() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("runtime.toml");
    let original = "[experimental]\npane_history = false\n";
    std::fs::write(&path, original)?;
    let history = History::Local(Box::new(Settings::load_fixture(path.clone())?));
    let error = history
        .save(true, &AtomicBool::new(true))
        .err()
        .context("cancelled write must fail")?;
    assert!(
        matches!(error, crate::Error::Io(error) if error.kind() == std::io::ErrorKind::Interrupted)
    );
    assert_eq!(std::fs::read_to_string(&path)?, original);

    let external = "# external edit\n[experimental]\npane_history = false\n";
    std::fs::write(&path, external)?;
    let error = history
        .save(true, &AtomicBool::new(false))
        .err()
        .context("conflicting write must fail")?;
    let error: &(dyn std::error::Error + 'static) = &error;
    assert!(
        std::iter::successors(Some(error), |error| error.source()).any(|error| {
            matches!(
                error.downcast_ref::<crate::herdr_settings::Error>(),
                Some(crate::herdr_settings::Error::Conflict)
            )
        })
    );
    assert_eq!(std::fs::read_to_string(path)?, external);
    Ok(())
}

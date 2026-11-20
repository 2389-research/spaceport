// ABOUTME: Exercises recorder refusal with observed telemetry shapes and documented system sampling identities.
// ABOUTME: Keeps ordinary OS kernel providers distinct from CPU profile sessions.
use super::*;
use std::{fs, path::PathBuf};

fn control_child(directory: &Path, script: &str) -> Result<Child> {
    let root = std::env::var_os("SystemRoot")
        .ok_or_else(|| anyhow::anyhow!("missing Windows SystemRoot"))?;
    let binary = Path::new(&root).join("System32/cmd.exe");
    let mut command = sheprd_client::new_command(binary);
    command
        .env_clear()
        .env("SystemRoot", root)
        .current_dir(directory)
        .args(["/d", "/c", script])
        .stdin(std::process::Stdio::null())
        .stdout(fs::File::create(directory.join("control.stdout"))?)
        .stderr(fs::File::create(directory.join("control.stderr"))?);
    Ok(command.spawn()?)
}

fn finish_control(child: Child, directory: &Path) -> Result<PathBuf> {
    let output = directory.join("control.stdout");
    wait_for_output(child, &output, Duration::from_secs(15), 65_536)?;
    Ok(output)
}

const TELEMETRY: &str = r#"Logger Name           : Mellanox-Kernel
Log File Mode         : Circular PersistOnHybridShutdown
Trace Flags           : 83793ec3-e8a3-4801-becf-5e921f79f06e:0x7fffffff:0x3

Logger Name           : Admin_PS_Provider
Log File Mode         : PersistOnHybridShutdown
Trace Flags           : "Microsoft-Windows-Kernel-Process":0x70:0xff
"#;

#[test]
fn observed_os_telemetry_without_cpu_recording_allows_preflight() -> Result<()> {
    ensure_available("WPR is not recording\r\n", TELEMETRY)
}

#[test]
fn observed_security_logger_has_a_present_empty_trace_flags_field() -> Result<()> {
    ensure_available(
        "WPR is not recording",
        "Logger Name : Eventlog-Security\nLog File Mode : NonStoppable Secure PersistOnHybridShutdown IndependentSession NoPerProcessorBuffering\nTrace Flags :\n",
    )
}

#[test]
fn observed_guid_providers_allow_omitted_keywords_and_levels() -> Result<()> {
    for provider in [
        "b1ad9b49-051a-4896-ae24-ebc9b8676e76::0x5",
        "2434b2ae-fcfd-4b69-9ab4-e58a0dd56565:0x3",
        "1b1d4ff4-f27b-4c99-8bd7-da8f1a74051a",
    ] {
        ensure_available(
            "WPR is not recording",
            &format!(
                "Logger Name : Observed telemetry\nLog File Mode : Circular\nTrace Flags : {provider}\n"
            ),
        )?;
    }
    Ok(())
}

#[test]
fn actual_low_privilege_defender_logger_is_not_a_wpr_identity() -> Result<()> {
    ensure_available(
        "WPR is not recording",
        r#"Logger Name : DefenderApiLoggerLowPriv
Log File Mode : PersistOnHybridShutdown IndependentSession NoPerProcessorBuffering
Trace Flags : b1c7a9e1-2e44-4c7f-9b30-6fa52f8a7711:0xffffffffffffffff:0xff+".NET Common Language Runtime":0x8:0xff+"Microsoft-Windows-PowerShell":0xffffffffffffffff:0xff
"#,
    )
}

#[test]
fn refuses_default_named_and_system_profile_recordings() {
    assert!(ensure_available("WPR recording is in progress", TELEMETRY).is_err());
    for logger in [
        "Logger Name : WPR_initiated_CPU_External\nLog File Mode : Sequential\nTrace Flags :\n",
        "Logger Name : NT Kernel Logger\nLog File Mode : Sequential\nTrace Flags : PROC_THREAD+LOADER+PROFILE\n",
        "Logger Name : Other\nLog File Mode : SystemLogger\nTrace Flags : 0x01000000\n",
        "Logger Name : Other\nLog File Mode : Sequential\nTrace Flags : bfeb0324-1cee-496f-a409-2ac2b48a6322:0x1:0x5\n",
    ] {
        assert!(ensure_available("WPR is not recording", logger).is_err());
    }
}

#[test]
fn refuses_missing_truncated_or_unclassified_logger_state() {
    for loggers in [
        "",
        "Logger Name : Other\nLog File Mode : Sequential\n",
        "Logger Name : Other\nLog File Mode : Sequential\nTrace Flags : UNKNOWN_KERNEL_FLAG\n",
    ] {
        assert!(ensure_available("WPR is not recording", loggers).is_err());
    }
    assert!(ensure_available("unrecognized WPR output", TELEMETRY).is_err());
}

#[test]
fn receipt_failure_still_submits_the_one_owned_stop() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let stopped = directory.path().join("control.stdout");
    let mut authority = Some("owned-profile-instance".to_owned());
    let result = submit_stop(
        &mut authority,
        |_| {
            fs::write(directory.path(), b"receipt")?;
            Ok(())
        },
        |_| control_child(directory.path(), "echo acknowledged"),
        |child, _| finish_control(child, directory.path()),
    );
    let error = result
        .err()
        .ok_or_else(|| anyhow::anyhow!("failed receipt unexpectedly succeeded"))?;
    assert!(
        error.downcast_ref::<std::io::Error>().is_some(),
        "receipt I/O cause was lost: {error:#}"
    );
    assert!(authority.is_none());
    assert!(
        fs::read(stopped)?.starts_with(b"acknowledged"),
        "receipt failure skipped the owned Stop child"
    );
    Ok(())
}

#[test]
fn unknown_stop_outcome_retires_authority_before_any_retry() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let replayed = directory.path().join("replayed");
    let mut authority = Some("owned-profile-instance".to_owned());
    let result = submit_stop(
        &mut authority,
        |_| Ok(()),
        |_| control_child(directory.path(), "exit 7"),
        |child, _| finish_control(child, directory.path()),
    );
    assert!(result.is_err());
    assert!(authority.is_none());
    let replay: Result<PathBuf> = submit_stop(
        &mut authority,
        |_| Ok(()),
        |_| {
            fs::write(&replayed, b"replayed")?;
            control_child(directory.path(), "exit 0")
        },
        |child, _| finish_control(child, directory.path()),
    );
    assert!(replay.is_err());
    assert!(
        !replayed.exists(),
        "post-spawn failure replayed the owned command"
    );
    Ok(())
}

#[test]
fn output_setup_failure_keeps_authority_when_stop_never_starts() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut authority = Some("owned-profile-instance".to_owned());
    let result = submit_stop(
        &mut authority,
        |_| Ok(()),
        |_| {
            // This is the actual stdout setup boundary before command_output can spawn.
            fs::File::create(directory.path().join("missing-parent/stop.stdout.log"))?;
            anyhow::bail!("invalid output path unexpectedly opened")
        },
        |child, _| finish_control(child, directory.path()),
    );
    let error = result
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing output parent unexpectedly succeeded"))?;
    assert_eq!(
        error
            .downcast_ref::<std::io::Error>()
            .ok_or_else(|| anyhow::anyhow!("lost output setup error source: {error:#}"))?
            .kind(),
        std::io::ErrorKind::NotFound
    );
    assert!(
        authority.is_some(),
        "pre-spawn log failure discarded active Stop authority"
    );
    Ok(())
}

#[test]
fn known_process_spawn_failure_keeps_unsubmitted_stop_authority() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut authority = Some("owned-profile-instance".to_owned());
    let result = submit_stop(
        &mut authority,
        |_| Ok(()),
        |_| {
            // A missing executable produces a real spawn error, with no child to reap.
            Ok(
                sheprd_client::new_command(directory.path().join("missing-stop-helper.exe"))
                    .spawn()?,
            )
        },
        |child, _| finish_control(child, directory.path()),
    );
    let error = result
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing stop executable unexpectedly succeeded"))?;
    assert_eq!(
        error
            .downcast_ref::<std::io::Error>()
            .ok_or_else(|| anyhow::anyhow!("lost process spawn error source: {error:#}"))?
            .kind(),
        std::io::ErrorKind::NotFound
    );
    assert!(
        authority.is_some(),
        "known spawn failure discarded unsubmitted Stop authority"
    );
    Ok(())
}

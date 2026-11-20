// ABOUTME: Validates the retained sampler definition through the actual installed WPR parser.
// ABOUTME: Checks private artifact identity without starting or stopping any recording.
use super::*;
use crate::working_profile::harness::Sandbox;
use sha2::{Digest as _, Sha256};

#[test]
fn retained_profile_has_native_sampled_stacks_and_matching_identity() -> Result<()> {
    let fixture = Fixture {
        sandbox: Some(Sandbox::new()),
        gui: None,
        server: None,
        request: 0,
    };
    let directory = &fixture.sandbox()?.dir;
    let selection = retain_profile(directory, "validation")?;
    let mut expected = directory.join("validation-cpu.wprp").into_os_string();
    expected.push("!SpaceportCpu");
    assert_eq!(selection, expected);
    let definition = fs::read(directory.join("validation-cpu.wprp"))?;
    assert_eq!(definition, CPU_PROFILE);
    let identity: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("validation-profile.json"))?)?;
    assert_eq!(identity["bytes"], definition.len());
    let digest: String = Sha256::digest(&definition)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(identity["sha256"], digest);
    assert_eq!(identity["selection"], selection.to_string_lossy().as_ref());

    let tools = Tools::installed()?;
    let mut command = fixture
        .sandbox()?
        .command(tools.wpr, "definition.stderr.log");
    hide(&mut command);
    command
        .arg("-profiledetails")
        .arg(selection)
        .arg("-filemode");
    let decoded = command_output(
        command,
        &directory.join("definition.txt"),
        Duration::from_secs(15),
    )?;
    let decoded = std::str::from_utf8(&decoded)?;
    assert!(decoded.contains("SpaceportCpu.Verbose.File"), "{decoded}");
    let stacks = decoded
        .split("System Stacks")
        .nth(1)
        .context("native stack definition")?;
    assert_eq!(
        stacks.split_whitespace().collect::<Vec<_>>(),
        ["SampledProfile"]
    );
    Ok(())
}

#[test]
fn retaining_profile_preserves_existing_definition_or_identity() -> Result<()> {
    for name in ["validation-cpu.wprp", "validation-profile.json"] {
        let directory = tempfile::tempdir()?;
        let existing = directory.path().join(name);
        fs::write(&existing, b"existing private artifact")?;
        assert!(retain_profile(directory.path(), "validation").is_err());
        assert_eq!(fs::read(existing)?, b"existing private artifact");
    }
    Ok(())
}

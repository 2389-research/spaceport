// ABOUTME: Diagnoses loaded Windows keyboard layouts before native AltGr qualification.
// ABOUTME: Starts no GUI and changes only its disposable helper thread's active layout.
use super::{Duration, Running, fs};

#[test]
#[ignore = "requires Windows native APIs; diagnoses loaded layouts without loading or typing"]
fn native_keyboard_layout_inventory_restores_its_helper_thread() {
    let mut running = Running::prepared();
    fs::write(
        running.sandbox.dir.join("native_layouts.cs"),
        include_str!("../support/native_layouts.cs"),
    )
    .unwrap();
    let output = running
        .shell_probe(
            r#"
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
Add-Type -Path "$PSScriptRoot\native_layouts.cs"
[Console]::WriteLine([SpaceportLayoutInventory]::Run())
"#,
            Duration::from_secs(20),
        )
        .unwrap();
    assert!(output.contains("LAYOUT_INVENTORY klids="), "{output}");
    assert!(
        output.contains("helper_restored=true failure_cleanup=true"),
        "{output}"
    );
    eprintln!("{output}");
}

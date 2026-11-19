# ABOUTME: Compiles and runs the empty-only native clipboard fixture on the disposable Windows runner.
# ABOUTME: Never reads unowned clipboard data or changes clipboard history preferences.
param([int]$ProcessId, [ValidateSet('Ownership','Watchdog')][string]$Scenario)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -Path "$PSScriptRoot\native_clipboard.cs" -ReferencedAssemblies @(
    [System.Windows.Forms.Form].Assembly.Location,
    [System.Drawing.Point].Assembly.Location,
    [System.ComponentModel.Win32Exception].Assembly.Location
)
[NativeClipboardChecks]::Run("$PSScriptRoot\clipboard-stop", $Scenario -eq 'Watchdog')

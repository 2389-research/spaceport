# ABOUTME: Runs the bounded native announcement listener for one owned fixture window.
# ABOUTME: Writes registration and removal receipts without PowerShell worker callbacks.
param([int]$ProcessId)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type -Path "$PSScriptRoot\uia_live.cs" -ReferencedAssemblies @(
    [System.Windows.Automation.AutomationElement].Assembly.Location,
    [System.Windows.Automation.AutomationEvent].Assembly.Location
)
$process = Get-Process -Id $ProcessId
$handle = $process.MainWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'The private GUI has no main window' }
$observer = [SpaceportLiveEvents]::new($handle, $ProcessId)
try {
    [Console]::WriteLine('UIA_LIVE_READY')
    [System.IO.File]::WriteAllText("$PSScriptRoot\live-events-ready", 'ready')
    $deadline = [DateTime]::UtcNow.AddSeconds(60)
    while (-not [System.IO.File]::Exists("$PSScriptRoot\live-events-stop")) {
        if ($process.HasExited) { throw 'The owned GUI exited during native observation' }
        if ([DateTime]::UtcNow -ge $deadline) { throw 'Native announcement observation exceeded its deadline' }
        foreach ($record in $observer.Drain()) {
            [Console]::WriteLine('UIA_LIVE ' + (ConvertTo-Json -InputObject $record -Compress))
        }
        Start-Sleep -Milliseconds 25
    }
} finally {
    $observer.Dispose()
}
foreach ($record in $observer.Drain()) {
    [Console]::WriteLine('UIA_LIVE ' + (ConvertTo-Json -InputObject $record -Compress))
}
[Console]::WriteLine('UIA_LIVE_REMOVED')

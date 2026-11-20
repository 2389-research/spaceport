# ABOUTME: Prepares animations only on disposable Windows Actions desktops.
# ABOUTME: Saves the original preference and verifies restoration for native profiling.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Enable', 'Restore')]
    [string]$Action,
    [Parameter(Mandatory = $true)]
    [string]$EvidenceDirectory
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') {
    throw 'Desktop animation preparation requires a disposable Windows Actions runner.'
}
if (-not (Test-Path -LiteralPath $EvidenceDirectory -PathType Container)) {
    throw 'The private evidence directory must already exist.'
}
$null = [Windows.UI.ViewManagement.UISettings,Windows.UI.ViewManagement,ContentType=WindowsRuntime]
function Get-DesktopAnimations {
    $settings = [Windows.UI.ViewManagement.UISettings]::new()
    return [bool]$settings.AnimationsEnabled
}
function Set-DesktopAnimations {
    param([bool]$Enabled)
}
$saved = Join-Path $EvidenceDirectory 'desktop-animation-original.json'
if ($Action -eq 'Enable') {
    $original = Get-DesktopAnimations
    $stream = [IO.File]::Open($saved, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $writer = [IO.StreamWriter]::new($stream, [Text.UTF8Encoding]::new($false))
        try { $writer.Write(([ordered]@{animations_enabled = $original} | ConvertTo-Json -Compress)) }
        finally { $writer.Dispose() }
    } finally { $stream.Dispose() }
    if (-not $original) { Set-DesktopAnimations $true }
} else {
    $record = Get-Content -LiteralPath $saved -Raw | ConvertFrom-Json
    if ($record.animations_enabled -isnot [bool]) { throw 'The saved animation preference must be a Boolean.' }
    $original = $record.animations_enabled
    if ((Get-DesktopAnimations) -ne $original) { Set-DesktopAnimations $original }
    if ((Get-DesktopAnimations) -ne $original) { throw 'Desktop animation restoration did not take effect.' }
}
[ordered]@{action = $Action; original = $original; animations_enabled = (Get-DesktopAnimations)} |
    ConvertTo-Json -Compress | Write-Output

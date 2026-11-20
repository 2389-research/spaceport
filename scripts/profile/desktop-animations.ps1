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
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class SpaceportDesktopAnimations {
    [DllImport("user32.dll", EntryPoint = "SystemParametersInfoW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool Read(uint action, uint parameter, [MarshalAs(UnmanagedType.Bool)] out bool value, uint flags);
    [DllImport("user32.dll", EntryPoint = "SystemParametersInfoW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool Write(uint action, uint parameter, IntPtr value, uint flags);
}
'@
function Get-DesktopAnimations {
    $settings = [Windows.UI.ViewManagement.UISettings]::new()
    return [bool]$settings.AnimationsEnabled
}
function Get-ClientAreaAnimations {
    $enabled = $false
    if (-not [SpaceportDesktopAnimations]::Read(0x1042, 0, [ref]$enabled, 0)) {
        throw [ComponentModel.Win32Exception]::new([Runtime.InteropServices.Marshal]::GetLastWin32Error())
    }
    return $enabled
}
function Set-DesktopAnimations {
    param([bool]$Enabled)
    $value = if ($Enabled) { [IntPtr]1 } else { [IntPtr]::Zero }
    if (-not [SpaceportDesktopAnimations]::Write(0x1043, 0, $value, 0)) {
        throw [ComponentModel.Win32Exception]::new([Runtime.InteropServices.Marshal]::GetLastWin32Error())
    }
}
$saved = Join-Path $EvidenceDirectory 'desktop-animation-original.json'
if ($Action -eq 'Enable') {
    $original = Get-DesktopAnimations
    $nativeOriginal = Get-ClientAreaAnimations
    $stream = [IO.File]::Open($saved, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $writer = [IO.StreamWriter]::new($stream, [Text.UTF8Encoding]::new($false))
        try {
            $writer.Write(([ordered]@{animations_enabled = $original; client_area_animations = $nativeOriginal} | ConvertTo-Json -Compress))
        }
        finally { $writer.Dispose() }
    } finally { $stream.Dispose() }
    if (-not $nativeOriginal) { Set-DesktopAnimations $true }
} else {
    $record = Get-Content -LiteralPath $saved -Raw | ConvertFrom-Json
    if ($record.animations_enabled -isnot [bool]) { throw 'The saved animation preference must be a Boolean.' }
    if ($record.client_area_animations -isnot [bool]) { throw 'The saved client-area preference must be a Boolean.' }
    $original = $record.animations_enabled
    $nativeOriginal = $record.client_area_animations
    if ((Get-ClientAreaAnimations) -ne $nativeOriginal) { Set-DesktopAnimations $nativeOriginal }
    if ((Get-ClientAreaAnimations) -ne $nativeOriginal) { throw 'Client-area animation restoration did not take effect.' }
    if ((Get-DesktopAnimations) -ne $original) { throw 'Desktop animation restoration did not take effect.' }
}
[ordered]@{action = $Action; original = $original; native_original = $nativeOriginal; animations_enabled = (Get-DesktopAnimations); client_area_animations = (Get-ClientAreaAnimations)} |
    ConvertTo-Json -Compress | Write-Output

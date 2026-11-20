# ABOUTME: Reads activation and exact thread ownership for the profile's owned Win32 HWND.
# ABOUTME: Observes native state without activating a window or sending desktop input.
param([long]$WindowHandle, [uint32]$ProcessId, [string]$ReceiptFile)
$ErrorActionPreference = 'Stop'
function Write-Stage([string]$stage) {
  $line = [DateTime]::UtcNow.ToString('O') + ' ' + $stage
  [Console]::Error.WriteLine('WORKING PROFILE native ' + $line)
  [IO.File]::AppendAllText($ReceiptFile, $line + [Environment]::NewLine)
}
Write-Stage 'script_enter'
Write-Stage 'utility_import_begin'
Microsoft.PowerShell.Core\Import-Module -Name Microsoft.PowerShell.Utility -ErrorAction Stop
Write-Stage 'utility_import_complete'
Write-Stage 'add_type_begin'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
[assembly: DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
public static class WorkingProfileWindow {
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint owner);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr window);
}
'@
Write-Stage 'add_type_complete'
$window = [IntPtr]$WindowHandle
[uint32]$owner = 0
Write-Stage 'ownership_begin'
$thread = [WorkingProfileWindow]::GetWindowThreadProcessId($window, [ref]$owner)
Write-Stage ('ownership_complete thread=' + $thread + ' owner=' + $owner)
if ($thread -eq 0 -or $owner -ne $ProcessId) {
  throw 'Profile HWND no longer belongs to the exact GUI process.'
}
[ordered]@{
  hwnd = $WindowHandle
  gui_pid = $owner
  thread_id = $thread
  foreground_hwnd = [WorkingProfileWindow]::GetForegroundWindow().ToInt64()
  visible = [WorkingProfileWindow]::IsWindowVisible($window)
  minimized = [WorkingProfileWindow]::IsIconic($window)
} | ConvertTo-Json -Compress

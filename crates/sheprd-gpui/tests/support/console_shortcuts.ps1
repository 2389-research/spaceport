# ABOUTME: Captures real key and modifier records inside the private Herdr console pane.
# ABOUTME: Publishes nonce-bound attachment and result receipts without echoing typed keys.
param([string]$Nonce)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
if ($Nonce -notmatch '^[a-f0-9]{32}$') { throw 'Console capture requires a private nonce' }
Add-Type @'
using System;
using System.Runtime.InteropServices;
[assembly: DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
public static class SpaceportConsoleCapture {
  [DllImport("kernel32.dll", SetLastError=true)] public static extern IntPtr GetStdHandle(int id);
  [DllImport("kernel32.dll", SetLastError=true)] public static extern bool GetConsoleMode(IntPtr handle, out uint mode);
}
'@
$inputHandle = [SpaceportConsoleCapture]::GetStdHandle(-10)
$mode = [uint32]0
if ([Console]::IsInputRedirected -or -not [SpaceportConsoleCapture]::GetConsoleMode($inputHandle, [ref]$mode)) {
  throw 'Pane producer has no attached real console input'
}
$table = @(Get-Content -LiteralPath "$PSScriptRoot\windows_shortcut_cases.json" -Raw | ConvertFrom-Json)
$cases = @($table | Where-Object { $_.outcome.kind -eq 'pane_key' })
if ($cases.Count -lt 2 -or $cases.Count -gt 8 -or $cases[0].id -cne 'console-attachment') {
  throw 'Console prerequisite is not first in the fixed key cases'
}
function Publish([string]$kind, [string]$id, $value) {
  if ($id -notmatch '^[a-z0-9-]{1,64}$') { throw 'Console case ID exceeds fixture bounds' }
  $path = Join-Path $PSScriptRoot "console-$kind-$Nonce-$id.json"
  $temporary = "$path.tmp"
  [IO.File]::WriteAllText($temporary, ($value | ConvertTo-Json -Compress), [Text.UTF8Encoding]::new($false))
  [IO.File]::Move($temporary, $path)
}
foreach ($case in $cases) {
  Publish 'ready' $case.id @{ nonce=$Nonce; id=$case.id; attached=$true; producer=$PID; mode=$mode }
  [Console]::WriteLine("SP_CONSOLE_READY $Nonce $($case.id)")
  $deadline = [DateTime]::UtcNow.AddSeconds(35)
  while (-not [Console]::KeyAvailable) {
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Private console key did not arrive' }
    Start-Sleep -Milliseconds 10
  }
  $key = [Console]::ReadKey($true)
  $modifiers = @('Alt','Shift','Control' | Where-Object {
    ([int]$key.Modifiers -band [int][Enum]::Parse([System.ConsoleModifiers], $_)) -ne 0
  })
  Publish 'result' $case.id @{ nonce=$Nonce; id=$case.id; attached=$true; producer=$PID;
    key=$key.Key.ToString(); modifiers=$modifiers; character=[int]$key.KeyChar }
  [Console]::WriteLine("SP_CONSOLE_RESULT $Nonce $($case.id) $($key.Key) $($key.Modifiers)")
}

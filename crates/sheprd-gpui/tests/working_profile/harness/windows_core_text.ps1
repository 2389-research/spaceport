# ABOUTME: Reads bounded UI Automation text from the mixed profile's exact owned Windows HWND.
# ABOUTME: Retains marker counts and geometry without storing the text returned by UI Automation.
param([long]$WindowHandle, [uint32]$ProcessId, [uint32]$ThreadId)
$ErrorActionPreference = 'Stop'
$receipt = [ordered]@{
  platform = 'windows'
  status = 'failed'
  stage = 'script_enter'
  hwnd = $WindowHandle
  owner_before = $null
  owner_after = $null
  positive = $null
  absent = $null
  failure = $null
  exception_type = $null
}
function Write-Stage([string]$stage, $metadata) {
  $receipt.stage = $stage
  $line = [DateTime]::UtcNow.ToString('O') + ' ' + $stage
  if ($null -ne $metadata) { $line += ' ' + ($metadata | ConvertTo-Json -Depth 4 -Compress) }
  [IO.File]::AppendAllText('core-native-text.stages.txt', $line + [Environment]::NewLine)
}
function Fail-Read([string]$reason) {
  $receipt.failure = $reason
  throw 'Native Core text check failed.'
}

Write-Stage 'utility_import_begin'
Microsoft.PowerShell.Core\Import-Module -Name Microsoft.PowerShell.Utility -ErrorAction Stop
try {
  Write-Stage 'assembly_load_begin'
  Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
  Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
[assembly: DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
public static class WorkingProfileTextWindow {
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint owner);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr window);
}
'@
  $window = [IntPtr]$WindowHandle
  $marker = $env:SPACEPORT_PROFILE_CORE_MARKER
  $absentMarker = 'SP_CORE_UIA_AUTHORED_ABSENT_2B74E3EA'
  if ([string]::IsNullOrEmpty($marker) -or $marker.Length -gt 128 -or $marker -ceq $absentMarker) {
    Fail-Read 'invalid authored marker'
  }
  $automation = [System.Windows.Automation.AutomationElement]
  $condition = [System.Windows.Automation.AndCondition]::new(
    [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Document),
    [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Terminal')
  )

  function Require-OwnedWindow([string]$checkpoint) {
    [uint32]$owner = 0
    $thread = [WorkingProfileTextWindow]::GetWindowThreadProcessId($window, [ref]$owner)
    $visible = [WorkingProfileTextWindow]::IsWindowVisible($window)
    $minimized = [WorkingProfileTextWindow]::IsIconic($window)
    $receipt[$checkpoint] = [ordered]@{ pid = $owner; thread_id = $thread; visible = $visible; minimized = $minimized }
    if ($thread -eq 0 -or $thread -ne $ThreadId -or $owner -ne $ProcessId -or -not $visible -or $minimized) {
      Fail-Read 'Win32 identity or visibility differs from the observed window'
    }
  }

  function Read-TerminalDocuments([string]$control, [int]$attempt) {
    $checkAbsent = $control -eq 'absent'
    $sample = [ordered]@{
      attempt = $attempt
      complete = $false
      named_documents = 0
      visible_documents = 0
      readable_documents = 0
      positive_matches = 0
      absent_matches = $null
      document_bounds = @()
    }
    $receipt[$control] = $sample
    if ($checkAbsent) { $sample.absent_matches = 0 }

    $root = $automation::FromHandle($window)
    if ($null -eq $root) { return }
    $current = $root.Current
    if ($current.ProcessId -ne $ProcessId -or [IntPtr]$current.NativeWindowHandle -ne $window) {
      Fail-Read 'UI Automation root differs from the observed window'
    }
    $documents = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
    $sample.named_documents = $documents.Count
    if ($documents.Count -gt 16) { Fail-Read 'named document count exceeds 16' }
    foreach ($document in $documents) {
      $current = $document.Current
      if ($current.ProcessId -ne $ProcessId) { Fail-Read 'terminal document belongs to another process' }
      if ($current.IsOffscreen) { continue }
      $bounds = $current.BoundingRectangle
      if ($bounds.Width -le 0 -or $bounds.Height -le 0) { continue }
      foreach ($number in @($bounds.Left, $bounds.Top, $bounds.Width, $bounds.Height)) {
        if ([double]::IsNaN($number) -or [double]::IsInfinity($number)) { Fail-Read 'terminal bounds are not finite' }
      }
      $sample.visible_documents += 1
      $documentReceipt = [ordered]@{
        pid = $current.ProcessId; positive_match = $null
        left = $bounds.Left; top = $bounds.Top; width = $bounds.Width; height = $bounds.Height
      }
      $sample.document_bounds += $documentReceipt
      $pattern = $null
      if (-not $document.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref]$pattern)) { continue }
      $text = $pattern.DocumentRange.GetText(16384)
      if ($null -eq $text) { continue }
      $sample.readable_documents += 1
      $documentReceipt.positive_match = $text.IndexOf($marker, [StringComparison]::Ordinal) -ge 0
      if ($documentReceipt.positive_match) { $sample.positive_matches += 1 }
      if ($checkAbsent -and $text.IndexOf($absentMarker, [StringComparison]::Ordinal) -ge 0) { $sample.absent_matches += 1 }
    }
    $sample.complete = $true
  }

  function Require-Control([string]$control) {
    Write-Stage ($control + '_begin')
    $clock = [Diagnostics.Stopwatch]::StartNew()
    for ($attempt = 1; $attempt -le 50 -and $clock.ElapsedMilliseconds -lt 5000; $attempt += 1) {
      try { Read-TerminalDocuments $control $attempt }
      catch [System.Windows.Automation.ElementNotAvailableException] {
        # AccessKit can replace nodes while the native tree is becoming ready.
      }
      $sample = $receipt[$control]
      if ($sample.positive_matches -gt 1) { Fail-Read 'multiple documents contain the Core marker' }
      if ($sample.absent_matches -gt 0) { Fail-Read 'a document contains the absent marker' }
      if ($sample.complete -and $sample.visible_documents -eq $sample.readable_documents -and $sample.positive_matches -eq 1) {
        Write-Stage ($control + '_complete') $sample
        return
      }
      [Threading.Thread]::Sleep(100)
    }
    Fail-Read 'marker control did not reach a readable and unique terminal document before the deadline'
  }

  Write-Stage 'owner_before'
  Require-OwnedWindow 'owner_before'
  Write-Stage 'owner_before_complete' $receipt.owner_before
  try {
    Require-Control 'positive'
    Require-Control 'absent'
  } finally {
    Require-OwnedWindow 'owner_after'
  }
  Write-Stage 'complete' $receipt.owner_after
  $receipt.status = 'qualified'
} catch {
  $receipt.exception_type = $_.Exception.GetBaseException().GetType().Name
}
[Console]::WriteLine(($receipt | ConvertTo-Json -Depth 6 -Compress))
if ($receipt.status -ne 'qualified') { exit 1 }

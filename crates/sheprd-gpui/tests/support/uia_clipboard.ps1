# ABOUTME: Pastes authored native clipboard text through the actual chat or standalone field.
# ABOUTME: Checks limits, cancellation before acquisition completes, and native reader recovery.
param([int]$ProcessId, [ValidateSet('Chat','Field')][string]$Scenario)
. "$PSScriptRoot\uia_tree.ps1" -ProcessId $ProcessId
Add-Type -AssemblyName System.Drawing
Add-Type -Path "$PSScriptRoot\native_clipboard.cs" -ReferencedAssemblies @(
  [System.Windows.Forms.Form].Assembly.Location,
  [System.Drawing.Point].Assembly.Location,
  [System.ComponentModel.Win32Exception].Assembly.Location
)
if ($windows.Count -ne 1) { throw 'Native clipboard check requires exactly one owned window' }
$root = $windows[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Native clipboard window has no HWND' }
$name = if ($Scenario -eq 'Chat') { 'Message the agent' } else { 'Native standalone field' }
$condition = [System.Windows.Automation.AndCondition]::new(
  [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit),
  [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, $name)
)
$edits = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
if ($edits.Count -ne 1) { throw 'Native clipboard check has no unique named field' }
$field = $edits[0]
$value = $field.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
function Wait-For([scriptblock]$condition, [string]$failure, [int]$milliseconds = 5000) {
  $deadline = [DateTime]::UtcNow.AddMilliseconds($milliseconds)
  while (-not (& $condition)) {
    if ([DateTime]::UtcNow -ge $deadline) { throw $failure }
    Start-Sleep -Milliseconds 10
  }
}
function Require-Window {
  if ([HerdrUia]::GetForegroundWindow() -ne $handle) { throw 'Native clipboard check lost exact foreground HWND' }
  $element = $automation::FocusedElement
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Native clipboard focus left its owned process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Native clipboard focus left its owned window'
}
function Require-Released {
  foreach ($key in @(0x10, 0x11, 0x12, 0x5b, 0x5c)) {
    if (([HerdrUia]::GetAsyncKeyState($key) -band 0x8000) -ne 0) { throw 'A modifier is already held' }
  }
}
function Focus-Field {
  Require-Window
  $field.SetFocus()
  Wait-For { [System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $field) } 'Native field did not acknowledge focus' 1000
  Require-Window
}
function Send-FieldKeys([string]$keys) {
  Require-Window
  if (-not [System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $field)) { throw 'Native paste field lost focus before a key' }
  Require-Released
  [System.Windows.Forms.SendKeys]::SendWait($keys)
}
function Expect-Value([string]$expected) {
  Wait-For { $value.Current.Value -ceq $expected } 'Native field value differs from authored expectation'
}
function Reset-Draft {
  Focus-Field
  Send-FieldKeys '^a{BACKSPACE}draft'
  Expect-Value 'draft'
  Expect-Error 'none'
}
function Read-State {
  $stream = [System.IO.File]::Open("$PSScriptRoot\gui.log", [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
  try {
    $bytes = [byte[]]::new(1048577)
    $length = 0
    while ($length -lt $bytes.Length) {
      $read = $stream.Read($bytes, $length, $bytes.Length - $length)
      if ($read -eq 0) { break }
      $length += $read
    }
    if ($length -gt 1048576) { throw 'Native clipboard log exceeds bound' }
    $log = [System.Text.Encoding]::UTF8.GetString($bytes, 0, $length)
  } finally { $stream.Dispose() }
  $states = [regex]::Matches($log, 'CLIPBOARD_STATE pending=(true|false) reader=(true|false) utf8_bytes=(\d+) error=(none|prompt|acquisition|other)')
  if ($states.Count -eq 0) { return '' }
  $states[$states.Count - 1].Value
}
function Expect-State([bool]$pending, [bool]$reading, [int]$milliseconds = 1000) {
  $prefix = "CLIPBOARD_STATE pending=$($pending.ToString().ToLowerInvariant()) reader=$($reading.ToString().ToLowerInvariant()) "
  Wait-For { (Read-State).StartsWith($prefix) } 'Native clipboard pending/reader state did not match' $milliseconds
}
function Expect-Error([string]$kind) {
  Wait-For { (Read-State).EndsWith(" error=$kind") } 'Real field did not expose the expected paste error state'
}
Require-Released
try { [HerdrUia]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero) }
finally { [HerdrUia]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero) }
[HerdrUia]::SetForegroundWindow($handle) | Out-Null
Wait-For { [HerdrUia]::GetForegroundWindow() -eq $handle } 'Native clipboard window did not become foreground'
$stop = "$PSScriptRoot\clipboard-stop"
[NativeClipboardChecks]::RunOwned($stop, 90, [Action]{
  $owner = [NativeClipboardFixture]::new()
  try {
    Focus-Field
    $small = "Authored`r`n" + [char]0xe9 + [char]0x754c + [char]::ConvertFromUtf32(0x1f980)
    $owner.Publish($small)
    Send-FieldKeys '^v'
    Expect-Value ($small.Replace("`r`n", "`n"))
    [Console]::WriteLine('CLIPBOARD_APPLICATION unicode_crlf=true')

    Send-FieldKeys '^a{BACKSPACE}'
    Expect-Value ''
    $exact = [string]::new('a', 262142) + [char]0xe9
    $owner.Publish($exact)
    Send-FieldKeys '^v'
    Expect-Value $exact
    [Console]::WriteLine('CLIPBOARD_APPLICATION exact_utf8_bytes=262144 accepted=true')

    foreach ($stage in @('Prompt', 'Acquisition')) {
      Reset-Draft
      $length = if ($stage -eq 'Prompt') { 262145 } else { 2097153 }
      $payload = if ($stage -eq 'Prompt') { $exact + 'a' } else { [string]::new('a', $length) }
      if ([System.Text.Encoding]::UTF8.GetByteCount($payload) -ne $length) { throw 'Authored clipboard payload has an unexpected byte count' }
      $owner.Publish($payload)
      Send-FieldKeys '^v'
      Expect-Error ($stage.ToLowerInvariant())
      Expect-Value 'draft'
      Expect-State $false $false 5000
      [Console]::WriteLine("CLIPBOARD_APPLICATION rejected_utf8_bytes=$length draft_preserved=true error=$stage")
    }

    foreach ($cancel in @('Type', 'Blur')) {
      Reset-Draft
      $owner.Publish('Authored late paste')
      $owner.Hold([Action]{
        Send-FieldKeys '^v'
        Expect-State $true $true
        if ($cancel -eq 'Type') { Send-FieldKeys 'Z' }
        else {
          $targetName = if ($Scenario -eq 'Chat') { 'Attach files' } else { 'Native blur target' }
          $target = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,
            [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, $targetName))
          if ($null -eq $target) { throw 'Native blur target is missing' }
          Require-Window
          $target.SetFocus()
          Wait-For { [System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $target) } 'Native blur did not take focus' 500
        }
        Expect-State $false $true
        [Console]::WriteLine("CLIPBOARD_APPLICATION cancel=$cancel observed_before_release=true reader_still_active=true")
      })
      Expect-State $false $false 5000
      Expect-Value $(if ($cancel -eq 'Type') { 'draftZ' } else { 'draft' })
      Reset-Draft
      $owner.Publish('recovered')
      Send-FieldKeys '^a{BACKSPACE}^v'
      Expect-Value 'recovered'
      Expect-State $false $false 5000
      [Console]::WriteLine("CLIPBOARD_APPLICATION cancel=$cancel stale_absent=true reader_recovered=true")
    }
    [Console]::WriteLine("CLIPBOARD_APPLICATION native PASS scenario=$Scenario")
  } finally { $owner.Dispose() }
})

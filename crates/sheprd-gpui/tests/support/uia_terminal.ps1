# ABOUTME: Queries real terminal text, its unselected caret, and native output-change events.
# ABOUTME: Sends authored input only to the exact private foreground terminal window.
param([int]$ProcessId)
. "$PSScriptRoot\uia_tree.ps1" -ProcessId $ProcessId
Add-Type -Path "$PSScriptRoot\uia_live.cs" -ReferencedAssemblies @(
  [System.Windows.Automation.AutomationElement].Assembly.Location,
  [System.Windows.Automation.AutomationEvent].Assembly.Location
)
if ($windows.Count -ne 1) { throw 'Terminal check requires one owned window' }
$root = $windows[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Terminal check has no owned HWND' }
$condition = [System.Windows.Automation.AndCondition]::new(
  [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Document),
  [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Terminal')
)
$documents = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
if ($documents.Count -ne 1) { throw 'Terminal check has no unique named document' }
$document = $documents[0]
$pattern = $document.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
function Require-Terminal {
  if ([HerdrUia]::GetForegroundWindow() -ne $handle) { throw 'Terminal check lost exact foreground HWND' }
  $element = $automation::FocusedElement
  if (-not [System.Windows.Automation.Automation]::Compare($element, $document)) { throw 'Terminal document does not own native focus' }
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Terminal focus left its owned process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Terminal focus left its owned window'
}
function Require-Released {
  foreach ($key in @(0x10, 0x11, 0x12, 0x5b, 0x5c)) {
    if (([HerdrUia]::GetAsyncKeyState($key) -band 0x8000) -ne 0) { throw 'A modifier is already held' }
  }
}
Require-Released
try { [HerdrUia]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero) }
finally { [HerdrUia]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero) }
[HerdrUia]::SetForegroundWindow($handle) | Out-Null
$document.SetFocus()
$deadline = [DateTime]::UtcNow.AddSeconds(3)
while (-not [System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $document)) {
  if ([DateTime]::UtcNow -ge $deadline) { throw 'Native terminal focus was not acknowledged' }
  Start-Sleep -Milliseconds 10
}
Require-Terminal
$observer = [SpaceportLiveEvents]::new($handle, $ProcessId, [System.Windows.Automation.TextPattern]::TextChangedEvent)
try {
  Require-Terminal
  Require-Released
  [System.Windows.Forms.SendKeys]::SendWait('echo SP_A11Y_7{ENTER}')
  $changed = $false
  $deadline = [DateTime]::UtcNow.AddSeconds(5)
  do {
    Require-Terminal
    if ($observer.Drain() -ccontains 'Terminal') { $changed = $true }
    # Resolve a fresh range after each native text-change event.
    $text = $pattern.DocumentRange.GetText(16384)
    $output = ($text -split '\r?\n') -ccontains 'SP_A11Y_7'
    if ($changed -and $output) { break }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Native terminal text or changed-output event is missing' }
    Start-Sleep -Milliseconds 25
  } while ($true)
  $selection = $pattern.GetSelection()
  if ($selection.Length -ne 1) { throw 'Native unselected terminal has no unique caret range' }
  $start = [System.Windows.Automation.Text.TextPatternRangeEndpoint]::Start
  $end = [System.Windows.Automation.Text.TextPatternRangeEndpoint]::End
  if ($selection[0].CompareEndpoints($start, $selection[0], $end) -ne 0 -or $selection[0].GetText(1) -cne '') {
    throw 'Native terminal reports a selection instead of its unselected caret'
  }
  $rectangles = $selection[0].GetBoundingRectangles()
  if ($rectangles.Length -ne 4) { throw 'Native caret has no unique bounding rectangle' }
  $bounds = $document.Current.BoundingRectangle
  $x = $rectangles[0]; $y = $rectangles[1]; $width = $rectangles[2]; $height = $rectangles[3]
  foreach ($number in $rectangles) {
    if ([double]::IsNaN($number) -or [double]::IsInfinity($number)) { throw 'Native caret bounds are not finite' }
  }
  if ($width -lt 0 -or $height -le 0 -or $x -lt $bounds.Left - 1 -or $y -lt $bounds.Top - 1 -or
      $x + $width -gt $bounds.Right + 1 -or $y + $height -gt $bounds.Bottom + 1) {
    throw 'Native caret bounds escape the terminal document'
  }
  [Console]::WriteLine('TERMINAL_A11Y text=true native_text_changed=true unselected_caret=true caret_bounds=true')
} finally { $observer.Dispose() }
[Console]::WriteLine('TERMINAL_A11Y observer_removed=true')

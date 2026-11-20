# ABOUTME: Queries real terminal text, its unselected caret, and native output-change events.
# ABOUTME: Sends authored input only to the exact private foreground terminal window.
param([int]$ProcessId, [ValidateSet('Text','Resize','Observe')][string]$Action = 'Text')
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
if ($Action -eq 'Resize') {
  $documentBounds = $document.Current.BoundingRectangle
  $deadline = [DateTime]::UtcNow.AddSeconds(5)
  do {
    Require-Terminal
    $selection = $pattern.GetSelection()
    $ready = $false
    if ($selection.Length -eq 1) {
      $line = $selection[0].Clone()
      $line.ExpandToEnclosingUnit([System.Windows.Automation.Text.TextUnit]::Line)
      $rectangles = $line.GetBoundingRectangles()
      $ready = $rectangles.Length -eq 1 -and $rectangles[0].Left -gt $documentBounds.Left + $documentBounds.Width / 3
    }
    if ($ready) { break }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'The private right split has no presented native caret line' }
    Start-Sleep -Milliseconds 25
  } while ($true)
  [Console]::WriteLine("TERMINAL_RESIZE_STAGE presented_right=true left=$($rectangles[0].Left)")
  function Press-Resize([string]$key) {
    Require-Terminal
    Require-Released
    [System.Windows.Forms.SendKeys]::SendWait($key)
    Require-Released
    Require-Terminal
  }
  function Output-CaretBounds {
    Require-Terminal
    $range = $pattern.DocumentRange
    if ($null -eq $range) { return $null }
    $script:resizeOutputSeen = ($range.GetText(16384) -split '\r?\n') -ccontains 'SP_RESIZE_7'
    if (-not $script:resizeOutputSeen) { return $null }
    # Pinned AccessKit does not implement FindText; use its proven caret-range contract.
    $selection = $pattern.GetSelection()
    if ($null -eq $selection -or $selection.Length -eq 0) { return $null }
    if ($selection.Length -ne 1) { throw 'Native resize terminal has no unique caret range' }
    $start = [System.Windows.Automation.Text.TextPatternRangeEndpoint]::Start
    $end = [System.Windows.Automation.Text.TextPatternRangeEndpoint]::End
    if ($selection[0].CompareEndpoints($start, $selection[0], $end) -ne 0 -or $selection[0].GetText(1) -cne '') {
      throw 'Native resize terminal reports selected text instead of its caret'
    }
    $line = $selection[0].Clone()
    $line.ExpandToEnclosingUnit([System.Windows.Automation.Text.TextUnit]::Line)
    $rectangles = $line.GetBoundingRectangles()
    if ($rectangles.Length -eq 0) { return $null }
    if ($rectangles.Length -ne 1) { throw 'The native resize caret line has no unique rectangle' }
    $rect = $rectangles[0]
    $outer = $document.Current.BoundingRectangle
    foreach ($property in @('Left','Top','Width','Height')) {
      if ([Math]::Abs($outer.$property - $documentBounds.$property) -gt 1) { throw 'The terminal viewport moved during pane resizing' }
    }
    foreach ($number in @($rect.Left, $rect.Top, $rect.Width, $rect.Height)) {
      if ([double]::IsNaN($number) -or [double]::IsInfinity($number)) { throw 'Resize text geometry is not finite' }
    }
    if ($rect.Width -le 0 -or $rect.Height -le 0 -or
        $rect.Left -lt $outer.Left - 1 -or $rect.Top -lt $outer.Top - 1 -or
        $rect.Right -gt $outer.Right + 1 -or $rect.Bottom -gt $outer.Bottom + 1) {
      throw 'Resize text geometry escapes the private terminal'
    }
    return $rect
  }
  function Wait-Resize([string]$stage, $accept) {
    $deadline = [DateTime]::UtcNow.AddSeconds(5)
    $script:resizeOutputSeen = $false
    $last = 'none'
    do {
      $rect = Output-CaretBounds
      if ($null -ne $rect) { $last = "left=$($rect.Left),width=$($rect.Width)" }
      if ($null -ne $rect -and (& $accept $rect)) {
        [Console]::WriteLine("TERMINAL_RESIZE_STAGE stage=$stage output=true $last")
        return $rect
      }
      if ([DateTime]::UtcNow -ge $deadline) {
        throw "Native resize geometry did not reach its expected boundary: stage=$stage output=$script:resizeOutputSeen $last"
      }
      Start-Sleep -Milliseconds 25
    } while ($true)
  }
  Press-Resize 'echo SP_RESIZE_7{ENTER}'
  $before = Wait-Resize 'authored_output' { param($rect) $true }
  Press-Resize '^br'
  Press-Resize '{LEFT}'
  $changed = Wait-Resize 'left' { param($rect) $rect.Left -lt $before.Left - 1 }
  Press-Resize '{RIGHT}'
  $restored = Wait-Resize 'right' { param($rect) [Math]::Abs($rect.Left - $before.Left) -le 1 }
  Press-Resize '{ESC}'
  Require-Released
  Require-Terminal
  [Console]::WriteLine('TERMINAL_RESIZE native=true prefix=true left=true right=true restored=true released=true')
  return
}
$observer = [SpaceportLiveEvents]::new($handle, $ProcessId, [System.Windows.Automation.TextPattern]::TextChangedEvent)
try {
  Require-Terminal
  Require-Released
  $marker = 'SP_A11Y_7'
  if ($Action -eq 'Observe') {
    $marker = 'SP_A11Y_OBS_7'
    [IO.File]::WriteAllText("$PSScriptRoot\core-observe.ready", 'registered')
  } else {
    [System.Windows.Forms.SendKeys]::SendWait('echo SP_A11Y_7{ENTER}')
  }
  $changed = $false
  $deadline = [DateTime]::UtcNow.AddSeconds(5)
  do {
    Require-Terminal
    if ($observer.Drain() -ccontains 'Terminal') { $changed = $true }
    # Resolve a fresh range after each native text-change event.
    $text = $pattern.DocumentRange.GetText(16384)
    $output = ($text -split '\r?\n') -ccontains $marker
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
  # A collapsed range can validly have no rectangles; measure its enclosing visible line.
  $line = $selection[0].Clone()
  $line.ExpandToEnclosingUnit([System.Windows.Automation.Text.TextUnit]::Line)
  $rectangles = $line.GetBoundingRectangles()
  if ($rectangles.Length -eq 0) { throw 'Native caret line has no visible bounding rectangle' }
  $bounds = $document.Current.BoundingRectangle
  foreach ($rectangle in $rectangles) {
    foreach ($number in @($rectangle.X, $rectangle.Y, $rectangle.Width, $rectangle.Height)) {
      if ([double]::IsNaN($number) -or [double]::IsInfinity($number)) { throw 'Native caret-line bounds are not finite' }
    }
    if ($rectangle.Width -lt 0 -or $rectangle.Height -le 0 -or
        $rectangle.Left -lt $bounds.Left - 1 -or $rectangle.Top -lt $bounds.Top - 1 -or
        $rectangle.Right -gt $bounds.Right + 1 -or $rectangle.Bottom -gt $bounds.Bottom + 1) {
      throw 'Native caret-line bounds escape the terminal document'
    }
  }
  [Console]::WriteLine('TERMINAL_A11Y text=true native_text_changed=true unselected_caret=true caret_line_bounds=true')
} finally { $observer.Dispose() }
[Console]::WriteLine('TERMINAL_A11Y observer_removed=true')

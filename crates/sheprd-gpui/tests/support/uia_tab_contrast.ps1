# ABOUTME: Captures the actual selected tab before and after native keyboard focus.
# ABOUTME: Refuses foreign focus, changed geometry, unpainted image backdrops, and unbounded output.
param([int]$ProcessId, [switch]$ImageBackdrop)
. "$PSScriptRoot\uia_tree.ps1" -ProcessId $ProcessId
Add-Type -AssemblyName System.Drawing
Add-Type -Path "$PSScriptRoot\native_tab_capture.cs" -ReferencedAssemblies System.Drawing
if ($windows.Count -ne 1) { throw 'Tab contrast requires one private window' }
$root = $windows[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero -or $root.Current.ProcessId -ne $ProcessId) { throw 'Tab contrast has no exact owned HWND' }
$refused = $false
try {
  $unexpected = [NativeTabCapture]::Capture([IntPtr]::Zero,0,0,1,1)
  $unexpected.Dispose()
} catch {
  if (-not $_.Exception.ToString().Contains('Tab capture lost its exact foreground window')) { throw }
  $refused = $true
}
if (-not $refused) { throw 'Tab capture accepted a foreign window' }
[Console]::WriteLine('TAB_RING_REFUSAL foreign_window=true')
function Find-One($parent, $type, [string]$name) {
  $condition = [System.Windows.Automation.AndCondition]::new(
    [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, $type),
    [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, $name)
  )
  $found = $parent.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
  if ($found.Count -ne 1) { throw "Tab contrast has no unique owned $name" }
  return $found[0]
}
$terminal = Find-One $root ([System.Windows.Automation.ControlType]::Document) 'Terminal'
$strip = Find-One $root ([System.Windows.Automation.ControlType]::Tab) 'Tabs'
$tab = Find-One $strip ([System.Windows.Automation.ControlType]::TabItem) '1'
function Require-Released {
  foreach ($key in @(0x01,0x02,0x04,0x10,0x11,0x12,0x5b,0x5c)) {
    if (([HerdrUia]::GetAsyncKeyState($key) -band 0x8000) -ne 0) { throw 'Tab contrast found a held button or modifier' }
  }
}
function Require-Owned($focus) {
  Require-Released
  if ([HerdrUia]::GetForegroundWindow() -ne $handle) { throw 'Tab contrast lost exact foreground HWND' }
  $element = $automation::FocusedElement
  if (-not [System.Windows.Automation.Automation]::Compare($element, $focus)) { throw 'Tab contrast lost exact native focus' }
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Tab contrast focus left its owned process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Tab contrast focus left its owned window'
}
function Tab-Area {
  $bounds = $tab.Current.BoundingRectangle
  $outer = $root.Current.BoundingRectangle
  foreach ($number in @($bounds.Left,$bounds.Top,$bounds.Width,$bounds.Height)) {
    if ([double]::IsNaN($number) -or [double]::IsInfinity($number)) { throw 'Tab contrast has nonfinite geometry' }
  }
  if ($bounds.Width -lt 20 -or $bounds.Height -lt 10 -or $bounds.Width -gt 600 -or $bounds.Height -gt 150 -or
      $bounds.Left -lt $outer.Left -or $bounds.Top -lt $outer.Top -or
      $bounds.Right -gt $outer.Right -or $bounds.Bottom -gt $outer.Bottom) { throw 'Selected tab is clipped or outside its bounded private window' }
  if (-not $tab.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Current.IsSelected) { throw 'Native tab is no longer selected' }
  return [System.Drawing.Rectangle]::FromLTRB(
    [int][Math]::Ceiling($bounds.Left), [int][Math]::Ceiling($bounds.Top),
    [int][Math]::Floor($bounds.Right), [int][Math]::Floor($bounds.Bottom)
  )
}
function Capture-Tab($focus) {
  Require-Owned $focus
  $area = Tab-Area
  if ($area -ne $script:area) { throw 'Selected tab geometry changed during focus capture' }
  $bitmap = [NativeTabCapture]::Capture($handle,$area.X,$area.Y,$area.Width,$area.Height)
  try { Require-Owned $focus } catch { $bitmap.Dispose(); throw }
  return $bitmap
}
Require-Released
try { [HerdrUia]::keybd_event(0x12,0,0,[UIntPtr]::Zero) }
finally { [HerdrUia]::keybd_event(0x12,0,2,[UIntPtr]::Zero) }
[HerdrUia]::SetForegroundWindow($handle) | Out-Null
$terminal.SetFocus()
$deadline = [DateTime]::UtcNow.AddSeconds(3)
while ([HerdrUia]::GetForegroundWindow() -ne $handle -or -not $terminal.Current.HasKeyboardFocus) {
  if ([DateTime]::UtcNow -ge $deadline) { throw 'Private terminal did not acknowledge native focus' }
  Start-Sleep -Milliseconds 10
}
Require-Owned $terminal
[System.Windows.Forms.SendKeys]::SendWait('{ESC}')
Require-Owned $terminal
if ($ImageBackdrop) {
  $deadline = [DateTime]::UtcNow.AddSeconds(5)
  do {
    Require-Owned $terminal
    $bounds = $terminal.Current.BoundingRectangle
    $sample = [NativeTabCapture]::Capture($handle,[int]($bounds.Left+$bounds.Width/4),[int]($bounds.Top+$bounds.Height*0.75),[int]($bounds.Width/2),1)
    try {
      $a = $sample.GetPixel(0,0)
      $b = $sample.GetPixel($sample.Width-1,0)
      $visible = [Math]::Abs([int]$a.R-[int]$b.R) -gt 20 -and [Math]::Abs([int]$a.G-[int]$b.G) -gt 20 -and [Math]::Abs([int]$a.B-[int]$b.B) -gt 20
    } finally { $sample.Dispose() }
    if ($visible) { break }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Private authored image backdrop has not painted' }
    Start-Sleep -Milliseconds 25
  } while ($true)
  [Console]::WriteLine('TAB_RING_BACKDROP image=true painted=true')
}
$script:area = Tab-Area
$before = Capture-Tab $terminal
$after = $null
try {
  [System.Windows.Forms.SendKeys]::SendWait('^b{F6}')
  $deadline = [DateTime]::UtcNow.AddSeconds(3)
  while (-not $tab.Current.IsKeyboardFocusable) {
    if ([HerdrUia]::GetForegroundWindow() -ne $handle) { throw 'Tab region entry lost foreground HWND' }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Selected tab did not become a native keyboard stop' }
    Start-Sleep -Milliseconds 10
  }
  $tab.SetFocus()
  while (-not $tab.Current.HasKeyboardFocus) {
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Selected tab did not acknowledge native focus' }
    Start-Sleep -Milliseconds 10
  }
  $deadline = [DateTime]::UtcNow.AddSeconds(5)
  do {
    $candidate = Capture-Tab $tab
    try {
      if ([NativeTabCapture]::ChangedBorder($before,$candidate) -ge 20) {
        $after = $candidate
        $candidate = $null
        break
      }
    } finally { if ($null -ne $candidate) { $candidate.Dispose() } }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Native tab focus produced no painted border change' }
    Start-Sleep -Milliseconds 25
  } while ($true)
  [Console]::WriteLine("TAB_RING_BOUNDS x=$($area.X) y=$($area.Y) width=$($area.Width) height=$($area.Height)")
  [Console]::WriteLine('TAB_RING_PNG unfocused '+[NativeTabCapture]::Png($before))
  [Console]::WriteLine('TAB_RING_PNG focused '+[NativeTabCapture]::Png($after))
  Require-Owned $tab
  [Console]::WriteLine('TAB_RING_CAPTURE native=true selected=true focused=true stable_bounds=true released=true')
} finally {
  $before.Dispose()
  if ($null -ne $after) { $after.Dispose() }
}

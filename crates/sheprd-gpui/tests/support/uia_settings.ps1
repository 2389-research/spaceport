# ABOUTME: Drives the separate private Settings window through actual Windows keyboard input.
# ABOUTME: Guards its exact HWND and focus ancestry before every key and preserves private preferences.
param([int]$ProcessId, [ValidateSet('OpenAndRing','ToggleSpace','ToggleEnter','Bubbles','Timeline','Dropdown','Scale')][string]$Action)
. "$PSScriptRoot\uia_tree.ps1" -ProcessId $ProcessId

function Named-Control($root, [string]$name, $type) {
  $named = [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, $name)
  $typed = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, $type)
  $matches = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants,
    [System.Windows.Automation.AndCondition]::new($named, $typed))
  if ($matches.Count -ne 1) { throw "Expected one private Settings control '$name', found $($matches.Count)" }
  return $matches[0]
}

if ($Action -in @('OpenAndRing','Scale')) {
  if ($windows.Count -ne 1) { throw 'Settings opener requires exactly one private main window' }
  $opener = Named-Control $windows[0] 'Settings' ([System.Windows.Automation.ControlType]::Button)
  $opener.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}
$deadline = [DateTime]::UtcNow.AddSeconds(10)
do {
  $owned = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $ProcessId)
  $roots = $automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $owned)
  $settings = @(foreach ($candidate in $roots) {
    if ($candidate.Current.ControlType -eq [System.Windows.Automation.ControlType]::Window -and
        $candidate.Current.Name -ceq 'Settings') { $candidate }
  })
  if ($settings.Count -gt 1) { throw 'The private process created duplicate Settings windows' }
  if ($settings.Count -eq 1) { break }
  if ([DateTime]::UtcNow -ge $deadline) { throw 'The private Settings window did not appear' }
  Start-Sleep -Milliseconds 25
} while ($true)
$root = $settings[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Private Settings has no HWND' }

function Require-SettingsFocus {
  if ([HerdrUia]::GetForegroundWindow() -ne $handle) { throw 'Settings lost its exact foreground HWND' }
  $element = $automation::FocusedElement
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Settings focus left the owned process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Keyboard focus left the owned Settings window'
}

function Same-Focus($target) {
  Require-SettingsFocus
  return [System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $target)
}

function Press([string]$key) {
  Require-SettingsFocus
  foreach ($modifier in @(0x10, 0x11, 0x12)) {
    if (([HerdrUia]::GetAsyncKeyState($modifier) -band 0x8000) -ne 0) { throw 'A modifier was already held before a Settings key' }
  }
  [System.Windows.Forms.SendKeys]::SendWait($key)
  Start-Sleep -Milliseconds 100
  Require-SettingsFocus
}

function Tab-To([string]$name, $type, [string]$key = '{TAB}') {
  for ($index = 0; $index -lt 128; $index++) {
    Require-SettingsFocus
    $current = $automation::FocusedElement.Current
    if ($current.Name -ceq $name -and $current.ControlType -eq $type) {
      $target = Named-Control $root $name $type
      if (-not (Same-Focus $target)) { throw 'Settings target lost focus before activation' }
      return $target
    }
    Press $key
  }
  throw "Tab could not reach private Settings control '$name'"
}

function Wait-Ring([string]$marker) {
  $deadline = [DateTime]::UtcNow.AddSeconds(10)
  $scaleKeyboard = @{}
  do {
    Require-SettingsFocus
    $file = Join-Path $PSScriptRoot 'gui.log'
    # The GUI keeps its inherited writer open; permit that writer while reading a bounded snapshot.
    $stream = [System.IO.File]::Open($file, [System.IO.FileMode]::Open,
      [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
    try {
      $bytes = [byte[]]::new(1048577)
      $length = 0
      while ($length -lt $bytes.Length) {
        $read = $stream.Read($bytes, $length, $bytes.Length - $length)
        if ($read -eq 0) { break }
        $length += $read
      }
      if ($length -gt 1048576) { throw 'Private Settings log exceeds bound' }
      $text = [System.Text.Encoding]::UTF8.GetString($bytes, 0, $length)
    } finally { $stream.Dispose() }
    if ($text.Contains('SETTINGS_RING_ERROR ')) { throw 'The owned Settings ring observer failed' }
    if ($text.Contains('SETTINGS_SCALE_ERROR ')) { throw 'The owned Settings scale observer failed' }
    if ($Action -eq 'Scale') {
      foreach ($name in @('wide', 'narrow')) {
        if (-not $scaleKeyboard.ContainsKey($name) -and $text.Contains("SETTINGS_SCALE_KEYBOARD $name")) {
          $target = Named-Control $root 'Smooth scrolling' ([System.Windows.Automation.ControlType]::Button)
          if (-not (Same-Focus $target)) { throw 'Resize lost the exact native Settings row' }
          Press '{TAB}'
          Press '+{TAB}'
          if (-not (Same-Focus $target)) { throw 'Native resize keys did not restore the same Settings row' }
          $scaleKeyboard[$name] = $true
          [Console]::WriteLine("SETTINGS_SCALE_FOCUS_NATIVE $name")
        }
      }
    }
    if ($text.Contains($marker)) { return }
    if ([DateTime]::UtcNow -ge $deadline) { throw "Missing native Settings marker '$marker'" }
    Start-Sleep -Milliseconds 25
  } while ($true)
}

if ($Action -in @('OpenAndRing','Scale')) {
  if (([HerdrUia]::GetAsyncKeyState(0x12) -band 0x8000) -ne 0) { throw 'Alt was already held' }
  try { [HerdrUia]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero) }
  finally { [HerdrUia]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero) }
  [HerdrUia]::SetForegroundWindow($handle) | Out-Null
  Start-Sleep -Milliseconds 300
  $search = Named-Control $root 'Search settings' ([System.Windows.Automation.ControlType]::Edit)
  $search.SetFocus()
  # GPUI queues UIA focus requests to its foreground executor; the provider call does not await focus.
  $anchorDeadline = [DateTime]::UtcNow.AddSeconds(3)
  while (-not (Same-Focus $search)) {
    if ([DateTime]::UtcNow -ge $anchorDeadline) {
      throw 'Settings search focus was not acknowledged before its deadline'
    }
    Start-Sleep -Milliseconds 25
  }
  Press '{TAB}'
  $general = Named-Control $root 'General' ([System.Windows.Automation.ControlType]::TabItem)
  if (-not (Same-Focus $general)) { throw 'Tab did not reach General from search' }
  Press '+{TAB}'
  if (-not (Same-Focus $search)) { throw 'Shift-Tab did not return to the same search field' }
  Press '{TAB}'
  if (-not (Same-Focus $general)) { throw 'Tab did not restore the same General control' }
  Press ' '
  $switch = Tab-To 'Smooth scrolling' ([System.Windows.Automation.ControlType]::Button)
  if ($Action -eq 'Scale') {
    Wait-Ring 'SETTINGS_SCALE_PASS ui=24 wide=1100x780 narrow=680x780'
    Require-SettingsFocus
    [Console]::WriteLine('SETTINGS_KEY action=Scale native=true')
    return
  }
  Wait-Ring 'SETTINGS_RING_READY focused'
  Press '+{TAB}'
  if (Same-Focus $switch) { throw 'Shift-Tab did not leave the switch' }
  Wait-Ring 'SETTINGS_RING_PASS '
  Press '{TAB}'
  if (-not (Same-Focus $switch)) { throw 'Tab did not return to the same switch' }
} elseif ($Action -eq 'ToggleSpace' -or $Action -eq 'ToggleEnter') {
  $switch = Tab-To 'Smooth scrolling' ([System.Windows.Automation.ControlType]::Button)
  $toggle = $switch.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
  $before = $toggle.Current.ToggleState
  $key = if ($Action -eq 'ToggleSpace') { ' ' } else { '{ENTER}' }
  Press $key
  # SendWait does not await another process's message handling on the SendInput path.
  $toggleDeadline = [DateTime]::UtcNow.AddSeconds(3)
  while ($toggle.Current.ToggleState -eq $before) {
    if (-not (Same-Focus $switch)) { throw 'Settings switch lost focus while awaiting its state' }
    if ([DateTime]::UtcNow -ge $toggleDeadline) {
      throw 'Native Settings key did not change the switch state before its deadline'
    }
    Start-Sleep -Milliseconds 25
  }
} elseif ($Action -eq 'Bubbles' -or $Action -eq 'Timeline') {
  $traverse = if ($Action -eq 'Timeline') { '+{TAB}' } else { '{TAB}' }
  $segment = Tab-To $Action ([System.Windows.Automation.ControlType]::RadioButton) $traverse
  $selection = $segment.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern)
  if ($selection.Current.IsSelected) { throw 'Native Settings segment was already selected' }
  $key = if ($Action -eq 'Bubbles') { ' ' } else { '{ENTER}' }
  Press $key
  # SendWait does not await this process's UIA selection update.
  $selectionDeadline = [DateTime]::UtcNow.AddSeconds(3)
  while (-not $selection.Current.IsSelected) {
    if (-not (Same-Focus $segment)) { throw 'Settings segment lost focus while awaiting its state' }
    if ([DateTime]::UtcNow -ge $selectionDeadline) {
      throw 'Native Settings key did not select the segment before its deadline'
    }
    Start-Sleep -Milliseconds 25
  }
} elseif ($Action -eq 'Dropdown') {
  $trigger = Tab-To 'English' ([System.Windows.Automation.ControlType]::Button) '+{TAB}'
  $expanded = $trigger.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern)
  Press '{ENTER}'
  if ($expanded.Current.ExpandCollapseState -ne [System.Windows.Automation.ExpandCollapseState]::Expanded) { throw 'Native Enter did not open the language dropdown' }
  Press '{DOWN}'
  Press '{UP}'
  Press '{ESC}'
  if (-not (Same-Focus $trigger) -or $expanded.Current.ExpandCollapseState -ne [System.Windows.Automation.ExpandCollapseState]::Collapsed) { throw 'Dropdown dismissal lost its exact trigger' }
  Press ' '
  if ($expanded.Current.ExpandCollapseState -ne [System.Windows.Automation.ExpandCollapseState]::Expanded) { throw 'Native Space did not reopen the language dropdown' }
  Press '{ENTER}'
  if (-not (Same-Focus $trigger) -or $expanded.Current.ExpandCollapseState -ne [System.Windows.Automation.ExpandCollapseState]::Collapsed) { throw 'Dropdown selection lost its exact trigger' }
}
[Console]::WriteLine("SETTINGS_KEY action=$Action native=true")

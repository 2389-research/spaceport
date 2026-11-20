# ABOUTME: Sends fixed scan-code cases to one owned native Windows window.
# ABOUTME: Observes real UIA effects and preserves the existing scoped layout cleanup.
param([int]$ProcessId, [string]$CaseId, [long]$DeadlineEpochMs,
      [string]$FontMarker = '', [double]$ExpectedWidth = 0, [double]$MinimumBefore = 0,
      [string]$PrepareInput = '', [switch]$Measure, [switch]$WaitPrompt,
      [switch]$ObserveState, [switch]$ValidateContract, [switch]$Diagnostic)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type -Path @("$PSScriptRoot\native_profile_id.cs", "$PSScriptRoot\native_altgr.cs")
$tablePath = "$PSScriptRoot\windows_shortcut_cases.json"
if ((Get-Item -LiteralPath $tablePath).Length -gt 32768) { throw 'Native case table exceeds bound' }
# Windows PowerShell emits the JSON array as one pipeline object; assignment preserves its actual elements.
$table = Get-Content -LiteralPath $tablePath -Raw | ConvertFrom-Json
function Native-Case([string]$id) {
  $matching = @($table | Where-Object { $_.id -is [string] -and $_.id -ceq $id })
  if ($matching.Count -ne 1) { throw 'Native input requires one fixed case ID' }
  $selected = $matching[0]
  if ($selected.id -isnot [string] -or $selected.id -cne $id) { throw 'Native input selected the wrong case' }
  $chords = [System.Collections.Generic.List[int[]]]::new()
  foreach ($entry in $selected.chords) {
    $codes = [int[]]@($entry)
    if ($codes.Count -lt 1 -or $codes.Count -gt 4 -or @($codes | Select-Object -Unique).Count -ne $codes.Count -or
        @($codes | Where-Object { $_ -lt 1 -or $_ -gt 255 }).Count -ne 0) { throw 'Native chord violates physical-key bounds' }
    $chords.Add($codes)
  }
  if ($chords.Count -lt 1 -or $chords.Count -gt 4) { throw 'Native case violates chord-count bounds' }
  return @{ value=$selected; chords=$chords }
}
$native = Native-Case $CaseId
$case = $native.value
$nativeChords = $native.chords
if ($ValidateContract) {
  $ids = [System.Collections.Generic.HashSet[string]]::new()
  foreach ($row in $table) {
    if (-not $ids.Add($row.id)) { throw 'Native case IDs are not unique' }
    $selected = Native-Case $row.id
    if ($selected.value.id -cne $row.id) { throw 'Native consumer selected the wrong contract case' }
  }
  $rejected = $false
  try { $null = Native-Case '__unknown_native_case__' } catch { $rejected = $true }
  if (-not $rejected) { throw 'Native consumer accepted an unknown case' }
  $tokens = $null
  $parseErrors = $null
  [System.Management.Automation.Language.Parser]::ParseFile("$PSScriptRoot\console_shortcuts.ps1", [ref]$tokens, [ref]$parseErrors) | Out-Null
  if ($parseErrors.Count -ne 0) { throw ($parseErrors | Out-String) }
  & "$PSScriptRoot\console_shortcuts.ps1" -ValidateContract
  [Console]::WriteLine("WINDOWS_KEYS PowerShell_contract=true cases=$($ids.Count) unknown_case_rejected=true console_syntax=true input=false")
  exit 0
}
[Console]::WriteLine("WINDOWS_KEYS stage=case_loaded id=$CaseId chords=$($nativeChords.Count)")
$automation = [System.Windows.Automation.AutomationElement]
$tree = [System.Windows.Automation.TreeScope]::Descendants
$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
$owned = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $ProcessId)
function Windows { return $automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $owned) }
if ($Diagnostic) {
  $privateWindows = @(Windows)
  [Console]::WriteLine("WINDOWS_KEYS diagnostic_windows=$($privateWindows.Count)")
  foreach ($privateWindow in $privateWindows | Select-Object -First 4) {
    $elements = $privateWindow.FindAll($tree, [System.Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $elements | Select-Object -First 64) {
      $name = $element.Current.Name
      if ($name.Length -gt 160) { $name = $name.Substring(0,160) }
      [Console]::WriteLine("WINDOWS_KEYS diagnostic_node=$($element.Current.ControlType.ProgrammaticName) name=$name focused=$($element.Current.HasKeyboardFocus)")
      if ($element.Current.ControlType -eq [System.Windows.Automation.ControlType]::Document -and $element.Current.Name -ceq 'Terminal') {
        $text = $element.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern).DocumentRange.GetText(8192)
        [Console]::WriteLine("WINDOWS_KEYS private_terminal_begin`n$text`nWINDOWS_KEYS private_terminal_end")
      }
    }
  }
  exit 0
}
$deadline = [DateTimeOffset]::FromUnixTimeMilliseconds($DeadlineEpochMs - 5000).UtcDateTime
do {
  $windows = @(Windows)
  if ($case.outcome.kind -ne 'cycle_windows' -or ($windows.Count -eq 2 -and
      @($windows | Where-Object { $_.Current.NativeWindowHandle -eq 0 }).Count -eq 0)) { break }
  if ([DateTime]::UtcNow -ge $deadline) { throw 'The second exact owned HWND did not map' }
  Start-Sleep -Milliseconds 20
} while ($true)
if ($windows.Count -eq 1) { $root = $windows[0] }
elseif ($case.outcome.kind -eq 'cycle_windows' -and $windows.Count -eq 2) {
  $active = @($windows | Where-Object { [IntPtr]$_.Current.NativeWindowHandle -eq [SpaceportAltGr]::ForegroundWindow() })
  if ($active.Count -ne 1) { throw 'Cycle requires one exact owned foreground window' }
  $root = $active[0]
} else { throw 'Native case has an unexpected owned window count' }
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Native case has no HWND' }
function Wait-For([string]$label, $accept) {
  $phaseDeadline = [DateTime]::UtcNow.AddSeconds(12)
  if ($phaseDeadline -gt $deadline.AddSeconds(-1)) { $phaseDeadline = $deadline.AddSeconds(-1) }
  [Console]::WriteLine("WINDOWS_KEYS wait=$label")
  do {
    if (& $accept) { [Console]::WriteLine("WINDOWS_KEYS acknowledged=$label"); return }
    if ([DateTime]::UtcNow -ge $phaseDeadline) { throw "Native case timed out: $label" }
    Start-Sleep -Milliseconds 20
  } while ($true)
}
function Named($parent, $type, [string]$name) {
  $condition = [System.Windows.Automation.AndCondition]::new(
    [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, $type),
    [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, $name))
  return $parent.FindAll($tree, $condition)
}
function Require-Root {
  if ([SpaceportAltGr]::ForegroundWindow() -ne $handle) { throw 'Native case lost its exact HWND' }
  $element = $automation::FocusedElement
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Native focus left its process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Native focus left its owned window'
}
function Document {
  $documents = @(Named $root ([System.Windows.Automation.ControlType]::Document) 'Terminal')
  if ($documents.Count -ne 1) { throw 'Native case needs one owned terminal document' }
  return $documents[0]
}
function Wait-Keys([int[]]$keys) {
  $script:lastKeyError = ''
  Wait-For 'released native keys' {
    try { [SpaceportAltGr]::RequireReleased(); [SpaceportAltGr]::RequireKeysReleased($keys); $true }
    catch {
      $message = $_.Exception.Message
      if ($script:lastKeyError -cne $message) { [Console]::WriteLine("WINDOWS_KEYS held=$message"); $script:lastKeyError = $message }
      $false
    }
  }
}
if ($ObserveState) {
  Require-Root
  $document = Document
  if (-not $document.Current.HasKeyboardFocus) { throw 'Pane input moved focus into application chrome' }
  $selected = @((Named $root ([System.Windows.Automation.ControlType]::TabItem) '1') | Where-Object {
    $_.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Current.IsSelected
  })
  if ($selected.Count -ne 1) { throw 'Pane input changed the selected fixture tab' }
  [Console]::WriteLine((@{ hwnd=$handle.ToInt64(); terminal=$true; selected='1'; windows=$windows.Count;
    status=@(Named $root ([System.Windows.Automation.ControlType]::StatusBar) 'Status bar').Count;
    sidebar=@(Named $root ([System.Windows.Automation.ControlType]::Group) 'Sidebar').Count;
    strips=@(Named $root ([System.Windows.Automation.ControlType]::Tab) 'Tabs').Count } | ConvertTo-Json -Compress))
  exit 0
}
function Font-Width {
  if ($FontMarker -notmatch '^SP_FONT_[a-f0-9]{32}$') { throw 'Font marker is not a private nonce' }
  $document = Document
  $pattern = $document.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
  # Pinned AccessKit implements line/character movement, but not FindText.
  $range = $pattern.DocumentRange.Clone()
  $origin = $range.Clone()
  $start = [System.Windows.Automation.Text.TextPatternRangeEndpoint]::Start
  $end = [System.Windows.Automation.Text.TextPatternRangeEndpoint]::End
  $line = [System.Windows.Automation.Text.TextUnit]::Line
  $character = [System.Windows.Automation.Text.TextUnit]::Character
  $range.MoveEndpointByRange($end, $origin, $start)
  $range.ExpandToEnclosingUnit($line)
  $found = $false
  for ($index = 0; $index -lt 512; $index++) {
    if ($range.GetText(4096).TrimEnd() -ceq $FontMarker) { $found = $true; break }
    if ($range.Move($line, 1) -ne 1) { break }
  }
  if (-not $found) { return 0 }
  $origin = $range.Clone()
  $range.MoveEndpointByRange($end, $origin, $start)
  if ($range.MoveEndpointByUnit($end, $character, $FontMarker.Length) -ne $FontMarker.Length -or
      $range.GetText(4096) -cne $FontMarker) { throw 'Native marker range does not cover its exact authored text' }
  $rectangles = $range.GetBoundingRectangles()
  if ($rectangles.Length -ne 1) { return 0 }
  $rect = $rectangles[0]
  $outer = $document.Current.BoundingRectangle
  if ([double]::IsNaN($rect.Width) -or [double]::IsInfinity($rect.Width) -or $rect.Width -le 0 -or
      $rect.Left -lt $outer.Left - 1 -or $rect.Right -gt $outer.Right + 1) {
    throw 'Native font marker has invalid or clipped geometry'
  }
  return $rect.Width
}
if ($WaitPrompt) {
  Wait-For 'real pane shell prompt' {
    $pattern = (Document).GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
    $pattern.DocumentRange.GetText(-1).TrimEnd().EndsWith('>')
  }
  [Console]::WriteLine('WINDOWS_KEYS shell_prompt=true input=false')
  if ($PrepareInput -eq '') { exit 0 }
}
if ($Measure) {
  $script:width = 0
  Wait-For 'font marker measurement' { $script:width = Font-Width; $script:width -gt 0 }
  [Console]::WriteLine((@{ id=$CaseId; width=$script:width; input=$false } | ConvertTo-Json -Compress))
  exit 0
}
[SpaceportAltGr]::SetForegroundWindow($handle) | Out-Null
$document = Document
$document.SetFocus()
Wait-For 'terminal focus before native input' { $document.Current.HasKeyboardFocus }
Require-Root
if ($PrepareInput -ne '') {
  if ($PrepareInput.Length -gt 4096 -or $PrepareInput.Contains("`r") -or $PrepareInput.Contains("`n")) {
    throw 'Pane preparation requires one bounded trusted command'
  }
  $preparation = $null
  try {
    $preparation = [SpaceportAltGr]::Begin($handle, [uint32]$ProcessId, $DeadlineEpochMs)
    $preparation.TypePreparation($PrepareInput)
    Wait-For 'prepared command echo through the actual GUI input route' {
      Require-Root
      $pattern = (Document).GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
      $pattern.DocumentRange.GetText(32768).Replace("`r", '').Replace("`n", '').Contains($PrepareInput)
    }
    Require-Root
    $preparation.Chord([int[]]@(13))
  } finally {
    if ($null -ne $preparation) { $preparation.Dispose() }
  }
  [Console]::WriteLine('WINDOWS_KEYS pane_preparation=true actual_gui_input=true')
  exit 0
}
$beforeHandles = @($windows | ForEach-Object { [IntPtr]$_.Current.NativeWindowHandle })
$stripsBefore = @(Named $root ([System.Windows.Automation.ControlType]::Tab) 'Tabs').Count
$paletteName = 'Search workspaces, commands, agents, and projects...'
if (@(Named $root ([System.Windows.Automation.ControlType]::Edit) $paletteName).Count -ne 0 -or
    @(Named $root ([System.Windows.Automation.ControlType]::Edit) 'Search shortcuts...').Count -ne 0) {
  throw 'The native case began with an existing popup'
}
if ($case.outcome.kind -in @('zen','status_bar','sidebar','pane_key')) {
  if (@(Named $root ([System.Windows.Automation.ControlType]::StatusBar) 'Status bar').Count -ne 1 -or
      @(Named $root ([System.Windows.Automation.ControlType]::Group) 'Sidebar').Count -ne 1) {
    throw 'The native chrome case requires the normal visible chrome'
  }
}
$script:fontBefore = 0
if ($case.outcome.kind -in @('font_larger','font_smaller','font_reset')) {
  Wait-For 'acknowledged prepared font geometry' { $script:fontBefore = Font-Width; $script:fontBefore -gt $MinimumBefore + 0.1 }
}
$scope = $null
$passed = $false
try {
  $scope = [SpaceportAltGr]::Begin($handle, [uint32]$ProcessId, $DeadlineEpochMs)
  [Console]::WriteLine("WINDOWS_KEYS stage=layout_ready id=$CaseId")
  if ($case.outcome.kind -eq 'resume') {
    $scope.Chord([int[]]@(162,160,80))
    Wait-For 'palette preparation' { @(Named $root ([System.Windows.Automation.ControlType]::Edit) $paletteName).Count -eq 1 }
    $field = @(Named $root ([System.Windows.Automation.ControlType]::Edit) $paletteName)[0]
    $value = $field.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
    $value.SetValue('font')
    Wait-For 'saved palette query' { $value.Current.Value -ceq 'font' }
    Wait-Keys ([int[]]@(162,160,80))
    $scope.Chord([int[]]@(27))
    Wait-For 'closed palette before resume' { @(Named $root ([System.Windows.Automation.ControlType]::Edit) $paletteName).Count -eq 0 }
  }
  foreach ($chord in $nativeChords) {
    Require-Root
    [Console]::WriteLine("WINDOWS_KEYS stage=chord_begin id=$CaseId keys=$($chord -join ',')")
    Wait-Keys ([int[]]@($chord))
    $scope.Chord([int[]]@($chord))
    [Console]::WriteLine("WINDOWS_KEYS stage=chord_sent id=$CaseId")
  }
  switch ($case.outcome.kind) {
    'keybinds' {
      Wait-For 'native keybindings popup' {
        $fields = @(Named $root ([System.Windows.Automation.ControlType]::Edit) 'Search shortcuts...')
        $fields.Count -eq 1 -and $fields[0].Current.HasKeyboardFocus
      }
    }
    'settings' {
      Wait-For 'native Settings window' {
        $added = @((Windows) | Where-Object { [IntPtr]$_.Current.NativeWindowHandle -notin $beforeHandles })
        $added.Count -eq 1 -and $added[0].Current.Name -ceq 'Settings' -and
          [IntPtr]$added[0].Current.NativeWindowHandle -eq [SpaceportAltGr]::ForegroundWindow() -and
          @(Named $added[0] ([System.Windows.Automation.ControlType]::Edit) 'Search settings').Count -eq 1
      }
    }
    'split_editor' {
      Wait-For 'additional native editor group' { @(Named $root ([System.Windows.Automation.ControlType]::Tab) 'Tabs').Count -eq $stripsBefore + 1 }
    }
    'cycle_windows' {
      $other = @($beforeHandles | Where-Object { $_ -ne $handle })
      if ($other.Count -ne 1) { throw 'Cycle has no unique other HWND' }
      Wait-For 'other exact owned window active' { [SpaceportAltGr]::ForegroundWindow() -eq $other[0] }
    }
    'zen' {
      Wait-For 'native Zen chrome departure' {
        @(Named $root ([System.Windows.Automation.ControlType]::StatusBar) 'Status bar').Count -eq 0 -and
        @(Named $root ([System.Windows.Automation.ControlType]::Group) 'Sidebar').Count -eq 0
      }
      Wait-Keys ([int[]]@($case.chords[0]))
      $scope.Chord([int[]]@($case.chords[0]))
      Wait-For 'native Zen chrome return' {
        @(Named $root ([System.Windows.Automation.ControlType]::StatusBar) 'Status bar').Count -eq 1 -and
        @(Named $root ([System.Windows.Automation.ControlType]::Group) 'Sidebar').Count -eq 1
      }
    }
    'status_bar' {
      Wait-For 'native status bar departure' { @(Named $root ([System.Windows.Automation.ControlType]::StatusBar) 'Status bar').Count -eq 0 }
      Wait-Keys ([int[]]@($case.chords[0]))
      $scope.Chord([int[]]@($case.chords[0]))
      Wait-For 'native status bar return' { @(Named $root ([System.Windows.Automation.ControlType]::StatusBar) 'Status bar').Count -eq 1 }
    }
    'sidebar' {
      Wait-For 'native sidebar departure' { @(Named $root ([System.Windows.Automation.ControlType]::Group) 'Sidebar').Count -eq 0 }
      Wait-Keys ([int[]]@($case.chords[0]))
      $scope.Chord([int[]]@($case.chords[0]))
      Wait-For 'native sidebar return' { @(Named $root ([System.Windows.Automation.ControlType]::Group) 'Sidebar').Count -eq 1 }
    }
    'digit' {
      Wait-For 'native numbered tab selection' {
        $tabs = @(Named $root ([System.Windows.Automation.ControlType]::TabItem) ([string]$case.outcome.number))
        $tabs.Count -eq 1 -and $tabs[0].GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Current.IsSelected
      }
    }
    { $_ -in @('palette','resume') } {
      Wait-For 'native palette field' {
        $fields = @(Named $root ([System.Windows.Automation.ControlType]::Edit) $paletteName)
        if ($fields.Count -ne 1 -or -not $fields[0].Current.HasKeyboardFocus) { return $false }
        $value = $fields[0].GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
        return $value.Current.Value -ceq $(if ($case.outcome.kind -eq 'resume') { 'font' } else { '' })
      }
    }
    'font_larger' { Wait-For 'larger native glyph bounds' { (Font-Width) -gt $script:fontBefore + 0.1 } }
    'font_smaller' { Wait-For 'smaller native glyph bounds' { (Font-Width) -lt $script:fontBefore - 0.1 -and (Font-Width) -gt 0 } }
    'font_reset' {
      if ($ExpectedWidth -le 0 -or [Math]::Abs($ExpectedWidth - $script:fontBefore) -lt 0.1) { throw 'Reset has no different prepared font size' }
      Wait-For 'configured native glyph bounds' { [Math]::Abs((Font-Width) - $ExpectedWidth) -le 0.1 }
    }
  }
  if ($case.outcome.kind -in @('keybinds','palette','resume')) {
    Wait-Keys ([int[]]@($case.chords[0]))
    $scope.Chord([int[]]@(27))
    Wait-For 'popup cleanup' {
      @(Named $root ([System.Windows.Automation.ControlType]::Edit) 'Search shortcuts...').Count -eq 0 -and
      @(Named $root ([System.Windows.Automation.ControlType]::Edit) $paletteName).Count -eq 0
    }
  }
  $allKeys = [int[]]@($case.chords | ForEach-Object { $_ } | Select-Object -Unique)
  Wait-Keys $allKeys
  $passed = $true
} finally {
  if ($null -ne $scope) {
    [Console]::WriteLine("WINDOWS_KEYS stage=cleanup_begin id=$CaseId")
    $scope.Dispose()
    [Console]::WriteLine("WINDOWS_KEYS stage=cleanup_complete id=$CaseId")
  }
}
if (-not $passed) { throw 'Native case did not complete' }
if ($case.outcome.kind -notin @('settings','cycle_windows')) {
  $afterHandles = @((Windows) | ForEach-Object { [IntPtr]$_.Current.NativeWindowHandle })
  if ($afterHandles.Count -ne $beforeHandles.Count -or
      @($afterHandles | Where-Object { $_ -notin $beforeHandles }).Count -ne 0) {
    throw 'Native input changed the exact owned HWND set'
  }
}
if ($case.outcome.kind -in @('font_larger','font_smaller','font_reset')) {
  [Console]::WriteLine((@{ id=$CaseId; before=$script:fontBefore; width=(Font-Width); input=$true } | ConvertTo-Json -Compress))
}
if ($case.outcome.kind -eq 'settings') {
  $added = @((Windows) | Where-Object { [IntPtr]$_.Current.NativeWindowHandle -notin $beforeHandles })
  if ($added.Count -ne 1) { throw 'Settings cleanup lost its exact owned window' }
  $added[0].GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).Close()
  Wait-For 'Settings cleanup' { @(Windows).Count -eq $beforeHandles.Count }
}
[Console]::WriteLine("WINDOWS_KEYS native PASS id=$CaseId klid=00000415 keys_released=true layouts_restored=true")

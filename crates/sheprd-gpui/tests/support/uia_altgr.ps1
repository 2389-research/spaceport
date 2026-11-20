# ABOUTME: Verifies actual Polish AltGr text in the owned native workspace search field.
# ABOUTME: Checks focus, shortcut isolation, and scoped layout cleanup on success and intentional failure.
param([int]$ProcessId, [long]$DeadlineEpochMs)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type -Path @("$PSScriptRoot\native_profile_id.cs", "$PSScriptRoot\native_profile_id_tests.cs", "$PSScriptRoot\native_altgr.cs")
[SpaceportProfileIdTests]::Run()
$automation = [System.Windows.Automation.AutomationElement]
$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
$owned = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $ProcessId)
$windows = $automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $owned)
if ($windows.Count -ne 1) { throw 'AltGr fixture requires one private window' }
$root = $windows[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'AltGr fixture has no native HWND' }
$name = [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Search spaces')
$type = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)
$fields = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.AndCondition]::new($name, $type))
if ($fields.Count -ne 1) { throw 'AltGr fixture has no unique native Search spaces field' }
$field = $fields[0]
if ($field.Current.IsOffscreen -or -not $field.Current.IsEnabled) { throw 'Native search field is not visible and enabled' }
$value = $field.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
if ($value.Current.Value -cne '') { throw 'Private search field is not initially empty' }
[SpaceportAltGr]::RequireReleased()
[SpaceportAltGr]::SetForegroundWindow($handle) | Out-Null
$deadline = [DateTime]::UtcNow.AddSeconds(3)
do {
  $field.SetFocus()
  if ($field.Current.HasKeyboardFocus) { break }
  if ([DateTime]::UtcNow -ge $deadline) { throw 'Private search field did not receive focus' }
  Start-Sleep -Milliseconds 10
} while ($true)

function Require-Field {
  if ($root.Current.ProcessId -ne $ProcessId -or $field.Current.IsOffscreen) { throw 'AltGr field left its owned visible tree' }
  $focused = $automation::FocusedElement
  if ($null -eq $focused -or -not [System.Windows.Automation.Automation]::Compare($focused, $field)) { throw 'Native keyboard focus left Search spaces' }
  $element = $focused
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Native focus left the private process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Native focus left the exact private window'
}
function Require-Text([string]$expected) {
  $deadline = [DateTime]::UtcNow.AddSeconds(3)
  do {
    Require-Field
    $text = $value.Current.Value
    if ($text -ceq $expected) { break }
    if ($text -cne '' -and -not $expected.StartsWith($text, [System.StringComparison]::Ordinal)) { throw 'AltGr produced unexpected native text' }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'AltGr translation did not reach the native field' }
    Start-Sleep -Milliseconds 10
  } while ($true)
  $deadline = [DateTime]::UtcNow.AddSeconds(2)
  do {
    try { [SpaceportAltGr]::RequireReleased(); break }
    catch {
      if ([DateTime]::UtcNow -ge $deadline) { throw }
      Start-Sleep -Milliseconds 10
    }
  } while ($true)
}
Require-Field
# Run the same scope through an intentional failure before sending any text.
$scope = $null
$caught = $false
try {
  $scope = [SpaceportAltGr]::Begin($handle, [uint32]$ProcessId, $DeadlineEpochMs)
  throw [System.OperationCanceledException]::new('Intentional native layout cleanup probe')
} catch [System.OperationCanceledException] {
  $caught = $true
} finally {
  if ($null -ne $scope) { $scope.Dispose() }
}
if (-not $caught) { throw 'Native layout failure cleanup was not exercised' }
$scope = $null
try {
  Require-Field
  $scope = [SpaceportAltGr]::Begin($handle, [uint32]$ProcessId, $DeadlineEpochMs)
  Require-Field
  $scope.Type($false)
  Require-Text ([string][char]0x0142)
  Require-Field
  $scope.TypeLetter(0x4E, $false)
  Require-Text (([string][char]0x0142) + [char]0x0144)
  Require-Field
  $scope.TypeLetter(0x5A, $false)
  Require-Text (([string][char]0x0142) + [char]0x0144 + [char]0x017C)
  Require-Field
  $scope.Type($true)
  Require-Text (([string][char]0x0142) + [char]0x0144 + [char]0x017C + [char]0x0143)
  $button = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
  $create = [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Create')
  if ($root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.AndCondition]::new($button, $create)).Count -ne 0) { throw 'AltGr opened the Workspace dialog' }
  if ($automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $owned).Count -ne 1) { throw 'AltGr changed the owned window count' }
} finally {
  if ($null -ne $scope) { $scope.Dispose() }
}
[Console]::WriteLine('ALTGR native PASS: klid=00000415 text=U+0142,U+0144,U+017C,U+0143 workspace_dialog=false keys_released=true layouts_restored=true failure_cleanup=true')

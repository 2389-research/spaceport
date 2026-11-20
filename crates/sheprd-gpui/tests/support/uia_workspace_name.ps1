# ABOUTME: Types one authored private workspace name with native field acknowledgements.
# ABOUTME: Leaves submission to the caller after the complete value is visible through UI Automation.
param([int]$ProcessId, [Parameter(Mandatory=$true)][ValidatePattern('^[a-z0-9-]{1,64}$')][string]$Name)
. "$PSScriptRoot\uia_tree.ps1" -ProcessId $ProcessId
if ($windows.Count -ne 1) { throw 'Workspace name requires exactly one private window' }
$root = $windows[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Workspace name has no owned HWND' }
$create = [System.Windows.Automation.AndCondition]::new(
  [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button),
  [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Create')
)
$edit = [System.Windows.Automation.AndCondition]::new(
  [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit),
  [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, '')
)
function Require-Input {
  if ([HerdrUia]::GetForegroundWindow() -ne $handle) { throw 'Workspace name lost exact foreground HWND' }
  foreach ($key in @(0x10,0x11,0x12,0x5b,0x5c)) {
    if (([HerdrUia]::GetAsyncKeyState($key) -band 0x8000) -ne 0) { throw 'Workspace name found a held modifier' }
  }
  $element = $automation::FocusedElement
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Workspace name focus left its owned process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Workspace name focus left its owned window'
}
function Name-Field {
  Require-Input
  $buttons = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants,$create)
  $fields = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants,$edit)
  if ($buttons.Count -ne 1 -or $fields.Count -ne 1 -or $fields[0].Current.IsOffscreen -or
      -not $fields[0].Current.IsEnabled) { throw 'Workspace creation has no unique visible private name field' }
  return $fields[0].GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
}
function Wait-Value([string]$expected) {
  while ($true) {
    $field = Name-Field
    if ($field.Current.Value -ceq $expected) { return }
    if ([DateTime]::UtcNow -ge $script:deadline) { throw 'Native workspace field did not acknowledge its authored prefix' }
    Start-Sleep -Milliseconds 10
  }
}
foreach ($key in @(0x10,0x11,0x12,0x5b,0x5c)) {
  if (([HerdrUia]::GetAsyncKeyState($key) -band 0x8000) -ne 0) { throw 'Workspace name found an initially held modifier' }
}
try { [HerdrUia]::keybd_event(0x12,0,0,[UIntPtr]::Zero) }
finally { [HerdrUia]::keybd_event(0x12,0,2,[UIntPtr]::Zero) }
[HerdrUia]::SetForegroundWindow($handle) | Out-Null
$script:deadline = [DateTime]::UtcNow.AddSeconds(10)
while ([HerdrUia]::GetForegroundWindow() -ne $handle) {
  if ([DateTime]::UtcNow -ge $script:deadline) { throw 'Workspace name could not obtain its private foreground window' }
  Start-Sleep -Milliseconds 10
}
$field = Name-Field
[System.Windows.Forms.SendKeys]::SendWait('^a{BACKSPACE}')
Wait-Value ''
$prefix = ''
foreach ($character in $Name.ToCharArray()) {
  Require-Input
  [System.Windows.Forms.SendKeys]::SendWait([string]$character)
  $prefix += $character
  Wait-Value $prefix
}
Require-Input
[Console]::WriteLine('WORKSPACE_NAME native=true acknowledged=true submitted=false')

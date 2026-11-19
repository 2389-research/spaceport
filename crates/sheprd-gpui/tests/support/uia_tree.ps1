# ABOUTME: Inspects and acts only on the native fixture's owned UIA windows.
# ABOUTME: Refuses keyboard input unless the exact private window owns foreground focus.
# Prints a process's windows as UI Automation's control view shows them, the
# tree Narrator and NVDA read: one line per element, indented by depth, with
# its control type, name, and the patterns or states that matter here. With
# -Keys, it then brings the window forward, sends each key ("|"-separated,
# SendKeys syntax), and prints the element that has keyboard focus after it.
param([int]$ProcessId, [string]$Keys = '', [string]$Invoke = '')
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
Write-Output 'UIA stage: loading assemblies'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
Write-Output 'UIA stage: compiling native helpers'
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class HerdrUia {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void keybd_event(byte k, byte s, uint f, UIntPtr e);
}
"@
$automation = [System.Windows.Automation.AutomationElement]
$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker

function Describe($element) {
  $current = $element.Current
  $type = $current.ControlType.ProgrammaticName -replace 'ControlType\.', ''
  $notes = @()
  foreach ($pattern in $element.GetSupportedPatterns()) {
    $name = $pattern.ProgrammaticName -replace 'PatternIdentifiers\.Pattern', ''
    switch ($name) {
      'SelectionItem' { $notes += 'selected=' + $element.GetCurrentPattern($pattern).Current.IsSelected }
      'ExpandCollapse' { $notes += '' + $element.GetCurrentPattern($pattern).Current.ExpandCollapseState }
      'Toggle' { $notes += 'toggle=' + $element.GetCurrentPattern($pattern).Current.ToggleState }
      default { $notes += $name }
    }
  }
  if (-not $current.IsEnabled) { $notes += 'disabled' }
  $name = $current.Name -replace '\r?\n', ' / '
  "$type '$name' [$($notes -join ', ')]"
}

function Walk($element, $indent) {
  "$indent$(Describe $element)"
  $child = $walker.GetFirstChild($element)
  while ($null -ne $child) {
    Walk $child ($indent + '  ')
    $child = $walker.GetNextSibling($child)
  }
}

$condition = New-Object System.Windows.Automation.PropertyCondition ($automation::ProcessIdProperty), $ProcessId
Write-Output 'UIA stage: finding private windows'
$windows = $automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $condition)
Write-Output "UIA stage: reading $($windows.Count) private windows"
foreach ($window in $windows) {
  Walk $window ''
}
Write-Output 'UIA stage: tree complete'

if ($Invoke) {
  $name = [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, $Invoke)
  $button = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
  $namedButton = [System.Windows.Automation.AndCondition]::new($name, $button)
  $buttons = @(foreach ($window in $windows) {
    $window.FindAll([System.Windows.Automation.TreeScope]::Descendants, $namedButton)
  })
  if ($buttons.Count -ne 1) { throw "Expected one private button named '$Invoke', found $($buttons.Count)" }
  $buttons[0].GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}

if ($Keys) {
  $handle = (Get-Process -Id $ProcessId).MainWindowHandle
  if ($handle -eq [IntPtr]::Zero) { throw 'The private process has no main window' }
  # Windows lets a background process raise a window right after a key press.
  [HerdrUia]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
  [HerdrUia]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
  [HerdrUia]::SetForegroundWindow($handle) | Out-Null
  Start-Sleep -Milliseconds 500
  foreach ($key in $Keys.Split('|')) {
    if ([HerdrUia]::GetForegroundWindow() -ne $handle -or
        $automation::FocusedElement.Current.ProcessId -ne $ProcessId) {
      throw 'The private window does not own foreground keyboard focus'
    }
    [System.Windows.Forms.SendKeys]::SendWait($key)
    Start-Sleep -Milliseconds 400
    "focus after $key -> $(Describe $automation::FocusedElement)"
  }
}

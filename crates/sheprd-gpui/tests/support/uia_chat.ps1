# ABOUTME: Traverses the private chat's real controls and activates them with native keys.
# ABOUTME: Requires exact-window foreground and ancestry before every key press.
param([int]$ProcessId, [ValidateSet('Settings','Tool','Allow','Reject')][string]$Action)
# The parent verifies the full tree; actions need the same setup and exact-window checks.
. "$PSScriptRoot\uia_tree.ps1" -ProcessId $ProcessId -SkipTree
$fixture = Get-Content "$PSScriptRoot\native-chat-accessibility.json" -Raw | ConvertFrom-Json
if ($windows.Count -ne 1) { throw 'Native chat requires exactly one private window' }
$root = $windows[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Native chat has no HWND' }
function Chat-Stage([string]$stage) {
  [Console]::WriteLine("CHAT_STAGE action=$Action stage=$stage at=$([DateTime]::UtcNow.ToString('O'))")
}
function Require-ChatFocus {
  if ([HerdrUia]::GetForegroundWindow() -ne $handle) { throw 'Native chat lost exact foreground HWND' }
  $element = $automation::FocusedElement
  $depth = 0
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) {
      throw "Native chat focus left its process: expected=$ProcessId actual=$($element.Current.ProcessId) depth=$depth"
    }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
    $depth++
  }
  throw 'Native chat focus left its owned window'
}
$editType = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)
$editName = [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Message the agent')
$edits = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.AndCondition]::new($editType, $editName))
if ($edits.Count -ne 1) { throw 'Native chat has no unique named composer' }
$composer = $edits[0]
Chat-Stage 'read-draft'
if ($composer.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value -cne $fixture.draft) {
  throw 'Native composer draft changed before keyboard input'
}
$buttonType = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
$buttons = @($root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $buttonType))
$targets = @(foreach ($button in $buttons) {
  $name = $button.Current.Name
  if (($Action -eq 'Settings' -and $name -ceq 'Agent settings') -or
      ($Action -eq 'Tool' -and $name.Contains('Native fixture tool')) -or
      ($Action -eq 'Allow' -and $name -ceq $fixture.transcript.permissions[0].options[0].name) -or
      ($Action -eq 'Reject' -and $name -ceq $fixture.transcript.permissions[1].options[0].name)) { $button }
})
if ($targets.Count -ne 1) { throw 'Native chat action has no unique named button' }
$target = $targets[0]
if (-not $target.Current.IsEnabled) { throw 'Native chat action is disabled' }
if (([HerdrUia]::GetAsyncKeyState(0x12) -band 0x8000) -ne 0) { throw 'Alt is already held' }
try { [HerdrUia]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero) }
finally { [HerdrUia]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero) }
[HerdrUia]::SetForegroundWindow($handle) | Out-Null
Start-Sleep -Milliseconds 500
Chat-Stage 'anchor-composer'
$composer.SetFocus()
Start-Sleep -Milliseconds 200
Chat-Stage 'check-composer-focus'
Require-ChatFocus
if (-not [System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $composer)) {
  throw 'Native chat could not anchor focus in its actual composer'
}
$reached = $false
# Tool and permission controls precede the composer; Settings follows it.
$traverse = if ($Action -eq 'Settings') { '{TAB}' } else { '+{TAB}' }
for ($index = 0; $index -lt 32; $index++) {
  Chat-Stage "tab-$index"
  Require-ChatFocus
  if ([System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $target)) { $reached = $true; break }
  [System.Windows.Forms.SendKeys]::SendWait($traverse)
  Start-Sleep -Milliseconds 100
}
if (-not $reached) { throw 'Native Tab could not reach the named chat control' }
Require-ChatFocus
if (-not [System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $target)) {
  throw 'Native chat focus changed before activation'
}
$key = if ($Action -eq 'Reject') { ' ' } else { '{ENTER}' }
Chat-Stage 'activate'
[System.Windows.Forms.SendKeys]::SendWait($key)
[Console]::WriteLine("CHAT_KEY action=$Action native=true")

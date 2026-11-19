# ABOUTME: Delivers a held tab shortcut after observing its first real tab change.
# ABOUTME: Requires the exact private window and releases only keys this fixture pressed.
param([int]$ProcessId, [ValidateSet('New','Close')][string]$Action, [long]$DeadlineEpochMs)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Threading;
public static class HeldTabs {
  static readonly List<ushort> pressed = new List<ushort>();
  static Timer watchdog;
  static bool finished;
  static string releaseError;
  [StructLayout(LayoutKind.Sequential)] public struct Keyboard {
    public ushort key, scan; public uint flags, time; public UIntPtr extra;
  }
  [StructLayout(LayoutKind.Sequential)] public struct Mouse {
    public int x, y; public uint data, flags, time; public UIntPtr extra;
  }
  [StructLayout(LayoutKind.Explicit)] public struct Data {
    [FieldOffset(0)] public Keyboard keyboard;
    [FieldOffset(0)] public Mouse mouse;
  }
  [StructLayout(LayoutKind.Sequential)] public struct Input { public uint type; public Data data; }
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] input, int size);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr handle);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern short GetAsyncKeyState(int key);
  public static void Send(ushort key, bool up) {
    Input input = new Input(); input.type = 1;
    input.data.keyboard.key = key; input.data.keyboard.flags = up ? 2u : 0u;
    if (SendInput(1, new Input[] { input }, Marshal.SizeOf(typeof(Input))) != 1)
      throw new InvalidOperationException("Private native input failed: " + Marshal.GetLastWin32Error());
  }
  public static void Begin(long deadlineEpochMs) {
    long remaining = deadlineEpochMs - DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() - 5000;
    if (remaining < 12000 || remaining > 20000)
      throw new InvalidOperationException("Insufficient parent budget for held input and cleanup");
    watchdog = new Timer(delegate {
      if (Release()) Environment.Exit(73);
    }, null, (int)remaining, Timeout.Infinite);
  }
  public static void Press(ushort key) {
    lock (pressed) {
      if (finished) throw new InvalidOperationException("Held-input fixture already ended");
      Send(key, false);
      if (!pressed.Contains(key)) pressed.Add(key);
    }
  }
  static bool Release() {
    lock (pressed) {
      if (finished) return false;
      finished = true;
      for (int index = pressed.Count - 1; index >= 0; index--) {
        try { Send(pressed[index], true); }
        catch (Exception error) { releaseError = error.Message; }
      }
      if (watchdog != null) watchdog.Dispose();
      return true;
    }
  }
  public static void End() {
    Release();
    if (releaseError != null) throw new InvalidOperationException(releaseError);
  }
}
"@
$automation = [System.Windows.Automation.AutomationElement]
$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
$owned = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $ProcessId)
$windows = $automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $owned)
if ($windows.Count -ne 1) { throw 'Held-tab fixture requires exactly one private window' }
$root = $windows[0]
$stripType = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Tab)
$stripName = [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Tabs')
$strips = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.AndCondition]::new($stripType, $stripName))
if ($strips.Count -ne 1) { throw 'Held-tab fixture requires exactly one terminal tab strip' }
$strip = $strips[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Private window has no native handle' }

function Require-Focus {
  if ([HeldTabs]::GetForegroundWindow() -ne $handle) { throw 'Private HWND lost foreground' }
  $element = $automation::FocusedElement
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Keyboard focus left the private process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Keyboard focus left the exact private window'
}
function Tab-Count {
  $type = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::TabItem)
  return $strip.FindAll([System.Windows.Automation.TreeScope]::Descendants, $type).Count
}

# The existing fixture uses an Alt press to obtain foreground permission.
if (([HeldTabs]::GetAsyncKeyState(0x12) -band 0x8000) -ne 0) { throw 'Alt is already held' }
try { [HeldTabs]::Send(0x12, $false) } finally { [HeldTabs]::Send(0x12, $true) }
[HeldTabs]::SetForegroundWindow($handle) | Out-Null
Start-Sleep -Milliseconds 500
Require-Focus
$key = if ($Action -eq 'New') { 0x54 } else { 0x57 }
$modifiers = if ($Action -eq 'New') { @(0x11,0x10) } else { @(0x11,0x12,0x10) }
foreach ($code in @($modifiers) + @($key)) {
  if (([HeldTabs]::GetAsyncKeyState($code) -band 0x8000) -ne 0) { throw 'A fixture key is already held' }
}
$before = Tab-Count
$expected = if ($Action -eq 'New') { $before + 1 } else { $before - 1 }
if ($expected -lt 1) { throw 'The fixture must retain its last tab' }
[HeldTabs]::Begin($DeadlineEpochMs)
try {
  foreach ($modifier in $modifiers) {
    Require-Focus
    [HeldTabs]::Press($modifier)
  }
  Require-Focus
  [HeldTabs]::Press($key)
  $deadline = [DateTime]::UtcNow.AddSeconds(10)
  while ((Tab-Count) -ne $expected) {
    Require-Focus
    if ([DateTime]::UtcNow -ge $deadline) { throw 'First native shortcut had no single acknowledged tab change' }
    Start-Sleep -Milliseconds 25
  }
  for ($index = 0; $index -lt 5; $index++) {
    Require-Focus
    [HeldTabs]::Press($key)
    Start-Sleep -Milliseconds 100
    if ((Tab-Count) -ne $expected) { throw 'A held repeat changed the acknowledged tab count' }
  }
} finally {
  [HeldTabs]::End()
}
[Console]::WriteLine('HELD_TABS sent=5 released=true')

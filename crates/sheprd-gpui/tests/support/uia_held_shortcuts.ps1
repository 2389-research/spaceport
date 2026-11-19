# ABOUTME: Delivers held shortcuts after observing their first private tab or window change.
# ABOUTME: Requires the exact private window and releases only keys this fixture pressed.
param([int]$ProcessId, [ValidateSet('New','Close','Window')][string]$Action, [long]$DeadlineEpochMs)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\powershell_modules.ps1"
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Threading;
public static class HeldKeys {
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
function Private-Windows {
  return $automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $owned)
}
$windows = @(Private-Windows)
if ($Action -ne 'Window' -and $windows.Count -ne 1) { throw 'Held-tab fixture requires exactly one private window' }
if ($windows.Count -eq 0) { throw 'Held-shortcut fixture has no private window' }
$roots = if ($windows.Count -eq 1) { $windows } else {
  @($windows | Where-Object { [IntPtr]$_.Current.NativeWindowHandle -eq [HeldKeys]::GetForegroundWindow() })
}
if (@($roots).Count -ne 1) { throw 'Held shortcut has no unique owned foreground window' }
$root = @($roots)[0]
if ($Action -ne 'Window') {
  $stripType = [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Tab)
  $stripName = [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Tabs')
  $strips = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.AndCondition]::new($stripType, $stripName))
  if ($strips.Count -ne 1) { throw 'Held-tab fixture requires exactly one terminal tab strip' }
  $strip = $strips[0]
}
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Private window has no native handle' }

function Require-Focus {
  if ([HeldKeys]::GetForegroundWindow() -ne $handle) { throw 'Private HWND lost foreground' }
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
function Target-Count {
  if ($Action -eq 'Window') { return @(Private-Windows).Count }
  return Tab-Count
}

# The existing fixture uses an Alt press to obtain foreground permission.
if (([HeldKeys]::GetAsyncKeyState(0x12) -band 0x8000) -ne 0) { throw 'Alt is already held' }
try { [HeldKeys]::Send(0x12, $false) } finally { [HeldKeys]::Send(0x12, $true) }
[HeldKeys]::SetForegroundWindow($handle) | Out-Null
Start-Sleep -Milliseconds 500
Require-Focus
$key = switch ($Action) { 'New' { 0x54 }; 'Close' { 0x57 }; 'Window' { 0x4e } }
$modifiers = switch ($Action) { 'New' { @(0x11,0x10) }; 'Close' { @(0x11,0x12,0x10) }; 'Window' { @(0x11,0x12) } }
foreach ($code in @($modifiers) + @($key)) {
  if (([HeldKeys]::GetAsyncKeyState($code) -band 0x8000) -ne 0) { throw 'A fixture key is already held' }
}
$before = Target-Count
$beforeHandles = @($windows | ForEach-Object { [IntPtr]$_.Current.NativeWindowHandle })
$expected = if ($Action -eq 'Close') { $before - 1 } else { $before + 1 }
if ($expected -lt 1) { throw 'The fixture must retain its last tab' }
[HeldKeys]::Begin($DeadlineEpochMs)
try {
  foreach ($modifier in $modifiers) {
    Require-Focus
    [HeldKeys]::Press($modifier)
  }
  Require-Focus
  [HeldKeys]::Press($key)
  $deadline = [DateTime]::UtcNow.AddSeconds(10)
  while ((Target-Count) -ne $expected) {
    if ($Action -ne 'Window') { Require-Focus }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'First native shortcut had no single acknowledged tab change' }
    Start-Sleep -Milliseconds 25
  }
  if ($Action -eq 'Window') {
    # Adopt only the one newly created PID-owned root before sending more keys.
    $added = @((Private-Windows) | Where-Object { [IntPtr]$_.Current.NativeWindowHandle -notin $beforeHandles })
    if ($added.Count -ne 1) { throw 'New Window did not create one unique private HWND' }
    $root = $added[0]
    $handle = [IntPtr]$root.Current.NativeWindowHandle
    if ($handle -eq [IntPtr]::Zero) { throw 'New Window has no native handle' }
    # Creation precedes GPUI's deferred activation. Wait for this exact root.
    $focused = $false
    while (-not $focused) {
      try { Require-Focus; $focused = $true }
      catch {
        if ([DateTime]::UtcNow -ge $deadline) { throw }
        Start-Sleep -Milliseconds 25
      }
    }
    # GPUI activation releases Alt. Restore only this fixture's modifiers;
    # the main N key remains down, so its next down is still a held event.
    Require-Focus
    if (([HeldKeys]::GetAsyncKeyState($key) -band 0x8000) -eq 0) { throw 'New Window released the main held key' }
    foreach ($modifier in $modifiers) {
      Require-Focus
      if (([HeldKeys]::GetAsyncKeyState($modifier) -band 0x8000) -eq 0) { [HeldKeys]::Press($modifier) }
    }
  }
  for ($index = 0; $index -lt 5; $index++) {
    Require-Focus
    foreach ($code in @($modifiers) + @($key)) {
      if (([HeldKeys]::GetAsyncKeyState($code) -band 0x8000) -eq 0) { throw 'The held shortcut lost a required key' }
    }
    [HeldKeys]::Press($key)
    Start-Sleep -Milliseconds 100
    if ((Target-Count) -ne $expected) { throw 'A held repeat changed the acknowledged target count' }
  }
} finally {
  [HeldKeys]::End()
}
[Console]::WriteLine('HELD_SHORTCUT sent=5 released=true')

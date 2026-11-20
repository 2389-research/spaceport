// ABOUTME: Identifies already loaded Windows keyboard layouts on the helper thread only.
// ABOUTME: Verifies exact restoration after normal enumeration and an intentional early failure.
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class SpaceportLayoutInventory {
  sealed class ProbeFailure : Exception { public readonly bool Changed; public ProbeFailure(bool changed) { Changed = changed; } }
  [DllImport("user32.dll", SetLastError=true)] static extern int GetKeyboardLayoutList(int count, [Out] IntPtr[] layouts);
  [DllImport("user32.dll")] static extern IntPtr GetKeyboardLayout(uint thread);
  [DllImport("user32.dll", SetLastError=true)] static extern IntPtr ActivateKeyboardLayout(IntPtr layout, uint flags);
  [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern bool GetKeyboardLayoutNameW(StringBuilder name);

  static void Activate(IntPtr layout) {
    if (ActivateKeyboardLayout(layout, 0) == IntPtr.Zero || GetKeyboardLayout(0) != layout)
      throw new InvalidOperationException("Helper thread keyboard layout activation failed");
  }

  static SortedSet<string> Collect(bool fail) {
    IntPtr original = GetKeyboardLayout(0);
    if (original == IntPtr.Zero) throw new InvalidOperationException("Helper has no keyboard layout");
    int count = GetKeyboardLayoutList(0, null);
    if (count < 1 || count > 64) throw new InvalidOperationException("Loaded keyboard layout count exceeds fixture bounds");
    IntPtr[] layouts = new IntPtr[count];
    if (GetKeyboardLayoutList(count, layouts) != count || GetKeyboardLayoutList(0, null) != count)
      throw new InvalidOperationException("Loaded keyboard layouts changed during enumeration");
    SortedSet<string> names = new SortedSet<string>(StringComparer.Ordinal);
    try {
      for (int index = 0; index < layouts.Length; index++) {
        if (layouts[index] == IntPtr.Zero) throw new InvalidOperationException("Loaded keyboard layout is empty");
        Activate(layouts[index]);
        StringBuilder name = new StringBuilder(9);
        if (!GetKeyboardLayoutNameW(name) || name.Length != 8)
          throw new InvalidOperationException("Loaded keyboard layout has no bounded KLID");
        string klid = name.ToString();
        foreach (char character in klid) {
          if (!Uri.IsHexDigit(character)) throw new InvalidOperationException("Loaded keyboard layout has an invalid KLID");
        }
        names.Add(klid.ToUpperInvariant());
        if (fail && (layouts[index] != original || index == layouts.Length - 1))
          throw new ProbeFailure(layouts[index] != original);
      }
    } finally {
      // Flags zero changes this synchronous method's thread, never the GUI or the process.
      Activate(original);
    }
    return names;
  }

  public static string Run() {
    SortedSet<string> names = Collect(false);
    bool changed = false, caught = false;
    try { Collect(true); }
    catch (ProbeFailure failure) { changed = failure.Changed; caught = true; }
    if (!caught) throw new InvalidOperationException("Expected keyboard layout failure path did not run");
    return "LAYOUT_INVENTORY klids=" + String.Join(",", names)
      + " polish_programmers_loaded=" + names.Contains("00000415").ToString().ToLowerInvariant()
      + " helper_restored=true failure_cleanup=true failure_changed_layout=" + changed.ToString().ToLowerInvariant();
  }
}

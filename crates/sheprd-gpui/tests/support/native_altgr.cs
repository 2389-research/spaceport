// ABOUTME: Scopes a stock Polish keyboard profile to one disposable Windows CI fixture.
// ABOUTME: Activates only the owned GUI thread and releases scan-code keys before restoring layouts.
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

[assembly: DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
public sealed class SpaceportAltGr : IDisposable {
  const string Tip = "0x0415:0x00000415";
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
  struct Profile {
    public uint type;
    public ushort language;
    public Guid clsid, profile, category;
    public uint substitute, flags;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=260)] public string id;
    public string Identity() {
      return type + ":" + language + ":" + clsid + ":" + profile + ":" + category
        + ":" + substitute + ":" + flags + ":" + id;
    }
  }
  [StructLayout(LayoutKind.Sequential)] struct Keyboard {
    public ushort key, scan; public uint flags, time; public UIntPtr extra;
  }
  [StructLayout(LayoutKind.Sequential)] struct Mouse {
    public int x, y; public uint data, flags, time; public UIntPtr extra;
  }
  [StructLayout(LayoutKind.Explicit)] struct Data {
    [FieldOffset(0)] public Keyboard keyboard;
    [FieldOffset(0)] public Mouse mouse;
  }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint type; public Data data; }
  struct Pressed { public ushort scan; public uint flags; }
  [DllImport("input.dll", CharSet=CharSet.Unicode, ExactSpelling=true)] static extern bool InstallLayoutOrTip(string profile, uint flags);
  [DllImport("input.dll", CharSet=CharSet.Unicode, ExactSpelling=true)] static extern uint EnumEnabledLayoutOrTip(string user, string system, string software, [Out] Profile[] profiles, uint count);
  [DllImport("user32.dll")] static extern int GetKeyboardLayoutList(int count, [Out] IntPtr[] layouts);
  [DllImport("user32.dll")] static extern IntPtr GetKeyboardLayout(uint thread);
  [DllImport("user32.dll")] static extern IntPtr ActivateKeyboardLayout(IntPtr layout, uint flags);
  [DllImport("user32.dll", SetLastError=true)] static extern bool UnloadKeyboardLayout(IntPtr layout);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern bool GetKeyboardLayoutNameW(StringBuilder name);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr handle, out uint process);
  [DllImport("user32.dll", SetLastError=true)] static extern bool PostMessageW(IntPtr handle, uint message, IntPtr word, IntPtr value);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr handle);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  public static IntPtr ForegroundWindow() { return GetForegroundWindow(); }
  [DllImport("user32.dll")] static extern short GetAsyncKeyState(int key);
  [DllImport("user32.dll")] static extern uint MapVirtualKeyExW(uint key, uint type, IntPtr layout);
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);

  readonly List<Pressed> pressed = new List<Pressed>();
  readonly IntPtr handle, original, initiallyLoadedPolish;
  readonly uint process, thread;
  readonly long deadlineEpochMs;
  readonly SortedSet<string> profiles;
  IntPtr polish;
  bool added, disposed;
  Timer watchdog;

  static Profile[] Profiles() {
    uint count = EnumEnabledLayoutOrTip(null, null, null, null, 0);
    if (count < 1 || count > 64) throw new InvalidOperationException("Enabled profile count exceeds fixture bounds");
    Profile[] result = new Profile[count];
    if (EnumEnabledLayoutOrTip(null, null, null, result, count) != count
        || EnumEnabledLayoutOrTip(null, null, null, null, 0) != count)
      throw new InvalidOperationException("Enabled profiles changed during enumeration");
    foreach (Profile value in result) {
      if (value.type != 2) continue;
      string klid = SpaceportProfileId.Klid(value.language, value.id);
      Console.WriteLine("ALTGR_PROFILE language=" + value.language.ToString("X4") + " klid=" + klid);
    }
    return result;
  }
  static SortedSet<string> Identities(Profile[] values) {
    SortedSet<string> result = new SortedSet<string>(StringComparer.Ordinal);
    foreach (Profile value in values) result.Add(value.Identity());
    return result;
  }
  static bool HasPolish(Profile[] values) {
    foreach (Profile value in values)
      if (value.type == 2 && value.language == 0x0415 && SpaceportProfileId.Klid(value.language, value.id) == "00000415") return true;
    return false;
  }
  static IntPtr LoadedPolish() {
    IntPtr original = GetKeyboardLayout(0);
    int count = GetKeyboardLayoutList(0, null);
    if (original == IntPtr.Zero || count < 1 || count > 64)
      throw new InvalidOperationException("Loaded layout inventory exceeds fixture bounds");
    IntPtr[] layouts = new IntPtr[count];
    if (GetKeyboardLayoutList(count, layouts) != count || GetKeyboardLayoutList(0, null) != count)
      throw new InvalidOperationException("Loaded layouts changed during enumeration");
    try {
      foreach (IntPtr layout in layouts) {
        if (layout == IntPtr.Zero || ActivateKeyboardLayout(layout, 0) == IntPtr.Zero || GetKeyboardLayout(0) != layout)
          throw new InvalidOperationException("Cannot identify a loaded layout on the helper thread");
        StringBuilder name = new StringBuilder(9);
        if (!GetKeyboardLayoutNameW(name) || name.Length != 8)
          throw new InvalidOperationException("Loaded layout has no exact KLID");
        if (name.ToString() == "00000415") return layout;
      }
      return IntPtr.Zero;
    } finally {
      if (ActivateKeyboardLayout(original, 0) == IntPtr.Zero || GetKeyboardLayout(0) != original)
        throw new InvalidOperationException("Helper layout restoration failed");
    }
  }
  SpaceportAltGr(IntPtr window, uint owner, long deadline) {
    if (Environment.GetEnvironmentVariable("SPACEPORT_DISPOSABLE_LAYOUT_TEST") != "github-hosted-windows")
      throw new InvalidOperationException("Layout setup requires the disposable CI fixture");
    handle = window; process = owner;
    deadlineEpochMs = deadline;
    thread = GetWindowThreadProcessId(handle, out owner);
    if (thread == 0 || owner != process) throw new InvalidOperationException("Layout target is not the owned GUI");
    original = GetKeyboardLayout(thread);
    if (original == IntPtr.Zero) throw new InvalidOperationException("Owned GUI has no original layout");
    profiles = Identities(Profiles());
    initiallyLoadedPolish = LoadedPolish();
  }
  public static SpaceportAltGr Begin(IntPtr window, uint owner, long deadlineEpochMs) {
    long remaining = deadlineEpochMs - DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() - 5000;
    if (remaining < 12000 || remaining > 35000)
      throw new InvalidOperationException("Insufficient parent budget for native input and layout cleanup");
    SpaceportAltGr scope = new SpaceportAltGr(window, owner, deadlineEpochMs);
    try {
      scope.RequireForeground();
      RequireReleased();
      if (!HasPolish(Profiles())) {
        scope.added = true;
        if (!InstallLayoutOrTip(Tip, 0)) throw new InvalidOperationException("Current-session Polish profile setup was refused");
      }
      DateTime deadline = DateTime.UtcNow.AddSeconds(5);
      while ((scope.polish = LoadedPolish()) == IntPtr.Zero) {
        if (DateTime.UtcNow >= deadline) throw new InvalidOperationException("Polish profile is not loaded in this session");
        Thread.Sleep(20);
      }
      scope.ActivateGui(scope.polish);
      remaining = deadlineEpochMs - DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() - 5000;
      if (remaining < 1000) throw new InvalidOperationException("Layout setup consumed the native input budget");
      scope.watchdog = new Timer(delegate {
        lock (scope.pressed) {
          if (scope.disposed) return;
          Console.WriteLine("ALTGR watchdog_expired=true tracked_keys=" + scope.pressed.Count);
          try { scope.Dispose(); }
          catch (Exception error) { Console.WriteLine("ALTGR watchdog_cleanup_failure=" + error.Message); }
          finally { Environment.Exit(73); }
        }
      }, null, (int)remaining, Timeout.Infinite);
      return scope;
    } catch (Exception error) {
      try { scope.Dispose(); }
      catch (Exception cleanup) { throw new AggregateException(error, cleanup); }
      throw;
    }
  }
  void RequireForeground() {
    uint owner;
    if (GetForegroundWindow() != handle || GetWindowThreadProcessId(handle, out owner) != thread || owner != process)
      throw new InvalidOperationException("Native input left the exact owned GUI");
  }
  void ActivateGui(IntPtr layout) {
    uint owner;
    if (GetWindowThreadProcessId(handle, out owner) != thread || owner != process)
      throw new InvalidOperationException("Layout target no longer belongs to the fixture");
    if (!PostMessageW(handle, 0x0050, IntPtr.Zero, layout)) throw new InvalidOperationException("Owned layout request was refused");
    DateTime deadline = DateTime.UtcNow.AddSeconds(3);
    while (GetKeyboardLayout(thread) != layout) {
      if (DateTime.UtcNow >= deadline) throw new InvalidOperationException("Owned GUI did not acknowledge its exact HKL");
      Thread.Sleep(10);
    }
  }
  public static void RequireReleased() {
    RequireKeysReleased(new int[] { 0x10, 0x11, 0x12, 0x5B, 0x5C, 0x4C, 0x4E, 0x5A });
  }
  public static void RequireKeysReleased(int[] keys) {
    foreach (int key in keys) {
      if (key < 1 || key > 255) throw new ArgumentException("Native virtual key exceeds fixture bounds");
      if ((GetAsyncKeyState(key) & 0x8000) != 0) throw new InvalidOperationException("A fixture key is already held: VK_" + key.ToString("X2"));
    }
  }
  static void Send(Pressed key, bool up) {
    Input input = new Input(); input.type = 1;
    input.data.keyboard.scan = key.scan;
    input.data.keyboard.flags = key.flags | (up ? 2u : 0u);
    if (SendInput(1, new Input[] { input }, Marshal.SizeOf(typeof(Input))) != 1)
      throw new InvalidOperationException("Owned scan-code input failed: " + Marshal.GetLastWin32Error());
  }
  void Press(uint key) {
    lock (pressed) {
      if (disposed) throw new InvalidOperationException("Native layout fixture already ended");
      if (DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() >= deadlineEpochMs - 5000)
        throw new InvalidOperationException("Native input reached its cleanup reserve");
      RequireForeground();
      if (GetKeyboardLayout(thread) != polish) throw new InvalidOperationException("Owned GUI lost its Polish HKL");
      uint scan = MapVirtualKeyExW(key, 4, polish);
      if ((scan & 0xff) == 0 || scan > 0xe0ff || ((scan & 0xff00) != 0 && (scan & 0xff00) != 0xe000))
        throw new InvalidOperationException("Native key has no supported scan code");
      Pressed value = new Pressed { scan = (ushort)(scan & 0xff), flags = 8u | ((scan & 0xff00) == 0xe000 ? 1u : 0u) };
      pressed.Add(value);
      Send(value, false);
    }
  }
  void Release() {
    lock (pressed) {
      List<Exception> errors = new List<Exception>();
      for (int index = pressed.Count - 1; index >= 0; index--) {
        try { Send(pressed[index], true); pressed.RemoveAt(index); }
        catch (Exception error) { errors.Add(error); }
      }
      // Retain uncertain releases so Dispose can still release those exact keys.
      if (errors.Count != 0) throw new AggregateException(errors);
    }
  }
  public void Type(bool shifted) {
    TypeLetter(shifted ? 0x4Eu : 0x4Cu, shifted);
  }
  public void TypeLetter(uint letter, bool shifted) {
    if (letter != 0x4C && letter != 0x4E && letter != 0x5A)
      throw new ArgumentException("AltGr fixture accepts only L, N, and Z");
    RequireReleased();
    try {
      Press(0xA5); // Extended RightAlt; Windows supplies AltGr's control state.
      if (shifted) Press(0xA0);
      Press(letter);
    } finally { Release(); }
  }
  public void Chord(int[] keys) {
    if (keys == null || keys.Length < 1 || keys.Length > 4 || new HashSet<int>(keys).Count != keys.Length)
      throw new ArgumentException("Native chord must contain one to four distinct keys");
    RequireReleased();
    RequireKeysReleased(keys);
    try { foreach (int key in keys) Press((uint)key); }
    finally { Release(); }
  }
  public void Dispose() {
    lock (pressed) { if (disposed) return; disposed = true; }
    if (watchdog != null) watchdog.Dispose();
    List<Exception> errors = new List<Exception>();
    try { Release(); RequireReleased(); } catch (Exception error) { errors.Add(error); }
    try { ActivateGui(original); } catch (Exception error) { errors.Add(error); }
    if (added) {
      try {
        if (HasPolish(Profiles()) && !InstallLayoutOrTip(Tip, 1))
          throw new InvalidOperationException("Added Polish profile removal was refused");
      } catch (Exception error) { errors.Add(error); }
    }
    try {
      if (!profiles.SetEquals(Identities(Profiles()))) throw new InvalidOperationException("Enabled keyboard profiles did not return to their original state");
    } catch (Exception error) { errors.Add(error); }
    if (added && initiallyLoadedPolish == IntPtr.Zero) {
      try {
        IntPtr remaining = LoadedPolish();
        if (remaining != IntPtr.Zero && !UnloadKeyboardLayout(remaining))
          throw new InvalidOperationException("Added keyboard layout remains loaded: " + Marshal.GetLastWin32Error());
        if (LoadedPolish() != IntPtr.Zero) throw new InvalidOperationException("Added keyboard layout did not unload");
      } catch (Exception error) { errors.Add(error); }
    }
    if (errors.Count != 0) throw new AggregateException(errors);
  }
}

// ABOUTME: Sends one wheel delta after claiming the private fixture's physical pointer position.
// ABOUTME: Restores only its own unchanged pointer claim before the parent's cleanup deadline.
using System;
using System.Runtime.InteropServices;
using System.Threading;

public static class ImageWheel {
  [StructLayout(LayoutKind.Sequential)] public struct Point { public int x, y; }
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
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] input, int size);
  [DllImport("user32.dll", SetLastError=true)] static extern bool GetPhysicalCursorPos(out Point point);
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetPhysicalCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  static readonly object gate = new object();
  static Point original, target;
  static IntPtr window;
  static bool claimed, ended, sent, restored;
  static Timer watchdog;

  public static void Begin(IntPtr handle, long deadlineEpochMs) {
    long remaining = deadlineEpochMs - DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() - 5000;
    if (remaining < 12000 || remaining > 45000)
      throw new InvalidOperationException("Insufficient native image input and cleanup budget");
    window = handle;
    watchdog = new Timer(delegate { Restore(); Environment.Exit(73); }, null, (int)remaining, Timeout.Infinite);
  }
  public static void Move(int x, int y) {
    lock (gate) {
      if (ended || claimed || GetForegroundWindow() != window)
        throw new InvalidOperationException("Native image pointer has no fresh owned foreground claim");
      if (!GetPhysicalCursorPos(out original)) throw new InvalidOperationException("Cannot read private fixture pointer origin");
      if (original.x == x && original.y == y) throw new InvalidOperationException("Native pointer claim must exercise a real move");
      target.x = x; target.y = y; claimed = true;
      if (!SetPhysicalCursorPos(x, y)) throw new InvalidOperationException("Cannot move private fixture pointer");
    }
  }
  public static Point Position() {
    Point point;
    if (!GetPhysicalCursorPos(out point)) throw new InvalidOperationException("Cannot read physical pointer position");
    return point;
  }
  public static bool AtTarget() {
    lock (gate) {
      Point current;
      return !ended && claimed && GetForegroundWindow() == window && GetPhysicalCursorPos(out current)
        && current.x == target.x && current.y == target.y;
    }
  }
  public static void Wheel() {
    lock (gate) {
      if (sent || !AtTarget()) throw new InvalidOperationException("Native image wheel lost its unique pointer claim");
      sent = true;
      Input input = new Input(); input.type = 0;
      input.data.mouse.data = 30; input.data.mouse.flags = 0x0800;
      if (SendInput(1, new Input[] { input }, Marshal.SizeOf(typeof(Input))) != 1)
        throw new InvalidOperationException("Native image wheel injection failed: " + Marshal.GetLastWin32Error());
    }
  }
  static void Restore() {
    lock (gate) {
      if (ended) return;
      Point current;
      if (claimed && GetForegroundWindow() == window && GetPhysicalCursorPos(out current)
          && current.x == target.x && current.y == target.y)
        restored = SetPhysicalCursorPos(original.x, original.y);
      ended = true;
      if (watchdog != null) watchdog.Dispose();
    }
  }
  public static bool End() { Restore(); return restored; }
}

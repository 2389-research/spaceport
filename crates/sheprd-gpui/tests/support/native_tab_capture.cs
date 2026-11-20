// ABOUTME: Captures only a bounded visible rectangle in the exact private foreground window.
// ABOUTME: Restores helper-thread DPI awareness and releases every temporary drawing resource.
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;

public static class NativeTabCapture {
  [StructLayout(LayoutKind.Sequential)] struct Point { public int x, y; }
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr window, uint flags);
  [DllImport("user32.dll", SetLastError=true)] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

  static void Owned(IntPtr window, Rectangle area) {
    if (window == IntPtr.Zero || GetForegroundWindow() != window)
      throw new InvalidOperationException("Tab capture lost its exact foreground window");
    foreach (Point point in new Point[] {
      new Point { x=area.Left, y=area.Top }, new Point { x=area.Right-1, y=area.Top },
      new Point { x=area.Left, y=area.Bottom-1 }, new Point { x=area.Right-1, y=area.Bottom-1 },
      new Point { x=area.Left+area.Width/2, y=area.Top+area.Height/2 }
    }) {
      if (GetAncestor(WindowFromPoint(point), 2) != window)
        throw new InvalidOperationException("Tab capture rectangle is occluded or outside its owned window");
    }
  }

  public static Bitmap Capture(IntPtr window, int x, int y, int width, int height) {
    if (width < 1 || height < 1 || (long)width*height > 2000000)
      throw new InvalidOperationException("Tab capture rectangle exceeds its pixel bound");
    IntPtr previous = SetThreadDpiAwarenessContext(new IntPtr(-4));
    if (previous == IntPtr.Zero) throw new InvalidOperationException("Cannot enter physical-pixel capture context");
    Bitmap bitmap = null;
    try {
      Rectangle area = new Rectangle(x, y, width, height);
      Owned(window, area);
      bitmap = new Bitmap(width, height, PixelFormat.Format32bppArgb);
      using (Graphics graphics = Graphics.FromImage(bitmap)) {
        graphics.CopyFromScreen(x, y, 0, 0, area.Size, CopyPixelOperation.SourceCopy);
      }
      Owned(window, area);
    } catch {
      if (bitmap != null) bitmap.Dispose();
      throw;
    } finally {
      if (SetThreadDpiAwarenessContext(previous) == IntPtr.Zero) {
        if (bitmap != null) bitmap.Dispose();
        throw new InvalidOperationException("Cannot restore helper-thread DPI context");
      }
    }
    return bitmap;
  }

  public static int ChangedBorder(Bitmap before, Bitmap after) {
    if (before.Size != after.Size) throw new InvalidOperationException("Tab capture dimensions changed");
    int changed = 0;
    for (int y=0; y<before.Height; y++) for (int x=0; x<before.Width; x++) {
      if ((x<2 || y<2 || x>=before.Width-2 || y>=before.Height-2)
          && before.GetPixel(x,y).ToArgb() != after.GetPixel(x,y).ToArgb()) changed++;
    }
    return changed;
  }

  public static string Png(Bitmap bitmap) {
    using (MemoryStream stream = new MemoryStream()) {
      bitmap.Save(stream, ImageFormat.Png);
      if (stream.Length > 131072) throw new InvalidOperationException("Tab PNG exceeds 128 KiB");
      return Convert.ToBase64String(stream.ToArray());
    }
  }
}

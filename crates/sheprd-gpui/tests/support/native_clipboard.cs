// ABOUTME: Publishes authored Windows clipboard data only after an empty or exact-owner check.
// ABOUTME: Keeps native allocation transfer and cleanup behind the fixture's owned HWND and sequence fence.
using System;
using System.ComponentModel;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Windows.Forms;

public sealed class ClipboardOccupiedException : Exception {
    public ClipboardOccupiedException() : base("Native clipboard is nonempty; refusing unchanged") { }
}

public sealed class ClipboardOwnershipException : Exception {
    public ClipboardOwnershipException() : base("Refusing a stale or foreign clipboard publication") { }
}

public sealed class ClipboardHoldExpiredException : TimeoutException {
    public ClipboardHoldExpiredException() : base("GUI callback outlived the bounded clipboard hold") { }
}

public sealed class NativeClipboardFixture : IDisposable {
    const uint UnicodeText = 13;
    const int MaxUnits = 2 * 1024 * 1024 + 1;
    readonly Form window;
    readonly IntPtr handle;
    uint? publication;
    bool disposed;
    public string Cleanup { get; private set; }
    static readonly object Gate = new object();
    static readonly List<NativeClipboardFixture> Active = new List<NativeClipboardFixture>();
    static volatile bool cancelled;

    [DllImport("user32.dll", SetLastError = true)] static extern bool OpenClipboard(IntPtr owner);
    [DllImport("user32.dll", SetLastError = true)] static extern bool CloseClipboard();
    [DllImport("user32.dll", SetLastError = true)] static extern bool EmptyClipboard();
    [DllImport("user32.dll", SetLastError = true)] static extern int CountClipboardFormats();
    [DllImport("user32.dll")] static extern IntPtr GetClipboardOwner();
    [DllImport("user32.dll")] static extern uint GetClipboardSequenceNumber();
    [DllImport("user32.dll", SetLastError = true)] static extern IntPtr SetClipboardData(uint format, IntPtr memory);
    [DllImport("user32.dll", SetLastError = true)] static extern IntPtr GetClipboardData(uint format);
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)] static extern uint RegisterClipboardFormat(string name);
    [DllImport("kernel32.dll")] static extern void SetLastError(uint error);
    [DllImport("kernel32.dll", SetLastError = true)] static extern IntPtr GlobalAlloc(uint flags, UIntPtr bytes);
    [DllImport("kernel32.dll", SetLastError = true)] static extern IntPtr GlobalLock(IntPtr memory);
    [DllImport("kernel32.dll", SetLastError = true)] static extern bool GlobalUnlock(IntPtr memory);
    [DllImport("kernel32.dll", SetLastError = true)] static extern IntPtr GlobalFree(IntPtr memory);
    [DllImport("kernel32.dll", SetLastError = true)] static extern UIntPtr GlobalSize(IntPtr memory);

    static Win32Exception Failed(string operation) {
        return new Win32Exception(Marshal.GetLastWin32Error(), operation);
    }

    static void Unlock(IntPtr memory) {
        SetLastError(0);
        bool locked = GlobalUnlock(memory);
        int error = Marshal.GetLastWin32Error();
        if (!locked && error != 0) throw new Win32Exception(error, "Unlock authored clipboard allocation");
    }

    public NativeClipboardFixture() {
        if (Thread.CurrentThread.GetApartmentState() != ApartmentState.STA)
            throw new InvalidOperationException("Native clipboard fixture requires STA");
        window = new Form();
        handle = window.Handle; // A real owned, hidden HWND; the window is never shown or focused.
        lock (Gate) { try {
            CheckCancellation();
            Open();
            try { RequireEmpty(); }
            finally { Close(); }
            Active.Add(this);
        } catch {
            window.Dispose();
            throw;
        } }
    }

    static void CheckCancellation() {
        if (cancelled) throw new OperationCanceledException("Native clipboard owner is stopping");
    }

    public static void CancelAndClean() {
        cancelled = true;
        lock (Gate) {
            if (Active.Count != 0) {
                var access = Active[0];
                access.Open();
                try {
                    foreach (var fixture in Active) {
                        if (fixture.OwnsPublication()) {
                            if (!EmptyClipboard()) throw Failed("Guardian cleanup of exact-owned clipboard");
                            RequireEmpty();
                            break;
                        }
                    }
                } finally { Close(); }
            }
        }
        Console.WriteLine("CLIPBOARD_GUARDIAN cleanup_safe=true");
    }

    void Open() {
        DateTime deadline = DateTime.UtcNow.AddSeconds(2);
        while (!OpenClipboard(handle)) {
            if (DateTime.UtcNow >= deadline) throw Failed("Open owned fixture clipboard");
            Thread.Sleep(10);
        }
    }

    static void Close() {
        if (!CloseClipboard()) throw Failed("Close owned fixture clipboard");
    }

    static void RequireEmpty() {
        SetLastError(0);
        int count = CountClipboardFormats();
        int error = Marshal.GetLastWin32Error();
        if (count == 0 && error != 0) throw new Win32Exception(error, "Count clipboard formats");
        if (count != 0) throw new ClipboardOccupiedException();
    }

    bool OwnsPublication() {
        return publication.HasValue && GetClipboardOwner() == handle
            && GetClipboardSequenceNumber() == publication.Value;
    }

    sealed class Allocation : IDisposable {
        IntPtr memory;
        public Allocation(byte[] bytes) {
            memory = GlobalAlloc(2, new UIntPtr((uint)bytes.Length)); // GMEM_MOVEABLE.
            if (memory == IntPtr.Zero) throw Failed("Allocate authored clipboard data");
            try {
                IntPtr pointer = GlobalLock(memory);
                if (pointer == IntPtr.Zero) throw Failed("Lock authored clipboard allocation");
                try { Marshal.Copy(bytes, 0, pointer, bytes.Length); }
                finally { Unlock(memory); }
            } catch { Dispose(); throw; }
        }
        public void Transfer(uint format) {
            if (SetClipboardData(format, memory) == IntPtr.Zero) throw Failed("Publish authored clipboard format");
            memory = IntPtr.Zero; // Windows owns and frees a successful transfer.
        }
        public void Dispose() {
            if (memory != IntPtr.Zero) {
                if (GlobalFree(memory) != IntPtr.Zero) throw Failed("Free untransferred clipboard allocation");
                memory = IntPtr.Zero;
            }
        }
    }

    public void Publish(string text) {
        lock (Gate) {
        CheckCancellation();
        if (disposed) throw new ObjectDisposedException("NativeClipboardFixture");
        if (text.Length > MaxUnits || text.IndexOf('\0') >= 0)
            throw new ArgumentException("Authored clipboard payload exceeds its bounded text contract");
        uint excluded = RegisterClipboardFormat("ExcludeClipboardContentFromMonitorProcessing");
        if (excluded == 0) throw Failed("Register clipboard history exclusion");
        using (var content = new Allocation(Encoding.Unicode.GetBytes(text + "\0")))
        using (var exclusion = new Allocation(new byte[] { 0 })) {
            bool emptied = false;
            Open();
            try {
                if (!OwnsPublication()) RequireEmpty();
                if (!EmptyClipboard()) throw Failed("Empty exact-owned fixture clipboard");
                emptied = true;
                // Publish the exclusion before text, while access remains exclusive.
                exclusion.Transfer(excluded);
                content.Transfer(UnicodeText);
            } finally {
                try { Close(); }
                finally {
                    // Also track a partial publish so failure cleanup can remove only its own data.
                    if (emptied) publication = GetClipboardSequenceNumber();
                }
            }
        }
        }
    }

    public void AssertOwnText(string expected) {
        lock (Gate) {
        CheckCancellation();
        Open();
        try {
            if (!OwnsPublication()) throw new ClipboardOwnershipException();
            IntPtr memory = GetClipboardData(UnicodeText);
            if (memory == IntPtr.Zero) throw Failed("Read exact-owned authored text");
            int bytes = checked((expected.Length + 1) * 2);
            ulong allocation = GlobalSize(memory).ToUInt64();
            if (allocation < (ulong)bytes || allocation > 8 * 1024 * 1024)
                throw new InvalidOperationException("Exact-owned clipboard allocation escaped its bound");
            IntPtr pointer = GlobalLock(memory);
            if (pointer == IntPtr.Zero) throw Failed("Lock exact-owned authored text");
            byte[] data = new byte[bytes];
            try { Marshal.Copy(pointer, data, 0, bytes); }
            finally { Unlock(memory); }
            if (data[bytes - 1] != 0 || data[bytes - 2] != 0
                || Encoding.Unicode.GetString(data, 0, bytes - 2) != expected)
                throw new InvalidOperationException("Exact-owned authored clipboard text changed");
        } finally { Close(); }
        }
    }

    public void ClearOwn() {
        lock (Gate) {
        CheckCancellation();
        Open();
        try {
            if (!OwnsPublication()) throw new ClipboardOwnershipException();
            if (!EmptyClipboard()) throw Failed("Clear exact-owned authored clipboard");
            RequireEmpty();
            // Retain the old publication fence. It can never authorize a later publication.
        } finally { Close(); }
        }
    }

    public void Hold(Action action) {
        var opened = new ManualResetEvent(false);
        var release = new ManualResetEvent(false);
        Exception failure = null;
        bool expired = false;
        var holder = new Thread(delegate() {
            try {
                lock (Gate) {
                    CheckCancellation();
                    Open();
                    try {
                        if (!OwnsPublication()) throw new ClipboardOwnershipException();
                    } catch { Close(); throw; }
                }
                try {
                    opened.Set();
                    // Release independently if UIA blocks against a regressed GUI thread.
                    expired = !release.WaitOne(2000);
                } finally { Close(); }
            } catch (Exception error) { failure = error; opened.Set(); }
        });
        holder.IsBackground = true;
        holder.SetApartmentState(ApartmentState.STA);
        holder.Start();
        bool joined = false;
        try {
            if (!opened.WaitOne(2500)) throw new TimeoutException("Owned clipboard hold did not start");
            if (failure != null) throw failure;
            // Only native clipboard calls run on the holder; this callback keeps its PS runspace.
            action();
        } finally {
            release.Set();
            joined = holder.Join(4500);
            if (joined) { opened.Dispose(); release.Dispose(); }
        }
        if (!joined) throw new TimeoutException("Owned clipboard holder did not exit");
        if (failure != null) throw failure;
        if (expired) throw new ClipboardHoldExpiredException();
    }

    public void AssertStaleFence() {
        lock (Gate) {
            Open();
            try {
                if (GetClipboardOwner() != handle || !publication.HasValue
                    || GetClipboardSequenceNumber() == publication.Value)
                    throw new InvalidOperationException("Stale-fence check did not retain the same HWND with a changed sequence");
            } finally { Close(); }
            bool refused = false;
            try { ClearOwn(); } catch (ClipboardOwnershipException) { refused = true; }
            if (!refused) throw new InvalidOperationException("Stale sequence authorized a second clear");
            Console.WriteLine("CLIPBOARD_GUARD same_hwnd_stale_sequence_refused=true");
        }
    }

    public void Dispose() {
        lock (Gate) {
        if (disposed) return;
        bool clean = false;
        try {
            Open();
            try {
                if (OwnsPublication()) {
                    if (!EmptyClipboard()) throw Failed("Clean exact-owned authored clipboard");
                    RequireEmpty();
                    Cleanup = "cleared";
                } else { Cleanup = "preserved"; }
            } finally { Close(); }
            clean = true;
        } finally {
            // A failed cleanup retains its lease and HWND for the independent guardian.
            if (clean) {
                disposed = true;
                Active.Remove(this);
                window.Dispose();
            }
        }
        }
    }
}

public sealed class ExpectedClipboardFailure : Exception { }

public static class NativeClipboardChecks {
    public static void Run(string stopFile, bool watchdog) {
        RunOwned(stopFile, 20, delegate() {
            if (watchdog) {
                using (var held = new NativeClipboardFixture()) {
                    held.Publish("Authored watchdog payload");
                    Console.WriteLine("CLIPBOARD_GUARDIAN publication_ready=true");
                    File.WriteAllText(stopFile + ".ready", "ready");
                    Application.Run(new ApplicationContext());
                }
            } else { OwnershipChecks(); }
        });
    }

    public static void RunOwned(string stopFile, int seconds, Action action) {
        if (seconds < 20 || seconds > 90) throw new ArgumentOutOfRangeException("seconds");
        if (File.Exists(stopFile)) throw new OperationCanceledException("Clipboard owner was stopped before startup");
        using (var finished = new ManualResetEvent(false)) {
            var guardian = new Thread(delegate() {
                DateTime deadline = DateTime.UtcNow.AddSeconds(seconds);
                while (!finished.WaitOne(20)) {
                    if (!File.Exists(stopFile) && DateTime.UtcNow < deadline) continue;
                    try {
                        NativeClipboardFixture.CancelAndClean();
                        File.WriteAllText(stopFile + ".cleaned", "safe");
                        Environment.Exit(3);
                    } catch (Exception error) {
                        Console.Error.WriteLine("CLIPBOARD_GUARDIAN cleanup_pending=" + error.GetType().Name);
                        Thread.Sleep(1000);
                    }
                }
            });
            guardian.IsBackground = true;
            guardian.Start();
            try {
                File.WriteAllText(stopFile + ".started", "ready");
                if (File.Exists(stopFile)) throw new OperationCanceledException("Clipboard owner stopped before publication");
                action();
            } finally {
                bool safe = false;
                try {
                    NativeClipboardFixture.CancelAndClean();
                    File.WriteAllText(stopFile + ".cleaned", "safe");
                    safe = true;
                } catch (Exception error) {
                    Console.Error.WriteLine("CLIPBOARD_GUARDIAN main_cleanup_pending=" + error.GetType().Name);
                }
                if (safe) { finished.Set(); guardian.Join(); }
                else {
                    // Keep the HWND, leases, and message pump alive while the guardian retries.
                    // No further publication can pass the cancelled guard.
                    Application.Run(new ApplicationContext());
                }
            }
        }
    }

    static void OwnershipChecks() {
        const string text = "Authored\r\n\u00e9\u754c\ud83e\udd80";
        var first = new NativeClipboardFixture();
        try {
            first.Publish(text);
            first.AssertOwnText(text);
            bool callbackFailed = false;
            try {
                first.Hold(delegate() { throw new ExpectedClipboardFailure(); });
            } catch (ExpectedClipboardFailure) { callbackFailed = true; }
            if (!callbackFailed) throw new InvalidOperationException("The hold callback's failure was lost");
            first.AssertOwnText(text);
            bool holdExpired = false;
            try {
                first.Hold(delegate() { Thread.Sleep(2200); });
            } catch (ClipboardHoldExpiredException) { holdExpired = true; }
            if (!holdExpired) throw new InvalidOperationException("The independent clipboard hold did not expire");
            first.AssertOwnText(text);
            Console.WriteLine("CLIPBOARD_GUARD bounded_hold_released=true callback_failure_released=true");
            bool refused = false;
            try {
                using (var unexpected = new NativeClipboardFixture()) { }
            } catch (ClipboardOccupiedException) { refused = true; }
            if (!refused) throw new InvalidOperationException("Nonempty clipboard guard did not refuse");
            first.AssertOwnText(text);
            Console.WriteLine("CLIPBOARD_GUARD nonempty_refused=true publication_preserved=true");

            foreach (int length in new int[] { 262144, 2097153 }) {
                string payload = new string('a', length);
                first.Publish(payload);
                first.AssertOwnText(payload);
                Console.WriteLine("CLIPBOARD_BROKER authored_utf8_bytes=" + length);
            }
            first.ClearOwn();
            first.AssertStaleFence();
            using (var successor = new NativeClipboardFixture()) {
                successor.Publish(text);
                first.Dispose();
                if (first.Cleanup != "preserved") throw new InvalidOperationException("Cleanup cleared a successor's publication");
                successor.AssertOwnText(text);
                Console.WriteLine("CLIPBOARD_GUARD foreign_publication_preserved=true");
            }
        } finally { first.Dispose(); }

        var failing = new NativeClipboardFixture();
        try {
            try {
                failing.Publish(text);
                throw new ExpectedClipboardFailure();
            } finally { failing.Dispose(); }
        } catch (ExpectedClipboardFailure) {
            if (failing.Cleanup != "cleared") throw new InvalidOperationException("Failure did not clean exact-owned clipboard data");
        }
        using (var empty = new NativeClipboardFixture()) {
            Console.WriteLine("CLIPBOARD_GUARD failure_cleanup=true empty=true");
        }
        Console.WriteLine("CLIPBOARD BROKER native PASS: empty-only admission, authored Unicode, exact owner/sequence, successor preservation, failure cleanup");
    }
}

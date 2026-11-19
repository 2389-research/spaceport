// ABOUTME: Observes live-region events only from the retained private GUI window.
// ABOUTME: Bounds callback data and unregisters on its dedicated MTA thread.
using System;
using System.Collections.Generic;
using System.Threading;
using System.Windows.Automation;

public sealed class SpaceportLiveEvents : IDisposable {
    private readonly int processId;
    private readonly IntPtr handle;
    private readonly object gate = new object();
    private readonly List<string> records = new List<string>();
    private readonly ManualResetEventSlim ready = new ManualResetEventSlim(false);
    private readonly ManualResetEventSlim stop = new ManualResetEventSlim(false);
    private readonly Thread thread;
    private Exception error;
    private int received;

    public SpaceportLiveEvents(IntPtr handle, int processId) {
        this.handle = handle;
        this.processId = processId;
        thread = new Thread(Run);
        thread.IsBackground = true;
        thread.SetApartmentState(ApartmentState.MTA);
        thread.Start();
        try {
            if (!ready.Wait(5000)) throw new TimeoutException("Private UIA registration exceeded its deadline");
            Check();
        } catch {
            Dispose();
            throw;
        }
    }

    private void Fail(Exception failure) {
        lock (gate) { if (error == null) error = failure; }
    }

    private void Check() {
        lock (gate) {
            if (error != null) throw new InvalidOperationException("Private UIA observer failed", error);
        }
    }

    private void OnEvent(object sender, AutomationEventArgs args) {
        try {
            var element = (AutomationElement)sender;
            if (element.Current.ProcessId != processId) return;
            if (args.EventId.Id != 20024) throw new InvalidOperationException("Unexpected UIA event");
            string name = element.Current.Name;
            if (name.Length > 4096) throw new InvalidOperationException("Private UIA name exceeds limit");
            lock (gate) {
                if (++received > 64) throw new InvalidOperationException("Private UIA event count exceeds limit");
                records.Add(name);
            }
        } catch (ElementNotAvailableException) {
            // A leaving node can disappear before its callback reads it.
        } catch (Exception failure) { Fail(failure); }
    }

    private void Run() {
        AutomationElement window = null;
        AutomationEvent eventId = null;
        AutomationEventHandler handler = OnEvent;
        bool registered = false;
        try {
            window = AutomationElement.FromHandle(handle);
            if (window.Current.ProcessId != processId) throw new InvalidOperationException("UIA window owner differs");
            eventId = AutomationElementIdentifiers.LiveRegionChangedEvent;
            if (eventId == null || eventId.Id != 20024) throw new InvalidOperationException("Native live-region event unavailable");
            Automation.AddAutomationEventHandler(eventId, window, TreeScope.Subtree, handler);
            registered = true;
            ready.Set();
            stop.Wait();
        } catch (Exception failure) {
            Fail(failure);
            ready.Set();
        } finally {
            if (registered) {
                try { Automation.RemoveAutomationEventHandler(eventId, window, handler); }
                catch (Exception failure) { Fail(failure); }
            }
        }
    }

    public string[] Drain() {
        Check();
        lock (gate) {
            var result = records.ToArray();
            records.Clear();
            return result;
        }
    }

    public void Dispose() {
        stop.Set();
        if (!thread.Join(5000)) throw new TimeoutException("Private UIA removal exceeded its deadline");
        ready.Dispose();
        stop.Dispose();
        Check();
    }
}

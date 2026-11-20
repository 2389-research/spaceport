// ABOUTME: Tests Polish evdev input in one GTK field on the private GNOME display.
// ABOUTME: This throwaway prerequisite never starts or qualifies Spaceport.
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk?version=4.0';

const bus = Gio.bus_get_sync(Gio.BusType.SESSION, null);
const settings = new Gio.Settings({schema_id: 'org.gnome.desktop.input-sources'});
const previous = settings.get_value('sources');
const managerPath = '/org/gnome/Mutter/RemoteDesktop';
const manager = 'org.gnome.Mutter.RemoteDesktop';
const sessionInterface = `${manager}.Session`;
const down = new Set();
let session = null;
let phase = 'prepare-overview';
let passed = false;
let window = null;
let entry = null;
let loop = null;
let actualText = '';
const deadline = GLib.get_monotonic_time() + 20_000_000;

function call(path, iface, method, parameters = null) {
    return bus.call_sync(manager, path, iface, method, parameters, null,
        Gio.DBusCallFlags.NONE, 2000, null);
}

function observeShell(phase) {
    try {
        const values = bus.call_sync('org.gnome.Shell', '/org/gnome/Shell',
            'org.freedesktop.DBus.Properties', 'GetAll',
            new GLib.Variant('(s)', ['org.gnome.Shell']), null,
            Gio.DBusCallFlags.NONE, 2000, null).deep_unpack()[0];
        const state = {};
        for (const name of ['Mode', 'OverviewActive', 'ShellVersion']) {
            const value = values[name];
            state[name] = value instanceof GLib.Variant ? value.deep_unpack() : value;
        }
        print(`GNOME_INPUT shell_state phase=${phase} state=${JSON.stringify(state)}`);
        return state;
    } catch (error) {
        printerr(`GNOME_INPUT shell_state phase=${phase} unavailable=${error.message}`);
        return null;
    }
}

function key(code, pressed) {
    // Retain uncertain presses until a release call has actually returned.
    if (pressed)
        down.add(code);
    call(session, sessionInterface, 'NotifyKeyboardKeycode',
        new GLib.Variant('(ub)', [code, pressed]));
    if (!pressed)
        down.delete(code);
}

function finish(success) {
    passed = success;
    actualText = entry.get_text();
    loop.quit();
    return GLib.SOURCE_REMOVE;
}

function cleanup() {
    let cleanupPassed = true;
    function attempt(operation) {
        try {
            operation();
            return true;
        } catch (error) {
            printerr(`GNOME_INPUT cleanup failed: ${error.message}`);
            cleanupPassed = false;
            return false;
        }
    }
    for (const code of [...down].reverse())
        attempt(() => key(code, false));
    const closed = session !== null && attempt(() => call(session, sessionInterface, 'Stop'));
    const restored = attempt(() => {
        if (!settings.set_value('sources', previous))
            throw new Error('private input-source restoration was refused');
        Gio.Settings.sync();
        if (!settings.get_value('sources').equal(previous))
            throw new Error('private input-source restoration was not observed');
    });
    if (window !== null)
        attempt(() => window.destroy());
    passed = passed && cleanupPassed && closed && down.size === 0;
    print(`GNOME_INPUT gtk_native=${passed} transport=evdev sequence_sent=${phase === 'observe-text'} polish=${JSON.stringify(actualText)} keys_released=${down.size === 0} session_closed=${closed} layout_restored=${restored} spaceport_started=false`);
}

try {
    if (!settings.set_value('sources', new GLib.Variant('a(ss)', [['xkb', 'pl']])))
        throw new Error('private Polish input source was refused');
    Gio.Settings.sync();
    Gtk.init();
    loop = new GLib.MainLoop(null, false);
    window = new Gtk.Window({title: 'Spaceport private input prerequisite'});
    entry = new Gtk.Entry();
    window.set_child(entry);
    window.present();
    entry.grab_focus();

    window.connect('close-request', () => finish(false));
    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
        try {
            if (GLib.get_monotonic_time() >= deadline) {
                observeShell('deadline');
                throw new Error(`GTK input deadline: mapped=${window.get_mapped()} active=${window.is_active} text=${JSON.stringify(entry.get_text())}`);
            }
            if (typeof window.is_active !== 'boolean')
                throw new Error('GTK active-window property is not Boolean');
            if (!window.get_mapped())
                return GLib.SOURCE_CONTINUE;
            if (phase === 'prepare-overview') {
                const state = observeShell('mapped');
                if (state?.Mode !== 'user' || typeof state.OverviewActive !== 'boolean')
                    throw new Error('private GNOME user-session state is unavailable');
                if (state.OverviewActive) {
                    bus.call_sync('org.gnome.Shell', '/org/gnome/Shell',
                        'org.freedesktop.DBus.Properties', 'Set',
                        new GLib.Variant('(ssv)', ['org.gnome.Shell', 'OverviewActive',
                            new GLib.Variant('b', false)]), null,
                        Gio.DBusCallFlags.NONE, 2000, null);
                    print('GNOME_INPUT overview_close_requested=true private_display=true');
                }
                phase = 'wait-overview';
                return GLib.SOURCE_CONTINUE;
            }
            if (phase === 'wait-overview') {
                const state = observeShell('overview');
                if (typeof state?.OverviewActive !== 'boolean')
                    throw new Error('private overview state is unavailable');
                if (state.OverviewActive)
                    return GLib.SOURCE_CONTINUE;
                phase = 'activate';
            }
            if (phase === 'activate') {
                session = call(managerPath, manager, 'CreateSession').deep_unpack()[0];
                call(session, sessionInterface, 'Start');
                if (!window.is_active) {
                    // GNOME's default Alt-Tab activates the only owned app.
                    key(56, true);
                    key(15, true);
                    key(15, false);
                    key(56, false);
                    print('GNOME_INPUT activation_requested=true route=alt-tab private_display=true');
                }
                phase = 'type';
                return GLib.SOURCE_CONTINUE;
            }
            if (!window.is_active)
                return GLib.SOURCE_CONTINUE;
            if (phase === 'type') {
                if (!entry.grab_focus())
                    throw new Error('owned GTK field refused focus');
                // Linux evdev codes: Right Alt, L, Left Shift, and N.
                key(100, true);
                key(38, true);
                key(38, false);
                key(100, false);
                key(42, true);
                key(100, true);
                key(49, true);
                key(49, false);
                key(100, false);
                key(42, false);
                phase = 'observe-text';
            }
            if (entry.get_text() === 'łŃ')
                return finish(true);
            return GLib.SOURCE_CONTINUE;
        } catch (error) {
            printerr(`GNOME_INPUT failed: ${error.message}`);
            return finish(false);
        }
    });
    loop.run();
} finally {
    cleanup();
}
if (!passed)
    throw new Error('private GTK native input prerequisite did not pass');

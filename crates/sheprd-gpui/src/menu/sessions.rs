// ABOUTME: Prepares session picker entries and routes exact session choices.
// ABOUTME: Keeps discovery, mutation, and selection ownership outside departure paint.
//! The sessions a device unfolds in the device picker: the named sessions on
//! this machine, or the ones a saved device reports, each under the device it
//! runs on.
//! Both lists are probes — a session socket outlives its daemon, and a device's
//! sessions cost an SSH round trip — so only an unfolded device asks.
// i18n: migrated
use super::devices::picker::Line;
use super::{Page, colors, devices};
use crate::i18n::fl;
use crate::{SheprdWindow, endpoint::Endpoint, sessions::DeviceScan};
use gpui::{prelude::*, *};
use sheprd_client::{ConnectTarget, LocalSession};
mod management;
mod rows;

pub(in crate::menu) use management::EditPaint;
pub(super) use management::{Edit, Target};
pub(in crate::menu) use rows::{NotePaint, SessionPaint};

/// A line under an unfolded device, in the order it is painted. Only a row can
/// be selected, so navigation walks the `Row` entries and skips notes.
pub(super) enum Entry {
    /// A muted note, carrying the selector diagnostics and tests look for.
    Muted {
        selector: String,
        text: String,
    },
    Error(String),
    /// A selectable line, carrying what it paints: a row resolved again at paint
    /// time could describe a session its own scan has already replaced.
    Row {
        row: Row,
        spec: Spec,
    },
}

/// What a row attaches to. A device keeps its endpoint id rather than an index,
/// so a catalog change between paint and click cannot pick the wrong one, and it
/// names the session itself, because a device has many.
#[derive(Clone, PartialEq)]
pub(super) enum Row {
    Local(String),
    Device { id: String, session: String },
    Add(Target),
}

/// What one row paints. Both kinds share one shape so they cannot drift apart.
#[derive(Clone)]
pub(super) struct Spec {
    label: SharedString,
    detail: SharedString,
    /// Running locally, or connected remotely.
    online: bool,
    checked: bool,
    enabled: bool,
}

impl Entry {
    pub(super) fn row(&self) -> Option<&Row> {
        match self {
            Self::Row { row, .. } => Some(row),
            _ => None,
        }
    }
}

/// The session a saved device attaches to. Asking the host what else it runs
/// would be an SSH round trip this popup does not take.
pub(super) fn target_session(target: &ConnectTarget) -> String {
    match target {
        ConnectTarget::Ssh { session, .. }
        | ConnectTarget::Wsl { session, .. }
        | ConnectTarget::Session { name: session, .. } => session.clone(),
        ConnectTarget::Socket(path) => path.display().to_string(),
        ConnectTarget::Local => "default".to_owned(),
    }
}

/// The state word a session paints, whether it was probed on this machine or
/// reported by its host.
fn state_word(running: bool) -> String {
    if running {
        fl!("sessions-menu-running")
    } else {
        fl!("sessions-menu-stopped")
    }
}

impl SheprdWindow {
    pub(super) fn finish_session_departure(&mut self) {
        // Selection is an ordinal shared with keyboard navigation. Once a row
        // disappears it must not transfer to the action now occupying its slot.
        if self.sessions.departure.is_some() && self.menu.page == Some(Page::Devices) {
            self.menu.selected = None;
        }
        self.sessions.finish_departure();
    }

    /// Opens the device picker with the device this window is on unfolded to
    /// its sessions, anchored where the footer's picker sits.
    pub(crate) fn open_sessions(
        &mut self,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.open_menu(window, cx) {
            return;
        }
        // Anchor to the control, not the pointer: every click position leaves the
        // same clear gap above the button.
        self.menu.anchor = anchor;
        self.menu.page = Some(Page::Devices);
        self.unfold_device(self.selected_endpoint);
        // The keys start on the session this window is on, among the others
        // they are opened to choose from.
        let current = self.picker_ordinal(
            |line| matches!(line, Line::Session(Entry::Row { spec, .. }) if spec.checked),
        );
        if current.is_some() {
            self.menu.selected = current;
        }
    }

    /// The socket this window's local endpoint would dial. `Local` resolves
    /// through the environment, so no session name can say which one is current.
    fn local_socket(&self) -> Option<std::path::PathBuf> {
        self.endpoints[0].connection.target.socket_path().ok()
    }

    /// The lines device `index` unfolds: its sessions, the notes that explain
    /// them, and its Add Session row. Index 0 is this machine.
    pub(super) fn device_session_entries(&self, index: usize) -> Vec<Entry> {
        let mut entries = Vec::new();
        if let Some(error) = &self.sessions.mutation_error {
            entries.push(Entry::Error(error.clone()));
        }
        if index == 0 {
            self.local_entries(&mut entries);
            // This machine's scan error belongs to its own sessions, not to a
            // saved device's.
            if let Some(error) = &self.sessions.error {
                entries.push(Entry::Error(error.clone()));
            }
        } else if let Some(endpoint) = self.endpoints.get(index) {
            self.device_entries(&mut entries, endpoint, index);
        }
        entries
    }

    /// One saved device's sessions, under the device's own row. The session it
    /// was saved with leads, and stands in for the list while that is still
    /// arriving or could not be read at all, so a host is never unreachable from
    /// the picker just because its session list is.
    fn device_entries(&self, entries: &mut Vec<Entry>, endpoint: &Endpoint, index: usize) {
        let saved = target_session(&endpoint.connection.target);
        let attached = self.selected_endpoint == index;
        let fallback = || {
            (
                saved.clone(),
                endpoint.live.status.to_string(),
                endpoint.live.status.is_connected(),
                attached,
            )
        };
        let (rows, note) = match self.sessions.devices.answers.get(&endpoint.id) {
            Some(DeviceScan::Sessions(sessions)) if !sessions.is_empty() => {
                let mut rows = Vec::with_capacity(sessions.len() + 1);
                // The session this device attaches to stays selectable even when
                // the host no longer lists it: the picker offers it too, because
                // the daemon starts a named session on demand.
                if !sessions.iter().any(|session| session.name == saved) {
                    rows.push(fallback());
                }
                rows.extend(
                    sessions
                        .iter()
                        .filter(|session| session.name == saved)
                        .chain(sessions.iter().filter(|session| session.name != saved))
                        .map(|session| {
                            (
                                session.name.clone(),
                                state_word(session.running),
                                session.running,
                                attached && session.name == saved,
                            )
                        }),
                );
                (rows, None)
            }
            Some(DeviceScan::Sessions(_)) => (
                vec![fallback()],
                Some(fl!("sessions-menu-no-device-sessions")),
            ),
            Some(DeviceScan::Failed(error)) => (vec![fallback()], Some(error.clone())),
            // A disabled device is never dialled, so it must not claim to be
            // waiting for an answer that is not coming.
            None if !endpoint.enabled() => (vec![fallback()], Some(fl!("sessions-menu-disabled"))),
            None => (vec![fallback()], Some(fl!("sessions-menu-checking"))),
        };
        for (name, detail, online, checked) in rows {
            entries.push(Entry::Row {
                row: Row::Device {
                    id: endpoint.id.clone(),
                    session: name.clone(),
                },
                spec: Spec {
                    label: name.into(),
                    detail: detail.into(),
                    online,
                    checked,
                    enabled: endpoint.enabled(),
                },
            });
        }
        if let Some(note) = note {
            entries.push(Entry::Muted {
                selector: format!("sessions-device-note-{}", endpoint.id),
                text: note,
            });
        }
        if let Some(host) = endpoint.connection.target.remote_host() {
            entries.push(self.add_session_entry(Target::Device {
                id: endpoint.id.clone(),
                host,
            }));
        }
    }

    /// This machine's sessions, with the notes that explain an empty list and what
    /// selecting a stopped session does.
    fn local_entries(&self, entries: &mut Vec<Entry>) {
        if self.sessions.entries.is_empty() {
            entries.push(Entry::Muted {
                selector: "sessions-empty".to_owned(),
                text: if self.sessions.scanning {
                    fl!("sessions-menu-checking")
                } else {
                    fl!("sessions-menu-no-local-sessions")
                },
            });
        }
        // A stopped session is still worth selecting: it starts its daemon, the
        // same way `herdr --session <name>` does.
        if self
            .sessions
            .entries
            .iter()
            .any(|session| !session.state.running())
        {
            entries.push(Entry::Muted {
                selector: "sessions-stopped".to_owned(),
                text: fl!("sessions-menu-stopped-note"),
            });
        }
        entries.extend(self.sessions.entries.iter().map(|session| Entry::Row {
            row: Row::Local(session.name.clone()),
            spec: self.local_session_spec(session),
        }));
        entries.push(self.add_session_entry(Target::Local));
    }

    fn local_session_spec(&self, session: &LocalSession) -> Spec {
        Spec {
            label: session.name.clone().into(),
            detail: state_word(session.state.running()).into(),
            online: session.state.running(),
            checked: self
                .local_socket()
                .is_some_and(|socket| socket == session.socket),
            enabled: true,
        }
    }

    /// One session row under its device. `row` counts the session rows painted
    /// so far and names the row's selectors; `ordinal` is its place among the
    /// picker's selectable lines, which hover and the keys address.
    /// Attach to a row's session. The row is what the caller painted, not an
    /// index into a list computed again here: a device list that refreshed in
    /// between would otherwise have the click name a different session.
    pub(super) fn choose_session(&mut self, row: Row, window: &mut Window, cx: &mut Context<Self>) {
        if self.sessions.mutation.is_some() {
            return;
        }
        match row {
            Row::Add(target) => self.open_session_create(target, window, cx),
            Row::Local(name) => {
                self.dismiss_menu(window, cx);
                self.select_local_session(&name, cx);
                // The picker's own follow-up: the sidebar follows the host that
                // was just chosen, and another session's tree is not this one's.
                self.set_device_scope(devices::DeviceScope::only(self.endpoints[0].id.clone()));
            }
            // A device row is one of that device's sessions, so it retargets the
            // device rather than adding an endpoint for every session it runs.
            Row::Device { id, session } => {
                // A disabled device has nothing to attach to, and the picker
                // refuses the same click. Filtering the sidebar to it would leave
                // the host list empty, so the menu stays as it is.
                if !self
                    .endpoints
                    .iter()
                    .any(|endpoint| endpoint.id == id && endpoint.enabled())
                {
                    return;
                }
                self.dismiss_menu(window, cx);
                self.select_device_session(&id, &session, cx);
                // The local branch's own follow-up: the sidebar follows the host
                // that was just chosen, and another session's tree is not this
                // one's.
                self.set_device_scope(devices::DeviceScope::only(id));
            }
        }
    }

    /// Keys for the open Add Session or Delete Session form.
    pub(super) fn sessions_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(&self.menu.session_edit, Some(Edit::Create { input, .. })
            if input.read(cx).is_composing() || !matches!(event.keystroke.key.as_str(), "enter" | "escape"))
        {
            return;
        }
        cx.stop_propagation();
        window.prevent_default();
        self.session_edit_key(event, window, cx);
    }
}

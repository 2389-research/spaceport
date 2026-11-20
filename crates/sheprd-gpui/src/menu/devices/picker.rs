// ABOUTME: Builds device picker rows and maintains their keyboard selection.
// ABOUTME: Shows Core setup only when the running session enables Core.
// i18n: migrated
//! The device picker's list: All Devices, each device with the session it is
//! on, the sessions of the one device unfolded beneath it, and Add Device.
//! Selection is an ordinal over the lines that can be chosen, so with nothing
//! unfolded it is the device row's own index.

use super::super::sessions::{Entry, Row, target_session};
use super::super::{Page, colors};
use super::{DeviceScope, SetupUnavailable, device_list_height, multi_select_hint};
use crate::SheprdWindow;
use crate::i18n::fl;
use gpui::{prelude::*, *};
use sheprd_client::ConnectTarget;

use super::rail::add_device_offer;

mod paint;
mod render;
pub(in crate::menu::devices) use paint::PickerPaint;

/// A line of the picker, in paint order.
pub(in crate::menu) enum Line {
    All,
    /// A device, by its index in the window's endpoints.
    Device(usize),
    /// A session row or note under the unfolded device.
    Session(Entry),
    AddDevice,
}

/// A device's sessions closing after it folded: drawn under its row,
/// shrinking from where they were, until [`crate::motion::LEAVE`] passes.
pub(in crate::menu) struct Folding {
    pub(super) device: String,
    pub(super) since: std::time::Instant,
    /// Their height when they began to close.
    height: Pixels,
    /// How open they were then, from 0 to 1.
    pub(super) from: f32,
}

impl Folding {
    /// How open they still are at `now`, or `None` once closed.
    fn shown(&self, now: std::time::Instant) -> Option<f32> {
        crate::motion::progress(
            self.since,
            now,
            crate::motion::duration(crate::motion::LEAVE),
        )
        .map(|k| self.from * (1. - k))
    }
}

/// What a selectable line is, apart from where it sits: the keys stay on it
/// while rows arrive above it.
#[derive(PartialEq)]
enum Pick {
    All,
    Device(String),
    Session(Row),
    AddDevice,
}

impl Line {
    fn selectable(&self) -> bool {
        match self {
            Self::Session(entry) => entry.row().is_some(),
            _ => true,
        }
    }
}

impl SheprdWindow {
    /// Every line of the picker, in paint order.
    pub(in crate::menu) fn picker_lines(&self) -> Vec<Line> {
        let mut lines = vec![Line::All];
        for (index, endpoint) in self.endpoints.iter().enumerate() {
            lines.push(Line::Device(index));
            if self.menu.unfolded_device.as_deref() == Some(endpoint.id.as_str()) {
                lines.extend(
                    self.device_session_entries(index)
                        .into_iter()
                        .map(Line::Session),
                );
            }
        }
        if self.core_enabled() {
            lines.push(Line::AddDevice);
        }
        lines
    }

    fn pick(&self, line: &Line) -> Option<Pick> {
        Some(match line {
            Line::All => Pick::All,
            Line::Device(index) => Pick::Device(self.endpoints.get(*index)?.id.clone()),
            Line::Session(Entry::Row { row, .. }) => Pick::Session(row.clone()),
            Line::Session(_) => return None,
            Line::AddDevice => Pick::AddDevice,
        })
    }

    /// Runs `update`, which may add or take away the picker's rows, and when
    /// it reports a change keeps the keys on the line they were on. A line
    /// that went away leaves nothing selected rather than its neighbour.
    pub(crate) fn keep_picker_selection(&mut self, update: impl FnOnce(&mut Self) -> bool) -> bool {
        let before = self
            .menu
            .selected
            .filter(|_| self.menu.page == Some(Page::Devices))
            .and_then(|ordinal| {
                let lines = self.picker_lines();
                let line = lines.iter().filter(|line| line.selectable()).nth(ordinal)?;
                self.pick(line)
            });
        let changed = update(self);
        if changed && let Some(before) = before {
            self.menu.selected =
                self.picker_ordinal(|line| self.pick(line).as_ref() == Some(&before));
        }
        changed
    }

    /// The ordinal of the first selectable line `pick` accepts.
    pub(in crate::menu) fn picker_ordinal(&self, pick: impl Fn(&Line) -> bool) -> Option<usize> {
        self.picker_lines()
            .iter()
            .filter(|line| line.selectable())
            .position(pick)
    }

    /// Unfolds device `index` to its sessions, folding any other, and selects
    /// its row. Unfolding is what asks the device for its sessions again.
    pub(in crate::menu) fn unfold_device(&mut self, index: usize) {
        let Some(id) = self
            .endpoints
            .get(index)
            .map(|endpoint| endpoint.id.clone())
        else {
            return;
        };
        let now = crate::motion::now();
        // Sessions still closing reopen from where they are.
        let from = self
            .menu
            .folding
            .take()
            .filter(|folding| folding.device == id)
            .and_then(|folding| folding.shown(now))
            .unwrap_or(0.);
        if self.menu.unfolded_device.as_ref() != Some(&id) {
            self.start_folding(now);
        }
        self.menu.unfolded_device = Some(id);
        self.menu.unfold_from = from;
        // The list from the last unfold paints at once, so it is never blank
        // while the new scan runs.
        self.sessions.refresh();
        self.select_device_line(index);
        // Bring the unfolded lines into view, which can fall below the list's
        // bottom. They are one child, where the first of them is; the heading
        // is the list's first child.
        let first = self
            .picker_lines()
            .iter()
            .position(|line| matches!(line, Line::Session(_)));
        self.menu.devices_reveal.set(first.map(|first| first + 1));
        self.menu.unfolded_since = Some(now);
    }

    pub(super) fn fold_device(&mut self, index: usize) {
        self.start_folding(crate::motion::now());
        self.select_device_line(index);
    }

    /// Folds the unfolded device, whose sessions close from however open
    /// they had come.
    fn start_folding(&mut self, now: std::time::Instant) {
        let Some(device) = self.menu.unfolded_device.take() else {
            return;
        };
        let opened = self
            .menu
            .unfolded_since
            .and_then(|since| {
                crate::motion::progress(since, now, crate::motion::duration(crate::motion::ENTER))
            })
            .unwrap_or(1.);
        let from = self.menu.unfold_from + (1. - self.menu.unfold_from) * opened;
        self.menu.folding = Some(Folding {
            device,
            since: now,
            height: self.menu.unfolded_height.get() * from,
            from,
        });
    }

    fn select_device_line(&mut self, index: usize) {
        self.menu.selected =
            self.picker_ordinal(|line| matches!(line, Line::Device(device) if *device == index));
    }

    fn toggle_unfolded(&mut self, index: usize, cx: &mut Context<Self>) {
        let unfolded = self
            .endpoints
            .get(index)
            .is_some_and(|endpoint| self.menu.unfolded_device.as_ref() == Some(&endpoint.id));
        if unfolded {
            self.fold_device(index);
        } else {
            self.unfold_device(index);
        }
        cx.notify();
    }

    /// The device index a picker line stands for in [`Self::choose_device`]:
    /// All Devices first, then each device, then Add Device.
    fn device_row(&self, line: &Line) -> Option<usize> {
        match line {
            Line::All => Some(0),
            Line::Device(index) => Some(index + 1),
            Line::AddDevice => Some(self.add_device_row()),
            Line::Session(_) => None,
        }
    }

    fn device_detail(target: &ConnectTarget) -> String {
        // Names: the host, machine, or distribution. The session is the chip
        // beside it.
        match target {
            ConnectTarget::Ssh { target, .. } => target.clone(),
            // i18n: data
            ConnectTarget::Wsl { distro, .. } => format!("WSL {distro}"),
            ConnectTarget::Socket(path) => path.display().to_string(),
            ConnectTarget::Session { .. } | ConnectTarget::Local => fl!("devices-this-device"),
        }
    }

    #[cfg(test)]
    fn picker_unfold_fraction(&self, now: std::time::Instant) -> f32 {
        if !self.menu.appear.live {
            return 1.;
        }
        crate::motion::progress(
            self.menu.unfolded_since.unwrap_or(now),
            now,
            crate::motion::duration(crate::motion::ENTER),
        )
        .map_or(1., |progress| {
            self.menu.unfold_from + (1. - self.menu.unfold_from) * progress
        })
    }

    fn picker_folding(&self, index: usize, now: std::time::Instant) -> Option<(&Folding, f32)> {
        #[cfg(test)]
        if !self.menu.appear.live {
            return None;
        }
        let folding = self.menu.folding.as_ref().filter(|folding| {
            self.endpoints
                .get(index)
                .is_some_and(|endpoint| endpoint.id == folding.device)
        })?;
        folding.shown(now)?;
        let shown = 1.
            - crate::motion::progress(
                folding.since,
                now,
                crate::motion::duration(crate::motion::LEAVE),
            )
            .unwrap_or(1.);
        Some((folding, shown))
    }

    /// The picker's rows after the devices: Add Device.
    pub(super) fn add_device_row(&self) -> usize {
        self.endpoints.len() + 1
    }

    fn choose_device(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index > self.add_device_row() {
            return;
        } else if index == self.add_device_row() {
            self.open_device_setup(window, cx);
        } else {
            let id = if index == 0 {
                None
            } else {
                let Some(endpoint) = self.endpoints.get(index - 1).filter(|e| e.enabled()) else {
                    return;
                };
                Some(endpoint.id.clone())
            };
            self.dismiss_menu(window, cx);
            if let Some(id) = &id
                && self.select_endpoint(id, cx).is_err()
            {
                return;
            }
            self.set_device_scope(id.map_or(DeviceScope::All, DeviceScope::only));
        }
        cx.notify();
    }

    /// Adds a device to the sidebar's list or takes it away, leaving the picker
    /// open to pick more. Taking away the device on screen selects the first
    /// one still listed, so the terminal always belongs to a listed device;
    /// like any switch of device, that closes the picker.
    fn toggle_listed_device(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        // All Devices and Add Device have nothing to add.
        let Some(endpoint) = index
            .checked_sub(1)
            .and_then(|index| self.endpoints.get(index))
        else {
            return self.choose_device(index, window, cx);
        };
        if !endpoint.enabled() {
            return;
        }
        let enabled = self
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.enabled())
            .map(|endpoint| endpoint.id.as_str());
        let Some(scope) = self.device_filter.toggled(
            &endpoint.id,
            &self.endpoints[self.selected_endpoint].id,
            enabled,
        ) else {
            return;
        };
        if !scope.shows(&self.endpoints[self.selected_endpoint].id) {
            let Some(next) = self
                .displayed_devices()
                .into_iter()
                .find(|(_, endpoint)| endpoint.enabled() && scope.shows(&endpoint.id))
                .map(|(_, endpoint)| endpoint.id.clone())
            else {
                return;
            };
            if self.select_endpoint(&next, cx).is_err() {
                return;
            }
        }
        self.set_device_scope(scope);
        cx.notify();
    }

    pub(in crate::menu) fn devices_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.menu.page == Some(Page::AddDevice) {
            if !self.add_device_key(event, window, cx) {
                return;
            }
        } else {
            self.picker_key(event.keystroke.key.as_str(), window, cx);
        }
        cx.stop_propagation();
        window.prevent_default();
    }

    fn picker_key(&mut self, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        let lines = self.picker_lines();
        // Each selectable line's child index in the list, past its heading.
        // The unfolded device's session lines share one child, the block
        // they open in.
        let mut child = 0;
        let mut in_block = false;
        let children: Vec<usize> = lines
            .iter()
            .filter_map(|line| {
                let session = matches!(line, Line::Session(_));
                if !(session && in_block) {
                    child += 1;
                }
                in_block = session;
                line.selectable().then_some(child)
            })
            .collect();
        let count = children.len();
        if matches!(key, "up" | "down") {
            let up = key == "up";
            let next = match self.menu.selected {
                Some(ordinal) => (ordinal.min(count - 1) + if up { count - 1 } else { 1 }) % count,
                None if up => count - 1,
                None => 0,
            };
            self.menu.selected = Some(next);
            self.reveal_picker_line(&lines, next, children.get(next).copied());
            cx.notify();
            return;
        }
        if key == "escape" {
            return self.dismiss_menu(window, cx);
        }
        // With nothing selected, as after a deleted row, no key acts on a line
        // that merely took its place.
        let Some(ordinal) = self.menu.selected else {
            return;
        };
        let Some(line) = lines.iter().filter(|line| line.selectable()).nth(ordinal) else {
            return;
        };
        // The device a session line sits under, for keys that act on it.
        let parent = lines
            .iter()
            .filter(|line| line.selectable())
            .take(ordinal + 1)
            .filter_map(|line| match line {
                Line::Device(device) => Some(*device),
                _ => None,
            })
            .last();
        match (key, line) {
            ("enter", Line::Session(Entry::Row { row, .. })) => {
                self.choose_session(row.clone(), window, cx);
            }
            ("backspace" | "delete", Line::Session(Entry::Row { row, .. })) => {
                if !matches!(row, Row::Add(_)) {
                    let target = self.management_target(row);
                    self.confirm_session_delete(row.clone(), target, window, cx);
                }
            }
            ("right", Line::Device(device)) => {
                // Already open, it stays as it is rather than opening again.
                let open = self.endpoints.get(*device).is_some_and(|endpoint| {
                    self.menu.unfolded_device.as_ref() == Some(&endpoint.id)
                });
                if !open {
                    self.unfold_device(*device);
                    cx.notify();
                }
            }
            ("left", Line::Device(_) | Line::Session(_)) => {
                if let Some(device) = parent.filter(|_| self.menu.unfolded_device.is_some()) {
                    self.fold_device(device);
                    cx.notify();
                }
            }
            ("enter", line) => {
                if let Some(index) = self.device_row(line) {
                    self.choose_device(index, window, cx);
                }
            }
            ("space", line) => {
                if let Some(index) = self.device_row(line) {
                    self.toggle_listed_device(index, window, cx);
                }
            }
            _ => {}
        }
    }
}

impl SheprdWindow {
    /// Scrolls the selectable line at `ordinal` into view. A session shares
    /// one child of the list with the rest of its device's block, so it is
    /// brought in by where it last painted; anything else by its `child`.
    fn reveal_picker_line(&self, lines: &[Line], ordinal: usize, child: Option<usize>) {
        let scroll = &self.menu.devices_scroll;
        let Some(index) = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.selectable())
            .nth(ordinal)
            .map(|(index, _)| index)
        else {
            return;
        };
        let session = matches!(lines[index], Line::Session(_)).then(|| {
            lines[..index]
                .iter()
                .filter(|line| matches!(line, Line::Session(_)))
                .count()
        });
        let row = session.and_then(|row| self.menu.unfolded_rows.borrow().get(row).copied());
        match row {
            Some(row) => {
                let view = scroll.bounds();
                let mut offset = scroll.offset();
                if row.top() < view.top() {
                    offset.y += view.top() - row.top();
                } else if row.bottom() > view.bottom() {
                    offset.y -= row.bottom() - view.bottom();
                }
                scroll.set_offset(offset);
            }
            None => {
                if let Some(child) = child {
                    scroll.scroll_to_item(child);
                }
            }
        }
    }
}

/// Sets a line under the unfolded device in from its row, with a rule that
/// ties it to the device above.
fn session_indent(theme: &crate::config::Theme) -> Div {
    div()
        .flex_none()
        .ml(px(12.))
        .pl(px(6.))
        .border_l_1()
        .border_color(rgb(theme.active))
}

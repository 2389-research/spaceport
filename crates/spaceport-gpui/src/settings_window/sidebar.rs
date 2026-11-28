// ABOUTME: Keeps live sidebar drafts and their touched preference keys.
// ABOUTME: Serializes committed drafts without replacing newer previews.
//! Live sidebar spacing, selection and host colours: previews reach every main
//! window at once, and commits write through Settings' serial save path.
// i18n: migrated
use super::{SettingsWindow, ui};
use crate::{
    config::{Config, preferences::Preference},
    search_input::SearchInput,
};
use gpui::{prelude::*, *};
pub(in crate::settings_window) use spaceport_settings::sidebar_draft::{
    SidebarIntent, SidebarTouched, diff_preferences, touched_preferences,
};

/// The host colour being typed as hex, for the host it was opened on: rows
/// can reorder while it is open.
pub(super) struct HexEdit {
    pub host: String,
    pub input: Entity<SearchInput>,
    pub invalid: bool,
    pub _blur: Subscription,
}

impl SettingsWindow {
    pub(super) fn sidebar_dirty(&self) -> bool {
        self.sidebar_intent.is_some()
    }

    /// Live, unsaved: updates the draft global and every SpaceportWindow.
    pub(super) fn preview_sidebar(&mut self, mut intent: SidebarIntent, cx: &mut Context<Self>) {
        if self.persistence.quitting() || self.persistence.closing() {
            return;
        }
        // A key stays touched for the draft's life, even dragged back.
        let changed = SidebarTouched::between(&self.config.sidebar_style, &intent.style);
        intent.touched.extend(&changed);
        if let Some(live) = &self.sidebar_intent {
            intent.touched.extend(&live.touched);
        }
        intent.apply(&mut self.config);
        super::drafts::drafts_mut(cx).edit_sidebar(intent.clone());
        self.sidebar_intent = Some(intent);
        self.broadcast_sidebar(cx);
    }

    /// Drops the live draft unwritten and shows the loaded values again.
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    pub(super) fn discard_sidebar_draft(&mut self, cx: &mut Context<Self>) {
        self.sidebar_intent = None;
        self.persistence.queue_sidebar(None);
        super::drafts::drafts_mut(cx).discard_sidebar();
        SidebarIntent::of(&self.loaded_config).apply(&mut self.config);
        self.broadcast_sidebar(cx);
    }

    /// Live and queued for save: preview, then write every changed key
    /// through one serial save.
    pub(super) fn commit_sidebar(&mut self, intent: SidebarIntent, cx: &mut Context<Self>) {
        if self.persistence.quitting() || self.persistence.closing() {
            return;
        }
        self.preview_sidebar(intent, cx);
        self.persistence.queue_sidebar(self.sidebar_intent.clone());
        self.save_sidebar_draft(cx);
    }

    pub(super) fn broadcast_sidebar(&self, cx: &mut Context<Self>) {
        let intent = SidebarIntent::of(&self.config);
        for handle in cx.windows() {
            if let Some(handle) = handle.downcast::<crate::SpaceportWindow>() {
                let _ = handle.update(cx, |view, _, cx| {
                    intent.apply(&mut view.config);
                    cx.notify();
                });
            }
        }
        // Windows opened later start from the draft too.
        let mut appearance = cx
            .try_global::<crate::app::InitialAppearance>()
            .cloned()
            .unwrap_or_default();
        intent.apply(&mut appearance.shown.config);
        cx.set_global(appearance);
        cx.notify();
    }

    fn sidebar_operation(
        &self,
        intent: &SidebarIntent,
    ) -> impl FnOnce() -> crate::Result<()> + Send + 'static + use<> {
        self.sidebar_write(diff_preferences(&self.loaded_config, intent))
    }

    fn sidebar_write(
        &self,
        edits: Vec<Preference>,
    ) -> impl FnOnce() -> crate::Result<()> + Send + 'static + use<> {
        #[cfg(test)]
        let io = self.controls.preference_io.clone();
        move || {
            #[cfg(test)]
            if let Some(io) = io {
                return edits.into_iter().try_for_each(|edit| (io.write)(edit));
            }
            edits
                .into_iter()
                .try_for_each(|edit| Config::save_preference(edit).map_err(crate::Error::from))
        }
    }

    /// Writes the draft unless another save is running; that save's
    /// completion resumes a queued draft through `finish_close`.
    pub(super) fn save_sidebar_draft(&mut self, cx: &mut Context<Self>) {
        let Some(intent) = self.sidebar_intent.clone() else {
            self.persistence.queue_sidebar(None);
            return;
        };
        if self.busy() {
            return;
        }
        self.persistence.queue_sidebar(None);
        self.save_sidebar_intent(intent, cx);
    }

    pub(super) fn save_queued_sidebar(&mut self, cx: &mut Context<Self>) {
        if let Some(intent) = self.persistence.take_sidebar() {
            self.save_sidebar_intent(intent, cx);
        }
    }

    fn save_sidebar_intent(&mut self, intent: SidebarIntent, cx: &mut Context<Self>) {
        if diff_preferences(&self.loaded_config, &intent).is_empty() {
            // Back where the file already is: nothing to write.
            self.finish_sidebar_save(Some(intent), true, cx);
            self.finish_close(cx);
            cx.notify();
            return;
        }
        let operation = self.sidebar_operation(&intent);
        #[cfg(test)]
        if let Some(io) = self.controls.preference_io.clone() {
            self.save_kind(
                operation,
                move || (io.load)(),
                false,
                super::Kind::Sidebar(intent),
                cx,
            );
            return;
        }
        self.save_kind(
            operation,
            self.loader(cx),
            false,
            super::Kind::Sidebar(intent),
            cx,
        );
    }

    /// Called by the save completion with the intent that save wrote. A newer
    /// draft made during the write stays live and queued.
    pub(super) fn finish_sidebar_save(
        &mut self,
        saved: Option<SidebarIntent>,
        ok: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(saved) = saved else {
            return false;
        };
        if !ok {
            return false;
        }
        // A newer draft stays live. Only its captured commit writes it;
        // a preview made after that commit must remain unwritten.
        if self.sidebar_intent.as_ref() == Some(&saved) {
            self.sidebar_intent = None;
            super::drafts::drafts_mut(cx).clear_sidebar();
        }
        true
    }

    /// The write a quit still owes, unless the running save already
    /// writes this very draft. The file may hold the running save's values
    /// by then, so the quit writes every touched key rather than a diff
    /// against the last load: a revert to the loaded value must still land.
    /// A draft's touched keys include every earlier version's, so a host a
    /// running save added and the draft dropped is cleared too.
    pub(super) fn take_shutdown_sidebar(
        &mut self,
    ) -> Option<impl FnOnce() -> crate::Result<()> + Send + 'static + use<>> {
        let intent = self.sidebar_intent.take()?;
        if matches!(self.persistence.kind(), Some(super::Kind::Sidebar(saved)) if saved == &intent)
        {
            return None;
        }
        Some(self.sidebar_write(touched_preferences(&intent)))
    }

    /// After a load: the live draft keeps only its touched keys over what
    /// was read, and `config` shows the result.
    pub(super) fn rebase_sidebar_draft(&mut self) {
        if let Some(intent) = &mut self.sidebar_intent {
            intent.rebase(&self.config);
            intent.overlay(&mut self.config);
        }
    }

    /// One row per host: connected endpoints first in window order, then
    /// configured hosts that are not connected, alphabetically. Each names
    /// the message hinting at its state, or is empty.
    pub(super) fn host_rows(&self) -> Vec<(String, Option<u32>, &'static str)> {
        let hosts = &self.config.sidebar_style.hosts;
        let mut rows: Vec<(String, Option<u32>, &'static str)> = Vec::new();
        for host in &self.source_window.hosts {
            if rows.iter().any(|(label, ..)| *label == host.label) {
                continue;
            }
            let color = hosts.get(&host.label).copied();
            let hint = if color.is_some() {
                ""
            } else {
                "settings-sidebar-host-uncoloured"
            };
            rows.push((host.label.clone(), color, hint));
        }
        // The map is sorted, so configured-only hosts come alphabetically.
        for (host, color) in hosts {
            if !rows.iter().any(|(label, ..)| label == host) {
                rows.push((host.clone(), Some(*color), "settings-sidebar-host-offline"));
            }
        }
        rows
    }

    pub(super) fn set_host_color(
        &mut self,
        host: &str,
        color: Option<u32>,
        cx: &mut Context<Self>,
    ) {
        let mut intent = SidebarIntent::of(&self.config);
        match color {
            Some(color) => intent.style.hosts.insert(host.to_owned(), color),
            None => intent.style.hosts.remove(host),
        };
        self.commit_sidebar(intent, cx);
    }

    pub(super) fn begin_hex_edit(
        &mut self,
        host: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.persistence.quitting() || self.persistence.closing() {
            return;
        }
        if !self.finish_hex_edit(true, cx) {
            self.finish_hex_edit(false, cx);
        }
        let current = self
            .host_rows()
            .into_iter()
            .find(|(label, ..)| label == host)
            .and_then(|(_, color, _)| color)
            .map(|color| format!("{color:06x}"))
            .unwrap_or_default();
        let input = cx.new(SearchInput::new);
        input.update(cx, |input, cx| {
            input.set_text_selected(&current, cx);
            input.set_appearance(self.config.terminal.clone(), self.theme.clone(), cx);
        });
        let focus = input.read(cx).focus.clone();
        // Like Enter: an invalid colour keeps the field open and marked;
        // only Escape discards it.
        let blur = cx.on_blur(&focus, window, |this, _, cx| {
            this.finish_hex_edit(true, cx);
        });
        self.hex_edit = Some(HexEdit {
            host: host.to_owned(),
            input,
            invalid: false,
            _blur: blur,
        });
        window.focus(&focus, cx);
        cx.notify();
    }

    /// Commits or abandons the typed colour. False means the text is not a
    /// colour (or is mid-composition) and the field stays open, marked.
    pub(super) fn finish_hex_edit(&mut self, save: bool, cx: &mut Context<Self>) -> bool {
        let Some(edit) = &mut self.hex_edit else {
            return true;
        };
        let color = if save {
            if edit.input.read(cx).is_composing() {
                return false;
            }
            let Some(color) = ui::parse_hex_input(edit.input.read(cx).text()) else {
                edit.invalid = true;
                cx.notify();
                return false;
            };
            Some(color)
        } else {
            None
        };
        let host = std::mem::take(&mut edit.host);
        self.hex_edit = None;
        if let Some(color) = color {
            self.set_host_color(&host, Some(color), cx);
        }
        cx.notify();
        true
    }
}

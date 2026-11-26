// ABOUTME: Previews layout choices and persists captured layout intents.
// ABOUTME: Fences reloads so later choices survive an earlier write.
//! Live layout choice, committed through Settings' serial save path on close.
// i18n: migrated
use super::SettingsWindow;
use crate::config::{Config, LayoutMode};
use gpui::Context;

#[cfg(test)]
#[derive(Clone)]
pub(super) struct LayoutIo {
    pub write: std::sync::Arc<dyn Fn(LayoutMode) -> crate::Result<()> + Send + Sync>,
    pub load: std::sync::Arc<dyn Fn() -> crate::Result<super::Loaded> + Send + Sync>,
}

impl SettingsWindow {
    pub(super) fn accept_layout_choice(&mut self, mode: LayoutMode, cx: &mut Context<Self>) {
        if self.persistence.quitting()
            || self.persistence.closing()
            || self.config.layout.mode == mode
        {
            return;
        }
        self.layout_intent = Some(mode);
        self.config.layout.mode = mode;
        super::drafts::drafts_mut(cx).begin_layout(mode);
        self.broadcast_layout(cx);
    }

    pub(super) fn broadcast_layout(&self, cx: &mut Context<Self>) {
        for handle in cx.windows() {
            if let Some(handle) = handle.downcast::<crate::SpaceportWindow>() {
                let _ = handle.update(cx, |view, _, cx| {
                    view.config.layout.mode = self.config.layout.mode;
                    cx.notify();
                });
            }
        }
        let mut appearance = cx
            .try_global::<crate::app::InitialAppearance>()
            .cloned()
            .unwrap_or_default();
        appearance.shown.config.layout.mode = self.config.layout.mode;
        cx.set_global(appearance);
        crate::menus::install(cx);
        cx.notify();
    }

    pub(super) fn layout_operation(
        &self,
        mode: LayoutMode,
    ) -> impl FnOnce() -> crate::Result<()> + Send + 'static + use<> {
        #[cfg(test)]
        let io = self.layout_io.clone();
        move || {
            #[cfg(test)]
            if let Some(io) = io {
                return (io.write)(mode);
            }
            Config::save_layout(mode).map_err(crate::Error::from)
        }
    }

    pub(super) fn save_layout_draft(&mut self, cx: &mut Context<Self>) {
        let Some(mode) = self.layout_intent else {
            return;
        };
        let operation = self.layout_operation(mode);
        if self.busy() {
            return;
        }
        #[cfg(test)]
        if let Some(io) = self.layout_io.clone() {
            self.save_kind(
                operation,
                move || (io.load)(),
                false,
                super::Kind::Layout(mode),
                cx,
            );
            return;
        }
        self.save_kind(
            operation,
            self.loader(cx),
            false,
            super::Kind::Layout(mode),
            cx,
        );
    }
}

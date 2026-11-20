// ABOUTME: Presents runtime release notes and acknowledges their first dismissal.
// ABOUTME: Captures bounded display data and scroll position for silent departure.
//! The daemon's update notice and release notes. Closing the sheet tells the
//! daemon the notes were read, so its other clients agree. An install command
//! is shown for the user to review and run; nothing here runs it.
// i18n: migrated
use super::{Page, chrome::Item};
use crate::i18n::fl;
use crate::{SheprdWindow, fonts::StyledFont, notifications::safe_text};
use gpui::{prelude::*, *};
use sheprd_client::Method;
use std::rc::Rc;

const MAX_BODY_HEIGHT: f32 = 360.;
const MAX_COMMAND: usize = 1024;

pub(in crate::menu) struct Paint {
    title: String,
    command: Option<String>,
    notes_caption: Option<String>,
    lines: Option<Rc<[crate::release_notes::Line]>>,
    offset: Point<Pixels>,
}

impl SheprdWindow {
    /// The menu entry for the sheet, when the daemon has something for it.
    pub(super) fn release_notes_item(&self) -> Option<Item> {
        let snapshot = self.live.snapshot.as_deref()?;
        if snapshot.update_available.is_some() {
            Some(Item::UpdateReady)
        } else if snapshot.release_notes.is_some() {
            Some(Item::WhatsNew)
        } else {
            None
        }
    }

    pub(super) fn render_release_notes(&self, cx: &mut Context<Self>) -> Div {
        self.render_release_notes_content(&self.release_notes_paint(), true, cx)
    }

    pub(in crate::menu) fn release_notes_paint(&self) -> Paint {
        let snapshot = self.live.snapshot.as_deref();
        let update = snapshot.and_then(|s| s.update_available.as_deref());
        let notes = snapshot.and_then(|s| s.release_notes.as_ref());
        let title = match (update, notes) {
            (Some(version), _) => fl!(
                "whats-new-update-ready",
                runtime = crate::constants::HERDR_NAME,
                version = safe_text(version, 64)
            ),
            (None, Some(notes)) => fl!(
                "whats-new-title",
                runtime = crate::constants::HERDR_NAME,
                version = safe_text(&notes.version, 64)
            ),
            (None, None) => fl!("whats-new-unavailable"),
        };
        let command = update.map(|_| {
            snapshot
                .map(|s| s.update_install_command.trim())
                .filter(|command| !command.is_empty())
                .map_or_else(
                    || fl!("whats-new-no-command"),
                    |command| safe_text(command, MAX_COMMAND),
                )
        });
        Paint {
            title,
            command,
            notes_caption: notes.filter(|_| update.is_some()).map(|notes| {
                fl!(
                    "whats-new-notes-for",
                    version = safe_text(&notes.version, 64)
                )
            }),
            lines: notes.map(|notes| self.daemon_text.release_notes.lines(&notes.body)),
            offset: self.menu.release_notes_scroll.offset(),
        }
    }

    pub(in crate::menu) fn render_release_notes_content(
        &self,
        paint: &Paint,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let mut sheet = div()
            .debug_selector(|| "release-notes".into())
            .flex()
            .flex_col()
            .child(
                div()
                    .debug_selector(|| "release-notes-title".into())
                    .p(px(8.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(paint.title.clone()),
            );
        if let Some(command) = &paint.command {
            sheet = sheet
                .child(div().p(px(8.)).child(fl!("whats-new-suggested-command")))
                .child(
                    div()
                        .debug_selector(|| "release-notes-command".into())
                        .mx(px(8.))
                        .p(px(8.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .bg(rgb(theme.active))
                        .text_font(&self.config.terminal)
                        .child(command.clone()),
                )
                .child(
                    div()
                        .p(px(8.))
                        .text_color(rgb(theme.subtext()))
                        .child(fl!("whats-new-nothing-runs")),
                );
        }
        if let Some(lines) = &paint.lines {
            let content = crate::release_notes::render_mode(
                "release-notes",
                lines,
                theme,
                &self.config.terminal,
                active,
            );
            sheet = sheet
                .when_some(paint.notes_caption.clone(), |sheet, caption| {
                    sheet.child(div().p(px(8.)).text_color(rgb(theme.muted)).child(caption))
                })
                .child(
                    div()
                        .id("release-notes-body")
                        .debug_selector(|| "release-notes-body".into())
                        .px(px(8.))
                        .py(px(4.))
                        .max_h(px(MAX_BODY_HEIGHT))
                        .when(active, |body| {
                            body.overflow_y_scroll()
                                .track_scroll(&self.menu.release_notes_scroll)
                        })
                        .when(!active, |body| body.overflow_hidden())
                        .child(
                            div()
                                .mt(if active { px(0.) } else { paint.offset.y })
                                .child(content),
                        ),
                );
        }
        sheet.child(
            div()
                .id("menu-close")
                .debug_selector(|| "release-notes-close".into())
                .p(px(8.))
                .child(fl!("whats-new-close"))
                .when(active, |button| {
                    button
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, window, cx| {
                            cx.stop_propagation();
                            this.dismiss_menu(window, cx);
                        }))
                }),
        )
    }

    /// Tells the daemon the shown release notes were read, as the sheet closes.
    pub(super) fn dismiss_release_notes(&mut self) {
        if self.menu.page != Some(Page::Update)
            || !self.menu_target_current()
            || !self.live.supports_release_notes_dismiss
        {
            return;
        }
        let Some(snapshot) = self.live.snapshot.as_deref() else {
            return;
        };
        let Some(notes) = snapshot.release_notes.as_ref() else {
            return;
        };
        let Some(handle) = &self.endpoints[self.selected_endpoint].connection.handle else {
            return;
        };
        if let Err(error) = handle.request(
            &snapshot.boot_id,
            Method::ReleaseNotesDismiss,
            serde_json::json!({ "version": notes.version }),
        ) {
            self.notices.show(
                crate::notice::Cause::Command,
                fl!("whats-new-dismiss-failed", error = error.to_string()),
            );
        }
    }
}

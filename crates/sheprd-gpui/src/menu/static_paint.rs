// ABOUTME: Captures display data for simple dialogs before live consent state resets.
// ABOUTME: Shares their templates while leaving action and input ownership behind.
use super::{Page, VersionNotice, whats_new};
use crate::{SheprdWindow, terminal::ExternalUrl};
use gpui::{AnyElement, App, Context, IntoElement};

pub(in crate::menu) enum StaticPaint {
    // keep-sorted start
    About,
    AgentSkill,
    OpenLink(ExternalUrl),
    Update(whats_new::Paint),
    VersionMismatch(VersionNotice),
    // keep-sorted end
}

#[cfg(test)]
mod tests;

impl SheprdWindow {
    pub(in crate::menu) fn static_dialog_paint(
        &self,
        page: Page,
        _cx: &App,
    ) -> Option<StaticPaint> {
        Some(match page {
            Page::About => StaticPaint::About,
            Page::AgentSkill => StaticPaint::AgentSkill,
            Page::VersionMismatch => {
                StaticPaint::VersionMismatch(self.menu.version_notice.clone()?)
            }
            Page::OpenLink => StaticPaint::OpenLink(self.menu.open_link.clone()?),
            Page::Update => StaticPaint::Update(self.release_notes_paint()),
            _ => return None,
        })
    }

    pub(in crate::menu) fn render_static_dialog_paint(
        &self,
        paint: &StaticPaint,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match paint {
            StaticPaint::About => self.render_about_mode(false, cx).into_any_element(),
            StaticPaint::AgentSkill => self
                .render_agent_skill_offer_mode(false, cx)
                .into_any_element(),
            StaticPaint::VersionMismatch(notice) => self
                .render_version_notice(notice, false, cx)
                .into_any_element(),
            StaticPaint::OpenLink(url) => {
                self.render_link_question(url, false, cx).into_any_element()
            }
            StaticPaint::Update(notes) => self
                .render_release_notes_content(notes, false, cx)
                .into_any_element(),
        }
    }
}

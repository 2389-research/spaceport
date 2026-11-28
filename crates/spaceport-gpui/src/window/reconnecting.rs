// ABOUTME: Renders outage, missing-runtime, and intentional-shutdown cards.
// ABOUTME: Offers explicit recovery while keeping daemon startup under endpoint ownership.
//! What the terminal area shows while the selected endpoint reconnects after a
//! drop: the last picture dimmed, and a card naming the host, why it dropped,
//! and a way to retry now. Ordinary outages keep retrying on their backoff.
//! Intentional shutdown waits for explicit recovery. The window stays on its
//! endpoint; nothing typed meanwhile is sent. When this
//! computer has no Herdr to start, the card says so instead and links
//! Herdr's TUI page.
// i18n: migrated
use super::SpaceportWindow;
use crate::{i18n::fl, notifications::safe_text};
use gpui::{prelude::*, *};

const MAX_WIDTH: f32 = 420.;
/// Where Install Herdr leads: the page for the Herdr this app attaches to,
/// the TUI, not Herdr Core. Opening it is all the button does.
const INSTALL_URL: &str = "https://herdr.dev/tui/";
const MAX_REASON: usize = 240;
/// How much of the stale picture shows through.
const DIM_ALPHA: u32 = 0xa0;

impl SpaceportWindow {
    /// Whether the legacy connection card shows. Core groups report their
    /// own connection and document state; a legacy outage cannot cover them.
    /// A replacement legacy snapshot keeps this card over the old picture
    /// until its first frame lands.
    pub(crate) fn reconnecting(&self) -> bool {
        if self.selected_core_group().is_some() {
            return false;
        }
        self.endpoints[self.selected_endpoint].outage().is_some() || self.presentation.stale()
    }

    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 119 lines")]
    pub(super) fn render_reconnecting(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.reconnecting() {
            return None;
        }
        let endpoint = &self.endpoints[self.selected_endpoint];
        // A device going away drops on purpose: say so, and offer no retry.
        let kind = if self.device_removing(endpoint) {
            Card::Removing
        } else if endpoint.herdr_missing() {
            Card::HerdrMissing
        } else if endpoint.herdr_stopped() {
            Card::HerdrStopped
        } else {
            Card::Reconnecting
        };
        let host = safe_text(&endpoint.label, 80);
        let (id, title, reason): (&'static str, _, _) = match kind {
            Card::Reconnecting => (
                "reconnecting",
                fl!("window-reconnecting", host = host),
                endpoint
                    .outage()
                    .map(|reason| ("reconnecting-reason", reason.to_owned())),
            ),
            Card::Removing => (
                "reconnecting",
                fl!("window-removing", host = host),
                Some(("reconnecting-reason", fl!("window-device-being-removed"))),
            ),
            // The status bar keeps the technical reason; the card says what to do.
            Card::HerdrMissing => (
                "herdr-missing",
                fl!("window-herdr-missing"),
                Some(("herdr-missing-note", fl!("window-herdr-missing-note"))),
            ),
            Card::HerdrStopped => (
                "herdr-stopped",
                fl!("window-herdr-stopped"),
                Some(("herdr-stopped-note", fl!("window-herdr-stopped-note"))),
            ),
        };
        let theme = &self.theme;
        let retry = |id, label| {
            card_button(id, label, theme).on_click(cx.listener(|this, _, _, cx| {
                cx.stop_propagation();
                this.reconnect(cx);
                cx.notify();
            }))
        };
        let buttons = match kind {
            Card::Reconnecting => Some(
                div()
                    .flex()
                    .child(retry("reconnect-now", fl!("window-reconnect-now"))),
            ),
            Card::Removing => None,
            Card::HerdrStopped => Some(
                div()
                    .flex()
                    .child(retry("start-herdr", fl!("window-start-herdr"))),
            ),
            // Opens the website only: Spaceport never downloads or runs Herdr's installer.
            Card::HerdrMissing => Some(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(
                        card_button("install-herdr", fl!("window-install-herdr"), theme).on_click(
                            |_, _, cx| {
                                cx.stop_propagation();
                                cx.open_url(INSTALL_URL);
                            },
                        ),
                    )
                    .child(retry("herdr-try-again", fl!("window-try-again"))),
            ),
        };
        let card = div()
            .id(id)
            .debug_selector(move || id.into())
            .min_w_0()
            .max_w(px(MAX_WIDTH))
            .occlude()
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(12.))
            .rounded(px(crate::config::corners::PANEL))
            .border_1()
            .border_color(rgb(theme.ink(theme.palette[3])))
            .bg(rgb(theme.surface))
            .text_color(rgb(theme.foreground))
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .truncate()
                    .child(title),
            )
            .children(reason.map(|(selector, reason)| {
                div()
                    .debug_selector(move || selector.into())
                    .text_color(rgb(theme.muted))
                    .child(safe_text(&reason, MAX_REASON))
            }))
            .children(buttons);
        Some(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .p(px(16.))
                .when(self.presentation.stale(), |overlay| {
                    overlay.bg(rgba((theme.background << 8) | DIM_ALPHA))
                })
                .child(card)
                .into_any_element(),
        )
    }
}

/// Which card shows over the endpoint's terminal area.
#[derive(Clone, Copy)]
enum Card {
    // keep-sorted start
    /// This computer has no Herdr to start: offer its website and a retry.
    HerdrMissing,
    /// An intentional stop requires an explicit request to start Herdr again.
    HerdrStopped,
    Reconnecting,
    /// The device is going away on purpose; nothing to retry.
    Removing,
    // keep-sorted end
}

/// A button on the card, named for screen readers by its visible label.
pub(super) fn card_button(
    id: &'static str,
    label: String,
    theme: &crate::config::Theme,
) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.into())
        .role(Role::Button)
        .aria_label(label.clone())
        .px(px(10.))
        .py(px(4.))
        .rounded(px(crate::config::corners::CONTROL))
        .border_1()
        .border_color(rgb(theme.active))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme.active)))
        .child(label)
}

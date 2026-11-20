// ABOUTME: Observes real native paste state without acquiring or changing clipboard data.
// ABOUTME: Mounts the standalone multiline field separately from chat's paste capture.
use crate::{SheprdWindow, i18n::Localize, multiline_input::MultilineInput};
use anyhow::Result;
use gpui::{
    AnyWindowHandle, AppContext as _, AsyncApp, Context, Entity, FocusHandle, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window, WindowHandle,
    accesskit::Role, div, px, rgb, size,
};
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

#[derive(Default)]
pub(super) struct Probe {
    previous: Option<(bool, bool, usize, &'static str)>,
}

impl Probe {
    pub(super) fn observe(&mut self, input: &MultilineInput) {
        #[cfg(windows)]
        let reader = crate::window::clipboard::reader_active();
        #[cfg(not(windows))]
        let reader = false;
        let error = match input.paste_error_text() {
            None => "none",
            Some(error) if error == crate::i18n::fl!("chat-prompt-too-long") => "prompt",
            Some(error)
                if error
                    == crate::window::clipboard::Error::Size {
                        limit: 2 * 1024 * 1024,
                    }
                    .localize() =>
            {
                "acquisition"
            }
            Some(_) => "other",
        };
        let state = (input.is_pasting(), reader, input.text().len(), error);
        if self.previous != Some(state) {
            eprintln!(
                "CLIPBOARD_STATE pending={} reader={} utf8_bytes={} error={}",
                state.0, state.1, state.2, state.3
            );
            self.previous = Some(state);
        }
    }
}

struct Field {
    input: Entity<MultilineInput>,
    other: FocusHandle,
}

impl Field {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            input: cx.new(|cx| {
                let mut input = MultilineInput::new(cx);
                input.set_label("Native standalone field", cx); // i18n: data
                input
            }),
            other: cx.focus_handle(),
        }
    }
}

impl Render for Field {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_4()
            .bg(rgb(0x101318))
            .child(self.input.clone())
            .child(
                div()
                    .id("clipboard-blur-target")
                    .role(Role::Button)
                    .aria_label("Native blur target") // i18n: data
                    .track_focus(&self.other)
                    .tab_index(0)
                    .child("Native blur target"), // i18n: data
            )
    }
}

pub(super) async fn standalone(
    handle: WindowHandle<SheprdWindow>,
    hold: Duration,
    cx: &mut AsyncApp,
) -> Result<()> {
    let handle: AnyWindowHandle = handle.into();
    let input = handle.update(cx, |_, window, cx| {
        window.set_window_title("Spaceport · Native clipboard field"); // i18n: data
        window.resize(size(px(900.), px(500.)));
        let field = window.replace_root(cx, |_, cx| Field::new(cx));
        field.read(cx).input.clone()
    })?;
    let timer = cx.background_executor().clone();
    let deadline = Instant::now() + hold;
    let mut probe = Probe::default();
    eprintln!("RICH CHAT READY case=clipboard-field"); // i18n: data
    while Instant::now() < deadline {
        timer.timer(Duration::from_millis(50)).await;
        handle.update(cx, |_, _, cx| probe.observe(input.read(cx)))?;
    }
    Ok(())
}

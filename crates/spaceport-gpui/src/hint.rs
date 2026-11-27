// ABOUTME: Draws the small tooltip the chrome's buttons and chips show on hover.
// ABOUTME: Takes its text and colors from the caller, so any bar can match it.
use gpui::{prelude::*, *};

/// A small tooltip in the colors of the bar it hangs from.
pub(crate) struct Hint {
    pub(crate) text: SharedString,
    pub(crate) foreground: u32,
    pub(crate) surface: u32,
}

impl Render for Hint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "hint".into())
            .px(px(8.))
            .py(px(4.))
            .rounded(px(crate::config::corners::CONTROL))
            .shadow_md()
            .text_size(px(12.))
            .text_color(rgb(self.foreground))
            .bg(rgb(self.surface))
            .child(self.text.clone())
    }
}

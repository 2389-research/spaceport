// ABOUTME: Captures only the visible device setup projection before disposable owners reset.
// ABOUTME: Routes passive Core, WSL, and removal bodies through their shared templates.
use super::{add_device, wsl};
use crate::{SheprdWindow, menu::Page};
use gpui::{App, Context, Div};

enum Content {
    // keep-sorted start
    Core(add_device::Paint),
    Remove(wsl::RemovalPaint),
    Wsl(Box<wsl::Paint>),
    // keep-sorted end
}

pub(in crate::menu) struct Paint(Content);

impl SheprdWindow {
    pub(in crate::menu) fn device_setup_paint(&self, cx: &App) -> Option<Paint> {
        let content = match self.menu.page? {
            Page::AddDevice => Content::Core(self.add_device_paint(cx)?),
            Page::AddWsl => Content::Wsl(Box::new(self.add_wsl_paint(true, cx)?)),
            Page::RemoveWsl => Content::Remove(self.remove_wsl_paint()?),
            _ => return None,
        };
        Some(Paint(content))
    }

    pub(in crate::menu) fn render_device_setup_paint(
        &self,
        paint: &Paint,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        match &paint.0 {
            Content::Core(paint) => self.render_add_device_paint(paint, compact, false, cx),
            Content::Remove(paint) => self.render_remove_wsl_paint(paint, false, cx),
            Content::Wsl(paint) => self.render_add_wsl_paint(paint, compact, false, cx),
        }
    }
}

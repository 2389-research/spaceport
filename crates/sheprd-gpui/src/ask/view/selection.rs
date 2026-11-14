// ABOUTME: Reuses native chat selection contracts for standalone Ask transcript lines.
// ABOUTME: Maps shaped glyph positions to exact prepared text without terminal input.
use super::*;
use crate::chat::{
    selection::{Caret, Cell, Key, Region},
    view::selection::{highlights, text_offset},
};
use sheprd_chat_protocol::EntryId;
use std::cell::RefCell;

#[derive(Clone, Default)]
pub(super) struct Geometry(Rc<RefCell<Vec<Hit>>>);

struct Hit {
    key: Key,
    bounds: Bounds<Pixels>,
    layout: TextLayout,
}

impl Geometry {
    pub(super) fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    pub(super) fn hit(&self, position: Point<Pixels>) -> Option<Caret> {
        let targets = self.0.borrow();
        let target = targets.iter().min_by(|left, right| {
            let distance = |target: &Hit| {
                let y = f32::from(position.y);
                (y - y.clamp(
                    f32::from(target.bounds.top()),
                    f32::from(target.bounds.bottom()),
                ))
                .abs()
            };
            distance(left).total_cmp(&distance(right))
        })?;
        Some(Caret {
            key: target.key,
            offset: text_offset(&target.layout, position),
        })
    }
}

impl View {
    pub(super) fn tick_selection_drag(&mut self, now: std::time::Instant) -> bool {
        if !self.selection.is_selecting() {
            return false;
        }
        let Some(point) = self.drag.point() else {
            return false;
        };
        let selected = self
            .geometry
            .hit(point)
            .is_some_and(|caret| self.selection.drag(self.prepared.text(), caret));
        self.drag.scroll(&self.list, now) || selected
    }

    pub(super) fn release_selection_drag(&mut self) {
        self.selection.release();
        self.drag.clear();
        self.drag_task = None;
    }

    fn start_selection_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.release_selection_drag();
        self.glide.user_scrolled();
        let timer = cx.background_executor().clone();
        self.drag_task = Some(cx.spawn_in(window, async move |this, cx| {
            loop {
                timer.timer(std::time::Duration::from_millis(16)).await;
                let active = this.update_in(cx, |this, window, cx| {
                    if !this.selection.is_selecting()
                        || !this.reader_focus.is_focused(window)
                        || !window.is_window_active()
                    {
                        this.release_selection_drag();
                        return false;
                    }
                    if this.tick_selection_drag(crate::motion::now()) {
                        cx.notify();
                    }
                    true
                });
                if !matches!(active, Ok(true)) {
                    let _ = this.update(cx, |this, _| this.release_selection_drag());
                    break;
                }
            }
        }));
    }

    pub(super) fn reading_line(
        &self,
        entry: u64,
        index: usize,
        line: &crate::release_notes::Line,
        cx: &mut Context<Self>,
    ) -> Div {
        let key = self
            .prepared
            .text()
            .key(EntryId(entry), Region::Body(0), Cell::row(index));
        let selected = key.and_then(|key| self.selection.highlight(self.prepared.text(), key));
        let marks = crate::release_notes::line_highlights(line, &self.theme, self.theme.primary());
        let text = StyledText::new(line.text.clone()).with_highlights(highlights(
            &line.text,
            marks,
            selected,
            &self.theme,
        ));
        let layout = text.layout().clone();
        let mut content = div().child(text);
        if let Some(key) = key {
            let geometry = self.geometry.clone();
            let measured = layout.clone();
            content = content
                .relative()
                .cursor(CursorStyle::IBeam)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                        this.start_selection_drag(window, cx);
                        let caret = Caret {
                            key,
                            offset: text_offset(&layout, event.position),
                        };
                        this.selection.press(
                            this.prepared.text(),
                            caret,
                            event.click_count,
                            event.modifiers.shift,
                        );
                        window.focus(&this.reader_focus, cx);
                        cx.stop_propagation();
                        cx.notify();
                    }),
                )
                .child(
                    canvas(
                        move |bounds, window, _| {
                            (
                                bounds.intersect(&window.content_mask().bounds),
                                measured.clone(),
                            )
                        },
                        move |_, (bounds, layout), _, _| {
                            if bounds.size.width > px(0.) && bounds.size.height > px(0.) {
                                geometry.0.borrow_mut().push(Hit {
                                    key,
                                    bounds,
                                    layout,
                                });
                            }
                        },
                    )
                    .absolute()
                    .inset_0(),
                );
        }
        crate::release_notes::line_with(
            "ask-message",
            index,
            line,
            &self.theme,
            &self.config.terminal,
            content.into_any_element(),
        )
        .debug_selector(move || format!("ask-line-{entry}-{index}"))
    }

    pub(super) fn reader_key(
        &mut self,
        event: &KeyDownEvent,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if !self.reader_focus.is_focused(window) {
            return;
        }
        let key = &event.keystroke;
        if (key.modifiers.platform || key.modifiers.control) && key.key == "c" {
            if let Some(text) = self.selection.text(self.prepared.text()) {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
            }
        } else if (key.modifiers.platform || key.modifiers.control) && key.key == "a" {
            self.selection.select_all(self.prepared.text());
            cx.notify();
        } else if key.key == "escape" {
            self.selection.clear();
            self.release_selection_drag();
            cx.notify();
        }
    }
}

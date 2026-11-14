// i18n: migrated
use gpui::{prelude::*, *};
use std::ops::Range;

use super::{SheprdWindow, input::ViewInputHandler};
use crate::caret::Carets;
use crate::text_editing::{
    EditCommand, Editing, KeyTable, Keys, Newlines, Platform, edit_command, from_utf16, to_utf16,
};

// Byte offsets internally; only the platform input boundary uses UTF-16.
#[derive(Default)]
pub(super) struct DialogInput {
    /// The draft's text, selection, and composition, read through `Deref`.
    edit: Editing,
    pub bounds: Bounds<Pixels>,
    /// Where the last drawn text puts each caret.
    carets: Option<Carets>,
    scroll: Pixels,
    /// What a screen reader calls the field, when a caption beside it
    /// names it rather than the field itself.
    pub label: Option<SharedString>,
}

/// One replacement inserts at most this many characters.
const MAX_INSERTED_CHARS: usize = 4096;
/// A replacement that would leave the draft longer than this is refused.
const MAX_BYTES: usize = 16384;

impl std::ops::Deref for DialogInput {
    type Target = Editing;

    fn deref(&self) -> &Editing {
        &self.edit
    }
}

impl std::ops::DerefMut for DialogInput {
    fn deref_mut(&mut self) -> &mut Editing {
        &mut self.edit
    }
}

impl DialogInput {
    pub fn new(text: String) -> Self {
        Self {
            edit: Editing {
                cursor: text.len(),
                text,
                ..Editing::default()
            },
            ..Self::default()
        }
    }

    pub fn range_from_utf16(&self, range: Range<usize>) -> Range<usize> {
        let start = from_utf16(&self.edit.text, range.start);
        start..from_utf16(&self.edit.text, range.end).max(start)
    }

    pub fn to_utf16(&self, range: Range<usize>) -> Range<usize> {
        to_utf16(&self.edit.text, range.start)..to_utf16(&self.edit.text, range.end)
    }

    /// Replaces UTF-16 `range`, else the composition, else the selection.
    /// A draft is one bounded line: control characters are dropped, and a
    /// replacement past the limits is cut short or refused.
    pub fn replace(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        marked: bool,
        selected: Option<Range<usize>>,
    ) {
        let range = range
            .map(|r| self.range_from_utf16(r))
            .unwrap_or_else(|| self.edit.replaced(None));
        let text: String = text
            .chars()
            .filter(|c| !c.is_control())
            .take(MAX_INSERTED_CHARS)
            .collect();
        if self.edit.text.len() - range.len() + text.len() > MAX_BYTES {
            return;
        }
        // An input method's backwards selection is its start alone.
        let selected = selected.map(|selected| selected.start..selected.end.max(selected.start));
        self.edit
            .replace_bytes(range, &text, marked, selected, Newlines::Strip);
        self.carets = None;
    }

    /// End a composition without changing its text.
    pub fn unmark(&mut self) {
        self.edit.marked = None;
    }

    pub fn key(&mut self, key: &Keystroke, cx: &mut App) -> bool {
        if self.edit.marked.is_some() {
            return false;
        }
        let keys = Keys {
            line: 0..self.edit.text.len(),
            words_hidden: false,
            table: KeyTable::Dialog,
            platform: Platform::CURRENT,
        };
        let Some(command) = edit_command(&key.key, key.modifiers, &self.edit, &keys) else {
            return false;
        };
        match command {
            EditCommand::Move { to, extend } => self.edit.select_to(to, extend),
            EditCommand::SelectAll => self.edit.select(0..self.edit.text.len()),
            EditCommand::Delete(range) => {
                self.edit.select(range);
                self.replace(None, "", false, None);
            }
            EditCommand::Copy | EditCommand::Cut => {
                let selection = self.edit.selection();
                if !selection.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        self.edit.text[selection].into(),
                    ));
                    if command == EditCommand::Cut {
                        self.replace(None, "", false, None);
                    }
                }
            }
            EditCommand::Paste => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.replace(None, &text, false, None);
                }
            }
            // The dialog table has no kill keys, and a draft no kill buffer.
            EditCommand::Kill(_) | EditCommand::Yank => return false,
        }
        true
    }

    pub fn index_at(&self, position: Point<Pixels>) -> Option<usize> {
        self.carets
            .as_ref()
            .map(|carets| carets.index_at(position.x - self.bounds.left() + self.scroll))
    }

    pub fn range_bounds(&self, range: Range<usize>) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(range);
        let extent = self.carets.as_ref()?.extent(range);
        let x = |x| {
            (self.bounds.left() + x - self.scroll).clamp(self.bounds.left(), self.bounds.right())
        };
        Some(Bounds::from_corners(
            point(x(extent.start), self.bounds.top()),
            point(x(extent.end), self.bounds.bottom()),
        ))
    }
}

impl SheprdWindow {
    pub(super) fn render_dialog_input(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let entity = cx.entity();
        let focus = self.menu.focus.clone();
        div()
            .id("dialog-input")
            .debug_selector(|| "dialog-input".into())
            .role(Role::TextInput)
            .aria_value(
                self.menu
                    .input
                    .as_ref()
                    .map(|input| SharedString::from(input.text.clone()))
                    .unwrap_or_default(),
            )
            .when_some(
                self.menu
                    .input
                    .as_ref()
                    .and_then(|input| input.label.clone()),
                |field, label| field.aria_label(label),
            )
            .w_full()
            .h(px(self.config.ui.line_height() + 10.))
            .px(px(6.))
            .py(px(5.))
            .rounded(px(crate::config::corners::CONTROL))
            .bg(rgb(self.theme.active))
            .overflow_hidden()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    // Another field of the same dialog may hold focus.
                    window.focus(&this.menu.focus, cx);
                    if let Some(input) = this.menu.input.as_mut()
                        && input.marked.is_none()
                        && let Some(index) = input.index_at(event.position)
                    {
                        input.select_to(index, event.modifiers.shift);
                        cx.notify();
                    }
                }),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        // Unfocused, the draft reads as plain text: no caret or
                        // selection that would look like it is about to be replaced.
                        let focused = focus.is_focused(window);
                        window.handle_input(
                            &focus,
                            ViewInputHandler::new(bounds, entity.clone()),
                            cx,
                        );
                        entity.update(cx, |this, cx| {
                            let Some(input) = this.menu.input.as_mut() else {
                                return;
                            };
                            let run = TextRun {
                                len: input.text.len(),
                                font: this.config.ui.font(),
                                color: rgb(this.theme.foreground).into(),
                                background_color: None,
                                underline: None,
                                strikethrough: None,
                            };
                            let line = window.text_system().shape_line(
                                input.text.clone().into(),
                                px(this.config.ui.size),
                                &[run],
                                None,
                            );
                            let carets = Carets::new(&input.text, &line);
                            let caret = carets.x(input.cursor);
                            input.scroll = input
                                .scroll
                                .min(caret)
                                .max(caret - bounds.size.width + px(2.))
                                .max(px(0.));
                            // Right to left, a line that fits sets against
                            // the field's right, its start.
                            let room = bounds.size.width - px(2.);
                            if crate::direction::is_rtl() && line.width < room {
                                input.scroll = line.width - room;
                            }
                            let origin = bounds.origin - point(input.scroll, px(0.));
                            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                                // Across a change of direction, a selection
                                // is a stretch on each side of it.
                                if focused && !input.selection().is_empty() {
                                    for span in carets.spans(input.selection()) {
                                        window.paint_quad(fill(
                                            Bounds::from_corners(
                                                point(origin.x + span.start, bounds.top()),
                                                point(origin.x + span.end, bounds.bottom()),
                                            ),
                                            rgb(this.theme.muted),
                                        ));
                                    }
                                }
                                let _ = line.paint(
                                    origin,
                                    bounds.size.height,
                                    TextAlign::Left,
                                    None,
                                    window,
                                    cx,
                                );
                                if focused {
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(origin.x + caret, bounds.top()),
                                            size(px(1.), bounds.size.height),
                                        ),
                                        rgb(this.theme.foreground),
                                    ));
                                }
                                for span in input
                                    .marked
                                    .clone()
                                    .map(|marked| carets.spans(marked))
                                    .unwrap_or_default()
                                {
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(origin.x + span.start, bounds.bottom() - px(1.)),
                                            size(span.end - span.start, px(1.)),
                                        ),
                                        rgb(this.theme.foreground),
                                    ));
                                }
                            });
                            input.bounds = bounds;
                            input.carets = Some(carets);
                        });
                    },
                )
                .size_full(),
            )
    }
}

#[cfg(test)]
mod tests;

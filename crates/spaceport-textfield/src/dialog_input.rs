//! The single-line draft a menu dialog is typed into: one bounded line with
//! its selection, its composition, the keys a draft takes, and where the last
//! paint put each caret, which is where a click and the platform input method
//! read their index from. The frame, the caption a screen reader reads and
//! what Enter means to the dialog stay with the window that shows it.

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Keystroke, Pixels, Point, SharedString, TextAlign,
    TextRun, Window, fill, point, px, rgb, size,
};
use spaceport_settings::{Config, FontConfig, Theme};
use spaceport_widgets::fonts::font_for;

use crate::caret::Carets;
use crate::text_editing::{
    EditCommand, Editing, KeyTable, Keys, Newlines, Platform, edit_command, from_utf16, to_utf16,
};

// Byte offsets internally; only the platform input boundary uses UTF-16.
pub struct DialogInput {
    pub bounds: Bounds<Pixels>,
    /// Where the last drawn text puts each caret.
    carets: Option<Carets>,
    /// The draft's text, selection, and composition, read through `Deref`.
    edit: Editing,
    font: FontConfig,
    /// What a screen reader calls the field, when a caption beside it
    /// names it rather than the field itself.
    pub label: Option<SharedString>,
    scroll: Pixels,
    theme: Theme,
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
            bounds: Bounds::default(),
            carets: None,
            edit: Editing {
                cursor: text.len(),
                text,
                ..Editing::default()
            },
            font: Config::default().ui,
            label: None,
            scroll: px(0.),
            theme: Theme::default(),
        }
    }

    pub fn range_from_utf16(&self, range: Range<usize>) -> Range<usize> {
        let start = from_utf16(&self.edit.text, range.start);
        start..from_utf16(&self.edit.text, range.end).max(start)
    }

    pub fn to_utf16(&self, range: Range<usize>) -> Range<usize> {
        to_utf16(&self.edit.text, range.start)..to_utf16(&self.edit.text, range.end)
    }

    /// The face and colours the field draws in. Unchanged is a no-op, so the
    /// owner may set them from its own render.
    pub fn set_appearance(&mut self, font: FontConfig, theme: Theme) {
        self.font = font;
        self.theme = theme;
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

    /// Paints the draft into `bounds`: its text, its selection, its caret and
    /// the range an input method marked, scrolling so the caret stays in view.
    /// What it drew is kept, for [`Self::index_at`] and [`Self::range_bounds`].
    /// Unfocused, the draft reads as plain text: no caret or selection that
    /// would look like it is about to be replaced.
    pub fn paint(
        &mut self,
        bounds: Bounds<Pixels>,
        focused: bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        let run = TextRun {
            len: self.edit.text.len(),
            font: font_for(&self.font),
            color: rgb(self.theme.foreground).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window.text_system().shape_line(
            self.edit.text.clone().into(),
            px(self.font.size),
            &[run],
            None,
        );
        let carets = Carets::new(&self.edit.text, &line);
        let caret = carets.x(self.edit.cursor);
        self.scroll = self
            .scroll
            .min(caret)
            .max(caret - bounds.size.width + px(2.))
            .max(px(0.));
        // Right to left, a line that fits sets against the field's right,
        // its start.
        let room = bounds.size.width - px(2.);
        if spaceport_widgets::direction::is_rtl() && line.width < room {
            self.scroll = line.width - room;
        }
        let origin = bounds.origin - point(self.scroll, px(0.));
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            // Across a change of direction, a selection is a stretch on each
            // side of it.
            if focused && !self.edit.selection().is_empty() {
                for span in carets.spans(self.edit.selection()) {
                    window.paint_quad(fill(
                        Bounds::from_corners(
                            point(origin.x + span.start, bounds.top()),
                            point(origin.x + span.end, bounds.bottom()),
                        ),
                        rgb(self.theme.muted),
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
                    rgb(self.theme.foreground),
                ));
            }
            for span in self
                .edit
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
                    rgb(self.theme.foreground),
                ));
            }
        });
        self.bounds = bounds;
        self.carets = Some(carets);
    }
}

#[cfg(test)]
mod tests;

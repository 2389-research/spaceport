//! Key handling for the multi-line prompt field: the arrows move the caret
//! between visual rows, and every other key is read from the table the
//! fields share (`crate::text_editing::edit_command`).

use gpui::{ClipboardItem, Context, EntityInputHandler, KeyDownEvent, Window, px};

use super::{MultilineInput, caret_position, is_rtl, line_range, rows_total};
use crate::text_editing::{EditCommand, KeyTable, Keys, Platform, edit_command};

impl MultilineInput {
    /// Move the caret one visual row up or down at its screen column,
    /// wrapping across logical lines. `goal_x` carries the column across a
    /// run of moves; it is reset by any other motion.
    fn vertical_move(&mut self, up: bool, extend: bool, cx: &mut Context<Self>) -> bool {
        let Some(bounds) = self.bounds else {
            return false;
        };
        let Some(lines) = self.lines.as_ref() else {
            return false;
        };
        if lines.is_empty() {
            return false;
        }
        let line_height = self.font.line_height();
        let Some(spot) = caret_position(lines, px(line_height), self.edit.cursor) else {
            return false;
        };
        let rows_before: usize = lines[..spot.line].iter().map(|line| line.rows.len()).sum();
        let total = rows_total(lines);
        let global = rows_before + spot.row;
        let target = match up {
            true => global.checked_sub(1),
            false => (global + 1 < total).then_some(global + 1),
        };
        let Some(target) = target else {
            return false;
        };
        let mut acc = 0;
        let mut found = None;
        for (line_index, line) in lines.iter().enumerate() {
            let next = acc + line.rows.len();
            if target < next {
                found = Some((line_index, target - acc));
                break;
            }
            acc = next;
        }
        let Some((line_index, row_index)) = found else {
            return false;
        };
        // The goal x in field-relative coordinates, set on the first move of
        // a run and held until any other motion.
        let goal = match self.goal_x {
            Some(goal) => goal,
            None => {
                let row = &lines[spot.line].rows[spot.row];
                let origin = if is_rtl() {
                    f32::from(bounds.size.width) - row.width()
                } else {
                    0.
                };
                let goal = origin + spot.x - row.start_x;
                self.goal_x = Some(goal);
                goal
            }
        };
        let line = &lines[line_index];
        let row = &line.rows[row_index];
        let origin = if is_rtl() {
            f32::from(bounds.size.width) - row.width()
        } else {
            0.
        };
        let local = (goal - origin).clamp(0., row.width());
        let byte = line.carets.index_at(px(local + row.start_x));
        let offset = line.range.start + byte.clamp(row.start, row.end);
        self.edit.select_to(offset, extend);
        self.reveal_caret = true;
        cx.notify();
        true
    }

    pub(super) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Let the OS consume composition keys, including picker
        // navigation/accept/dismiss.
        if self.is_composing() {
            return;
        }
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        let previous_cursor = self.edit.cursor;
        if key != "up" && key != "down" {
            self.goal_x = None;
        }
        // The arrows move the caret between visual rows when there is one;
        // otherwise they fall through to the window (completion menu,
        // prompt history).
        if matches!(key, "up" | "down") {
            if self.vertical_move(key == "up", modifiers.shift, cx) {
                cx.stop_propagation();
                window.prevent_default();
            }
            return;
        }
        let keys = Keys {
            line: line_range(&self.edit.text, self.edit.cursor),
            words_hidden: false,
            table: KeyTable::Field,
            platform: Platform::CURRENT,
        };
        // Enter, Escape, and Option/dead-key text are not edits: the window
        // and the native input handler take them.
        let Some(command) = edit_command(key, modifiers, &self.edit, &keys) else {
            return;
        };
        self.apply(command, window, cx);
        cx.stop_propagation();
        window.prevent_default();
        self.reveal_caret |= self.edit.cursor != previous_cursor;
        cx.notify();
    }

    fn apply(&mut self, command: EditCommand, window: &mut Window, cx: &mut Context<Self>) {
        match command {
            EditCommand::Move { to, extend } => self.edit.select_to(to, extend),
            EditCommand::SelectAll => self.edit.select(0..self.edit.text.len()),
            EditCommand::Delete(range) => {
                self.edit.select(range);
                self.replace_text_in_range(None, "", window, cx);
            }
            EditCommand::Kill(range) if range.is_empty() => {}
            EditCommand::Kill(range) => {
                self.killed = self.edit.text[range.clone()].to_owned();
                self.edit.select(range);
                self.replace_text_in_range(None, "", window, cx);
            }
            // With nothing cut yet, a selection stays rather than being
            // replaced by nothing.
            EditCommand::Yank if self.killed.is_empty() => {}
            EditCommand::Yank => {
                let killed = self.killed.clone();
                self.replace_text_in_range(None, &killed, window, cx);
            }
            EditCommand::Copy | EditCommand::Cut => {
                let selection = self.edit.selection();
                if !selection.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        self.edit.text[selection].into(),
                    ));
                    if command == EditCommand::Cut {
                        self.replace_text_in_range(None, "", window, cx);
                    }
                }
            }
            EditCommand::Paste => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.replace_text_in_range(None, &text, window, cx);
                }
            }
        }
    }
}

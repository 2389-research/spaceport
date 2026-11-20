// ABOUTME: Dispatches terminal controls without manufacturing history when an empty screen clears.
// ABOUTME: Delegates the pinned VTE handler contract so synchronized commands keep their execution order.
use crate::listener::Listener;
use alacritty_terminal::{
    Term,
    grid::{Dimensions, GridCell},
    term::TermMode,
    vte::ansi,
};

pub(super) struct Handler<'a>(pub(super) &'a mut Term<Listener>);

macro_rules! forward {
    ($($name:ident($($argument:ident: $kind:ty),*);)*) => {
        $(fn $name(&mut self, $($argument: $kind),*) {
            ansi::Handler::$name(self.0, $($argument),*);
        })*
    };
}

impl ansi::Handler for Handler<'_> {
    // Keep every other parsed control on Alacritty's existing implementation.
    forward! {
        // keep-sorted start
        backspace();
        bell();
        carriage_return();
        clear_line(mode: ansi::LineClearMode);
        clear_tabs(mode: ansi::TabulationClearMode);
        clipboard_load(clipboard: u8, terminator: &str);
        clipboard_store(clipboard: u8, data: &[u8]);
        configure_charset(index: ansi::CharsetIndex, charset: ansi::StandardCharset);
        decaln();
        delete_chars(count: usize);
        delete_lines(count: usize);
        device_status(status: usize);
        dynamic_color_sequence(prefix: String, index: usize, terminator: &str);
        erase_chars(count: usize);
        goto(line: i32, column: usize);
        goto_col(column: usize);
        goto_line(line: i32);
        identify_terminal(intermediate: Option<char>);
        input(character: char);
        insert_blank(count: usize);
        insert_blank_lines(count: usize);
        linefeed();
        move_backward(count: usize);
        move_backward_tabs(count: u16);
        move_down(count: usize);
        move_down_and_cr(count: usize);
        move_forward(count: usize);
        move_forward_tabs(count: u16);
        move_up(count: usize);
        move_up_and_cr(count: usize);
        newline();
        pop_keyboard_modes(count: u16);
        pop_title();
        push_keyboard_mode(mode: ansi::KeyboardModes);
        push_title();
        put_tab(count: u16);
        report_keyboard_mode();
        report_mode(mode: ansi::Mode);
        report_modify_other_keys();
        report_private_mode(mode: ansi::PrivateMode);
        reset_color(index: usize);
        reset_state();
        restore_cursor_position();
        reverse_index();
        save_cursor_position();
        scroll_down(count: usize);
        scroll_up(count: usize);
        set_active_charset(index: ansi::CharsetIndex);
        set_color(index: usize, color: ansi::Rgb);
        set_cursor_shape(shape: ansi::CursorShape);
        set_cursor_style(style: Option<ansi::CursorStyle>);
        set_horizontal_tabstop();
        set_hyperlink(link: Option<ansi::Hyperlink>);
        set_keyboard_mode(mode: ansi::KeyboardModes, behavior: ansi::KeyboardModesApplyBehavior);
        set_keypad_application_mode();
        set_mode(mode: ansi::Mode);
        set_modify_other_keys(mode: ansi::ModifyOtherKeys);
        set_mouse_cursor_icon(icon: ansi::cursor_icon::CursorIcon);
        set_private_mode(mode: ansi::PrivateMode);
        set_scp(character_path: ansi::ScpCharPath, update_mode: ansi::ScpUpdateMode);
        set_scrolling_region(top: usize, bottom: Option<usize>);
        set_tabs(interval: u16);
        set_title(title: Option<String>);
        substitute();
        terminal_attribute(attribute: ansi::Attr);
        text_area_size_chars();
        text_area_size_pixels();
        unset_keypad_application_mode();
        unset_mode(mode: ansi::Mode);
        unset_private_mode(mode: ansi::PrivateMode);
        // keep-sorted end
    }

    fn clear_screen(&mut self, mode: ansi::ClearMode) {
        let empty = matches!(mode, ansi::ClearMode::All)
            && !self.0.mode().contains(TermMode::ALT_SCREEN)
            && self.0.grid().history_size() == 0
            && self
                .0
                .grid()
                .display_iter()
                .all(|cell| cell.cell.is_empty());
        ansi::Handler::clear_screen(self.0, mode);
        if empty {
            // Alacritty saves one blank row even when this clear had no content to retain.
            // Keep it from moving the cursor when a later resize pulls history into view.
            self.0.grid_mut().clear_history();
        }
    }
}

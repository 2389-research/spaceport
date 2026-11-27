//! The Add Device dialog's frame: one title, a rail of what can be added, and
//! the chosen source's form beside it. Each source keeps its own form state,
//! so switching back and forth does not lose what was typed.
//!
//! Which sources this platform can add, and the words naming them, are the
//! app's: it hands the rail over ready to paint.

use crate::chrome::Text;
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::{Theme, corners};

/// The width the rail takes when the window is too narrow for its names, so
/// the form beside it keeps its room.
const COMPACT_WIDTH: f32 = 52.;
/// The narrowest the rail grows to for its names.
const NARROWEST: f32 = 172.;
/// The widest it grows, however long a name is.
const WIDEST: f32 = 240.;
/// The room an entry's icon takes.
const ICON_ROOM: f32 = 24.;

/// One entry of the rail, as the app prepared it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    /// The entry's debug selector, which also names its element.
    pub selector: String,
    /// The icon asset drawn before its name.
    pub icon: String,
    /// The word naming it.
    pub label: String,
    /// Whether the dialog shows this source's form.
    pub current: bool,
}

/// The dialog's frame, as the app prepared it: what its header says, and the
/// sources this platform offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// The dialog's own debug selector.
    pub dialog: String,
    /// The Close button's debug selector.
    pub close: String,
    /// What the header says.
    pub title: String,
    /// The Close button's word.
    pub close_label: String,
    /// The rail's entries, in the order it lists them. A lone entry needs no
    /// rail, since nothing is chosen.
    pub sources: Vec<Source>,
    /// Whether the rail shows its icons alone, because the window is narrow.
    pub compact: bool,
}

/// What pressing one of the frame's controls asks the window to do.
pub enum Action {
    /// Close the dialog.
    Close,
    /// Show the form of the entry at this index of [`Frame::sources`].
    Source(usize),
}

/// The dialog as one source's page draws it: the header, the rail of the
/// sources the app prepared, and `pane` — that source's own form and footer.
///
/// `interact` takes each painted control back and wires what pressing it does,
/// which only the window knows.
pub fn frame(
    text: &Text,
    theme: &Theme,
    frame: Frame,
    pane: impl IntoElement,
    mut interact: impl FnMut(Action, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let Frame {
        dialog,
        close,
        title,
        close_label,
        sources,
        compact,
    } = frame;
    let close = interact(
        Action::Close,
        div()
            .id(close.clone())
            .debug_selector(move || close)
            .px_2()
            .py_1()
            .cursor_pointer()
            .rounded(px(corners::CONTROL))
            .hover(|hovered| hovered.bg(rgb(theme.active)))
            .child(close_label),
    );
    let header = div()
        .debug_selector(|| "device-setup-header".into())
        .flex_none()
        .p(px(16.))
        .border_b_1()
        .border_color(rgb(theme.active))
        .flex()
        .items_center()
        .gap(px(12.))
        .child(
            div()
                .flex_1()
                .text_size(px(text.size * 1.35))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(close);
    let mut body = div().flex_1().min_h_0().flex();
    // A lone source needs no rail.
    if sources.len() > 1 {
        body = body.child(rail(theme, &sources, compact, interact));
    }
    div()
        .debug_selector(move || dialog)
        .flex_1()
        .flex()
        .flex_col()
        .min_h_0()
        .child(header)
        .child(
            body.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(pane),
            ),
        )
}

/// The rail of `sources`, each entry an item `interact` takes back wired with
/// what pressing it does.
fn rail(
    theme: &Theme,
    sources: &[Source],
    compact: bool,
    mut interact: impl FnMut(Action, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let rail = div()
        .debug_selector(|| "add-device-rail".into())
        .flex_none()
        // Wide enough for its names in any language, up to a limit.
        .when(compact, |rail| rail.w(px(COMPACT_WIDTH)))
        .when(!compact, |rail| rail.min_w(px(NARROWEST)).max_w(px(WIDEST)))
        .p(px(8.))
        .flex()
        .flex_col()
        .gap(px(2.))
        .border_r_1()
        .border_color(rgb(theme.active));
    sources
        .iter()
        .enumerate()
        .fold(rail, |rail, (index, source)| {
            let entry = entry(theme, source, compact);
            rail.child(interact(Action::Source(index), entry))
        })
}

/// One entry of the rail, painted, before the window wires what pressing it
/// does.
fn entry(theme: &Theme, source: &Source, compact: bool) -> Stateful<Div> {
    let selector = source.selector.clone();
    let icon = div()
        .flex_none()
        .size(px(ICON_ROOM))
        .rounded(px(corners::SMALL))
        .bg(rgb(theme.active))
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path(source.icon.clone())
                .size(px(14.))
                .text_color(rgb(theme.foreground)),
        );
    div()
        .id(selector.clone())
        .debug_selector(move || selector)
        .p(px(6.))
        .flex()
        .items_center()
        .gap(px(8.))
        .rounded(px(corners::CONTROL))
        .cursor_pointer()
        .when(source.current, |row| row.bg(rgb(theme.primary_wash())))
        .when(!source.current, |row| {
            row.hover(|hovered| hovered.bg(rgb(theme.active)))
        })
        .child(icon)
        .when(!compact, |row| {
            // Its own width counts, so the rail widens to fit it.
            row.child(
                div()
                    .flex_grow(1.)
                    .min_w_0()
                    .truncate()
                    .child(source.label.clone()),
            )
        })
}

//! Scrollbars for the two panes a Settings window scrolls: a thumb over the
//! pane's trailing edge, showing where the pane sits and how much of it shows,
//! and dragging it. The size and travel are the tab strip's thumb math, turned
//! on its side.

use gpui::{prelude::*, *};
use spaceport_settings::Theme;
use spaceport_widgets::direction::Logical;
use spaceport_widgets::thumb::Thumb;

/// How wide the thumb draws, and its margin from the pane's edge.
const WIDTH: f32 = 6.;
const INSET: f32 = 3.;

/// How opaque the thumb draws over the muted color, then hovered or held.
const ALPHA: u32 = 0x60;
const HOVER_ALPHA: u32 = 0x90;
const HELD_ALPHA: u32 = 0xc0;

/// The panes that scroll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    Sections,
    Body,
}

impl Pane {
    /// The thumb's element id, which a native test clicks by.
    fn selector(self) -> &'static str {
        match self {
            Self::Sections => "settings-sections-thumb",
            Self::Body => "settings-body-thumb",
        }
    }

    fn area(self) -> &'static str {
        match self {
            Self::Sections => "settings-sections-area",
            Self::Body => "settings-body-area",
        }
    }
}

/// What a thumb drags: the pane it scrolls.
#[derive(Clone, Copy, Debug)]
struct PaneThumb(Pane);

/// The thumb for `handle`'s pane as it last laid out, with the pane's bounds
/// and how far it scrolls; none while everything fits.
fn thumb(handle: &ScrollHandle) -> Option<(Thumb, Bounds<Pixels>, f32)> {
    let bounds = handle.bounds();
    let max = f32::from(handle.max_offset().y);
    let thumb = Thumb::of(
        f32::from(bounds.size.height),
        max,
        -f32::from(handle.offset().y),
    )?;
    Some((thumb, bounds, max))
}

/// Where on `handle`'s thumb the pointer at `y` takes hold of it: the pane
/// keeps this for the drag that follows.
pub fn grab(handle: &ScrollHandle, y: Pixels) -> f32 {
    thumb(handle).map_or(0., |(thumb, bounds, _)| {
        f32::from(y - bounds.top()) - thumb.left
    })
}

/// Moves `handle`'s thumb with the pointer at `y`, the pointer having taken
/// hold `grab` in; whether it scrolled.
pub fn drag(handle: &ScrollHandle, grab: f32, y: Pixels) -> bool {
    let Some((thumb, bounds, max)) = thumb(handle) else {
        return false;
    };
    let top = f32::from(y - bounds.top()) - grab;
    let scrolled = thumb.scrolled_at(f32::from(bounds.size.height), max, top);
    let offset = handle.offset();
    if (f32::from(offset.y) + scrolled).abs() < 0.5 {
        return false;
    }
    handle.set_offset(point(offset.x, px(-scrolled)));
    true
}

/// `content`, which scrolls with `handle`, with the thumb over its trailing
/// edge. The thumb is the content's sibling, so it stays put while the content
/// scrolls beneath it. `on_grab` takes hold of the thumb where the pointer went
/// down and `on_drag` moves it there; both are the window's, which owns the
/// scroll state the pane reads.
pub fn scroll_area(
    pane: Pane,
    theme: &Theme,
    handle: &ScrollHandle,
    content: impl IntoElement,
    on_grab: impl Fn(Pane, Pixels, &mut Window, &mut App) + 'static,
    on_drag: impl Fn(Pane, Pixels, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let muted = theme.muted;
    let color = move |alpha: u32| rgba((muted << 8) | alpha);
    let thumb = thumb(handle).map(|(thumb, _, _)| {
        div()
            .id(pane.selector())
            .debug_selector(move || pane.selector().into())
            .absolute()
            .inset_e(px(INSET))
            .top(px(thumb.left))
            .w(px(WIDTH))
            .h(px(thumb.width))
            .rounded_full()
            .bg(color(ALPHA))
            // The wheel still scrolls the pane over the thumb.
            .block_mouse_except_scroll()
            .hover(|s| s.bg(color(HOVER_ALPHA)))
            .active(|s| s.bg(color(HELD_ALPHA)))
            .on_mouse_down(
                MouseButton::Left,
                move |event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    on_grab(pane, event.position.y, window, cx);
                },
            )
            .on_drag(PaneThumb(pane), |_, _, _, cx| cx.new(|_| EmptyView))
    });
    div()
        .id(pane.area())
        .relative()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .flex()
        .flex_col()
        .on_drag_move(move |event: &DragMoveEvent<PaneThumb>, window, cx| {
            if event.drag(cx).0 == pane {
                on_drag(pane, event.event.position.y, window, cx);
            }
        })
        .child(content)
        .children(thumb)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;

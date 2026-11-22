//! Side panels resize the way the sidebar does: drag the edge that faces the
//! content, double-click it to go back to the default width. A panel never
//! takes more than `MAX_SHARE` of the window, however wide it was made, so a
//! narrow window keeps room for what the panel sits next to.
mod grip;

use gpui::{prelude::*, *};

pub(crate) use grip::{Grip, GripHold, THICKNESS as GRIP_THICKNESS};

/// How wide the grabbable edge is, laid over the panel's border.
const HANDLE: f32 = 6.;
/// The share of the surrounding space a panel may take.
const MAX_SHARE: f32 = 0.6;
/// A stored width beyond this is not a real panel.
const MAX_WIDTH: f32 = 4096.;

/// Which side of its content a panel sits on; its handle is on the edge
/// facing the content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Left,
    Right,
}

impl Side {
    /// Where a panel that leads its content sits: the left, or in a
    /// right-to-left language the right.
    pub(crate) fn start() -> Self {
        match crate::direction::is_rtl() {
            false => Self::Left,
            true => Self::Right,
        }
    }
}

/// The panel a drag resizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelDrag {
    /// The settings window's section list.
    SettingsNavigation,
}

/// The settings window's section list.
pub(crate) const SETTINGS_NAVIGATION: PanelWidth = PanelWidth::new(184., 150.);

/// A panel's width: its default until the user drags it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelWidth {
    chosen: Option<f32>,
    default: f32,
    min: f32,
}

impl PanelWidth {
    pub(crate) const fn new(default: f32, min: f32) -> Self {
        Self {
            chosen: None,
            default,
            min,
        }
    }

    /// The width to draw beside `available` pixels of space: the chosen or
    /// default width, never more than `MAX_SHARE` of the space, and never
    /// less than the minimum while the space allows it. Space not known yet
    /// (`0`) caps nothing.
    pub(crate) fn width(&self, available: f32) -> f32 {
        let width = self.chosen.unwrap_or(self.default);
        if available <= 0. {
            return width;
        }
        width
            .min(available * MAX_SHARE)
            .max(self.min.min(available))
    }

    /// Follows the pointer at `x` while the panel, laid out at `bounds`, is
    /// dragged by its edge. Whether the width changed.
    pub(crate) fn drag(&mut self, side: Side, bounds: Bounds<Pixels>, x: Pixels) -> bool {
        let width = match side {
            Side::Right => bounds.right() - x,
            Side::Left => x - bounds.left(),
        };
        let width = f32::from(width).clamp(self.min, MAX_WIDTH);
        if self.chosen == Some(width) {
            return false;
        }
        self.chosen = Some(width);
        true
    }

    /// Back to the default width.
    pub(crate) fn reset(&mut self) {
        self.chosen = None;
    }
}

/// The edge to drag, laid over the panel's inner edge; put it inside the
/// panel, which must be `relative()` and listen for `on_drag_move`. `reset`
/// runs on a double-click.
pub(crate) fn handle(
    id: &'static str,
    side: Side,
    drag: PanelDrag,
    reset: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Grip {
    let handle = grip(id, drag, reset);
    // Inside the panel's edge: a panel that slides open clips its overflow.
    Grip::new(
        Axis::Horizontal,
        match side {
            Side::Right => handle.left_0(),
            Side::Left => handle.right_0(),
        },
    )
}

/// The grabbable strip both share: the sidebar's cursor, a drag of `drag`,
/// and `reset` on a double-click.
fn grip(
    id: &'static str,
    drag: PanelDrag,
    reset: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.into())
        .absolute()
        .top_0()
        .h_full()
        .w(px(HANDLE))
        .cursor(CursorStyle::ResizeLeftRight)
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            cx.stop_propagation();
            if event.click_count == 2 {
                reset(event, window, cx);
            }
        })
        .on_drag(drag, |_, _, _, cx| cx.new(|_| EmptyView))
}

#[cfg(test)]
mod tests;

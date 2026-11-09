//! The thin progress bar shared by long-running panels.

use gpui::{prelude::*, *};
use std::time::Duration;

/// How far along a bar is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Progress {
    /// No reliable fraction: a segment slides back and forth.
    Busy,
    /// A settled fraction.
    Fraction(f32),
}

/// A bar selectable as `name`, with its fill as `{name}-fill`.
pub(crate) fn bar(name: &'static str, progress: Progress, accent: Hsla, track: Hsla) -> Div {
    let fill = div()
        .debug_selector(move || format!("{name}-fill"))
        .h_full()
        .rounded_full()
        .bg(accent);
    let fill = match progress {
        Progress::Fraction(fraction) => fill.w(relative(fraction.clamp(0., 1.))).into_any_element(),
        Progress::Busy => fill
            .absolute()
            .w(relative(0.3))
            .with_animation(
                ElementId::Name(format!("{name}-busy").into()),
                Animation::new(crate::motion::duration(Duration::from_secs(2))).repeat(),
                |fill, delta| {
                    fill.left(relative(
                        0.35 * (1. - (delta * std::f32::consts::TAU).cos()),
                    ))
                },
            )
            .into_any_element(),
    };
    div()
        .debug_selector(move || name.into())
        .relative()
        .flex_none()
        .w_full()
        .h(px(6.))
        .rounded_full()
        .overflow_hidden()
        .bg(track)
        .child(fill)
}

// ABOUTME: Exposes screen-reader actions and cached OS display preferences.
// ABOUTME: Keeps native accessibility policy separate from element annotations.
//! Small pieces the chrome uses to tell screen readers what GPUI's own
//! `aria_*` setters cannot, through AccessKit.

pub(crate) mod policy;

use gpui::{
    AccessibleAction, App, BoxShadow, Context, InteractiveElement, StatefulInteractiveElement,
    WeakEntity, Window, accesskit::ActionData, point, prelude::*, px, rgb,
};

/// Gives an enabled control the shared inset keyboard focus ring.
pub(crate) fn focusable<E: InteractiveElement>(control: E, color: u32) -> E {
    let ring = rgb(color);
    control.tab_index(0).focus_visible(move |style| {
        style.shadow(vec![BoxShadow {
            color: ring.into(),
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(2.),
            inset: true,
        }])
    })
}

/// Marks an element unavailable, so a screen reader says so and offers none
/// of its actions. GPUI has no setter for it, so the node is changed directly
/// once built.
pub(crate) fn disabled<E: StatefulInteractiveElement>(element: E, disabled: bool) -> E {
    if !disabled {
        return element;
    }
    element.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
}

/// A screen reader's action listener that runs `f` on a view, with the window.
pub(crate) fn update<T: 'static>(
    view: WeakEntity<T>,
    f: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
) -> impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static {
    move |_, window, cx| {
        let _ = view.update(cx, |this, cx| f(this, window, cx));
    }
}

/// Gives an element that folds content the expanded state and the Expand and
/// Collapse actions, which run the same `toggle` as a click on it. AccessKit
/// only asks for the action that changes the state, so both toggle.
pub(crate) fn expandable<T: 'static, E: StatefulInteractiveElement>(
    element: E,
    expanded: bool,
    view: WeakEntity<T>,
    toggle: impl Fn(&mut T, &mut Window, &mut Context<T>) + Clone + 'static,
) -> E {
    element
        .aria_expanded(expanded)
        .on_a11y_action(
            AccessibleAction::Expand,
            update(view.clone(), toggle.clone()),
        )
        .on_a11y_action(AccessibleAction::Collapse, update(view, toggle))
}

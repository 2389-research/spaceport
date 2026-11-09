//! Native chrome and local profile access.
// i18n: migrated
mod decorations;
mod navigation;
mod status;
mod tabs;

pub(crate) use tabs::{Ends, strip_height};

pub(crate) use decorations::controls;
pub(crate) use decorations::frame;
pub(crate) use decorations::show_controls;

use crate::direction::Logical;
use crate::i18n::fl;
use crate::{SheprdWindow, menu::Page};
use gpui::{prelude::*, *};

/// Profile icon. Smaller than the hit target, which stays a comfortable size.
const AVATAR: f32 = 20.;

const SIDEBAR_TOGGLE_SIZE: f32 = 28.;
const SIDEBAR_GLYPH_WIDTH: f32 = 18.;
/// Empty room between the toggle's edge and the glyph drawn inside it.
const SIDEBAR_TOGGLE_INSET: f32 = (SIDEBAR_TOGGLE_SIZE - SIDEBAR_GLYPH_WIDTH) / 2.;

/// Native chrome the window draws above its body; popups must clear it.
pub(super) const HEIGHT: f32 = 34.;

/// The narrowest window whose header keeps Back and Forward. Below it they
/// give way, so the account and window controls stay reachable; the keys and
/// mouse buttons still travel.
const NAVIGATION_MIN_WIDTH: f32 = 320.;

impl SheprdWindow {
    pub(crate) fn open_profile(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let shown = self.menu.page == Some(Page::Profile);
        if !shown && !self.open_menu(window, cx) {
            return;
        }
        self.menu.page = Some(Page::Profile);
        cx.notify();
    }

    pub(super) fn render_titlebar(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // Without the activity bar, the toggle leads the bar so it stays put
        // whether or not the sidebar below it is showing, and can always
        // bring the sidebar back. The bar keeps it in the left column instead,
        // which never leaves the screen. Back and Forward follow, as in
        // Finder and Files.
        let toggle = !self.activity_bar_leads();
        let leading = div()
            .row()
            .flex_none()
            .items_center()
            .h_full()
            .when(toggle, |leading| leading.child(self.sidebar_toggle(cx)))
            // No toggle precedes Back here, so its inset is not there to
            // complete the room before it.
            .when(!toggle, |leading| leading.ps(px(SIDEBAR_TOGGLE_INSET)))
            .when(
                f32::from(window.viewport_size().width) >= NAVIGATION_MIN_WIDTH,
                |leading| leading.child(self.navigation(cx)),
            )
            .into_any_element();
        render(
            self.fill().tint(bar(self.theme.surface)),
            Some(leading),
            window,
        )
        .child(
            div()
                .debug_selector(|| "titlebar-center".into())
                .flex_1()
                .min_w_0()
                .h_full()
                .when(self.config.usage.topbar, |center| {
                    center.child(status::render(&self.live, &self.config.ui, &self.theme))
                }),
        )
        .child(self.titlebar_end(window, cx))
    }

    pub(crate) fn sidebar_toggle(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id("toggle-sidebar")
            .debug_selector(|| "toggle-sidebar".into())
            .role(Role::Button)
            .aria_label(fl!("titlebar-sidebar"))
            .aria_expanded(self.sidebar_visible)
            .map(|control| self.chrome_stop(crate::window::chrome_focus::Region::TitleBar, control))
            .flex_none()
            .self_center()
            .size(px(SIDEBAR_TOGGLE_SIZE))
            .row()
            .items_center()
            .justify_center()
            .rounded(px(crate::config::corners::CONTROL))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(self.theme.active)))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(|this, _, window, cx| {
                cx.stop_propagation();
                this.command(crate::controls::Command::ToggleSidebar, window, cx);
            }))
            .child(sidebar_glyph(self.sidebar_visible, &self.theme))
            .into_any_element()
    }

    /// What ends the bar: the profile and client-drawn window controls.
    fn titlebar_end(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        div()
            .row()
            .flex_none()
            .items_center()
            .child(
                div()
                    .debug_selector(|| "titlebar-account-slot".into())
                    .row()
                    .items_center()
                    .justify_center()
                    .flex_none()
                    .w(px(40.))
                    .h_full()
                    .child(
                        div()
                            .id("titlebar-avatar")
                            .role(Role::Button)
                            .aria_label(fl!("titlebar-profile"))
                            .map(|control| {
                                self.chrome_stop(
                                    crate::window::chrome_focus::Region::TitleBar,
                                    control,
                                )
                            })
                            .group("titlebar-account")
                            .debug_selector(|| "titlebar-avatar".into())
                            .row()
                            .items_center()
                            .justify_center()
                            .relative()
                            .size(px(28.))
                            .rounded_full()
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_click(cx.listener(|this, _, window, cx| {
                                cx.stop_propagation();
                                this.open_profile(window, cx);
                            }))
                            .on_mouse_down(
                                MouseButton::Right,
                                cx.listener(|this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.open_profile(window, cx);
                                }),
                            )
                            .child(
                                div()
                                    .debug_selector(|| "titlebar-avatar-circle".into())
                                    .row()
                                    .items_center()
                                    .justify_center()
                                    .size(px(AVATAR))
                                    .rounded_full()
                                    .group_hover("titlebar-account", |s| {
                                        s.shadow(vec![BoxShadow {
                                            color: rgba((self.theme.foreground << 8) | 0x38).into(),
                                            offset: point(px(0.), px(0.)),
                                            blur_radius: px(5.),
                                            spread_radius: px(1.),
                                            inset: false,
                                        }])
                                    })
                                    .child(
                                        svg()
                                            .path("icons/user.svg")
                                            .size(px(AVATAR))
                                            .text_color(rgb(self.theme.foreground)),
                                    ),
                            ),
                    ),
            )
            // The in-window menu bar carries the window controls instead.
            .when(!crate::menus::IN_WINDOW, |end| {
                end.children(controls(false, window, &self.theme, |window, _| {
                    window.remove_window();
                }))
            })
    }
}

/// The bar's color, lifted a little off the theme's surface.
fn bar(surface: u32) -> Hsla {
    rgb(surface).blend(rgba(0xffffff1a)).into()
}

/// `leading` sits right after the traffic lights, ahead of the draggable center.
pub(super) fn render(
    background: Hsla,
    leading: Option<AnyElement>,
    window: &Window,
) -> Stateful<Div> {
    // AppKit keeps the traffic lights on the left whatever the app's own
    // language, so a mirrored bar clears them on its end instead.
    let rtl = crate::direction::is_rtl();
    let bar = div()
        .id("titlebar")
        .debug_selector(|| "titlebar".into())
        .row()
        .flex_none()
        .w_full()
        .h(px(HEIGHT))
        .bg(background)
        .when(rtl, |bar| bar.pl(px(LEADING)))
        .child(
            div()
                .flex_none()
                .w(px(if rtl { MARGIN } else { LEADING }))
                .h_full(),
        )
        .children(leading);
    movable(bar, window)
}

/// Makes `area` do a title bar's job: pressing it moves the window, and a
/// double-click acts as the platform's does. Its controls stop their own
/// presses so they never start a move.
///
/// macOS windows own titlebar dragging (`app_owns_titlebar_drag`), so AppKit
/// never moves the window from under a tab, and a client-decorated window has
/// nothing else to move it. A server-drawn frame moves the window from its own
/// title bar, so there `area` is left alone—unless the user has enabled window
/// controls, which implies they may hide the WM's title bar.
pub(crate) fn movable<E: InteractiveElement>(area: E, window: &Window) -> E {
    movable_with_controls(area, window, false)
}

/// Like `movable`, but `controls_enabled` forces dragging even with SSD,
/// for users who hide the WM's title bar and need the menu bar to move.
pub(crate) fn movable_with_controls<E: InteractiveElement>(
    area: E,
    window: &Window,
    controls_enabled: bool,
) -> E {
    let client = decorations::client(window);
    if !cfg!(target_os = "macos") && !client && !controls_enabled {
        return area;
    }
    // The move starts on the press: pointer motion during a press is claimed
    // by window-level drag handlers (selection, splits) before it bubbles up.
    let supported = window.window_controls();
    let area = area.on_mouse_down(MouseButton::Left, move |event, window, _| {
        match event.click_count {
            2 if !client => window.titlebar_double_click(),
            2 if supported.maximize => window.zoom_window(),
            _ => window.start_window_move(),
        }
    });
    if client && supported.window_menu {
        area.on_mouse_down(MouseButton::Right, |event, window, _| {
            window.show_window_menu(event.position);
        })
    } else {
        area
    }
}

/// The bar a secondary window shows: macOS always draws one under its
/// transparent titlebar, and a client-decorated window needs one to be moved
/// and closed at all. Elsewhere the platform frame already provides both.
pub(crate) fn header(
    theme: &crate::config::Theme,
    window: &Window,
    close: impl Fn(&mut Window, &mut App) + 'static,
) -> Option<Stateful<Div>> {
    (cfg!(target_os = "macos") || decorations::client(window)).then(|| {
        let buttons = controls(false, window, theme, close);
        render(bar(theme.surface), None, window).children(buttons)
    })
}

/// Tall enough to clear the traffic lights at `options`' position.
const ZEN_INSET: f32 = 28.;

/// What zen mode keeps of the header: a blank band in the terminal's color
/// that still moves the window, where macOS draws the traffic lights over it
/// and a client-decorated window has nothing else to move it by. Full
/// screen, and a server-drawn frame, need none.
pub(crate) fn zen_inset(ground: Hsla, window: &Window) -> Option<Stateful<Div>> {
    let needed = cfg!(target_os = "macos") || decorations::client(window);
    (needed && !window.is_fullscreen()).then(|| {
        movable(
            div()
                .id("zen-inset")
                .debug_selector(|| "zen-inset".into())
                .flex_none()
                .w_full()
                .h(px(ZEN_INSET))
                .bg(ground),
            window,
        )
    })
}

/// Room before the first control: macOS keeps it clear for the traffic
/// lights, which every other platform draws elsewhere or not at all.
const LEADING: f32 = if cfg!(target_os = "macos") {
    80.
} else {
    MARGIN
};
/// Room before the first control where nothing else needs clearing.
const MARGIN: f32 = 8.;

pub(super) fn options(title: &str) -> TitlebarOptions {
    TitlebarOptions {
        title: Some(title.to_owned().into()),
        appears_transparent: cfg!(target_os = "macos"),
        traffic_light_position: cfg!(target_os = "macos").then(|| point(px(9.), px(9.))),
    }
}

/// A window with its sidebar panel, filled while the sidebar is expanded.
pub(crate) fn sidebar_glyph(expanded: bool, theme: &crate::config::Theme) -> Div {
    div()
        .w(px(SIDEBAR_GLYPH_WIDTH))
        .h(px(14.))
        .flex_none()
        .border_1()
        .border_color(rgb(theme.foreground))
        .rounded(px(2.))
        // The panel sits on the side the sidebar does.
        .row()
        .child(
            div()
                .w(px(5.))
                .h_full()
                .border_e(px(1.))
                .border_color(rgb(theme.foreground))
                .when(expanded, |bar| bar.bg(rgb(theme.foreground))),
        )
}

#[cfg(test)]
mod tests;

#[cfg(all(test, target_os = "macos"))]
mod native_chrome_tests;
#[cfg(test)]
mod removed_git_tests;

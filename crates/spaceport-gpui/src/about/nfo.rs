//! The About box's credits, worded as a demoscene NFO file: the block-letter
//! mark, then one aligned row a field — the group, the version, the code
//! address, the greeting — in the terminal's own mono face.
//!
//! Every row is one message, so a translator words the whole line and the
//! stops that pad a field name stay put. Addresses are placeables, which
//! Fluent isolates, so a row reads right in either direction.
//!
//! The rows keep the debug selectors they have always had: they are what the
//! layout tests, QA and screen readers key on.

use super::demos::{Demo, Ink};
use super::effects::ramp::ramp;
use super::scene;
use super::{
    AUTHOR, COPYRIGHT_YEAR, LICENSE, LICENSE_URL, NOTICE, PUBLISHER, REPOSITORY, SHEPRD_NAME,
    UPSTREAM_REPOSITORY, WEBSITE,
};
use crate::SpaceportWindow;
use crate::config::Theme;
use crate::constants::HERDR_NAME;
use crate::fonts::StyledFont;
use crate::i18n::fl;
use crate::sidebar::label_text;
use gpui::{prelude::*, *};

/// The wordmark, a row at a time: SPACE over PORT, in block letters a mono
/// cell wide.
pub(super) const MARK: [&str; 11] = [
    "█████ ████   ███   ████ █████",
    "█     █   █ █   █ █     █    ",
    "█████ ████  █████ █     ████ ",
    "    █ █     █   █ █     █    ",
    "█████ █     █   █  ████ █████",
    "",
    "████   ███  ████  █████",
    "█   █ █   █ █   █   █  ",
    "████  █   █ ████    █  ",
    "█     █   █ █  ██   █  ",
    "█      ███  █   █   █  ",
];

/// The share of the window's text size the wordmark's cells are drawn at.
const MARK_SHARE: f32 = 0.72;
/// The share of the window's text size the rows are set at: small enough
/// that an address fits the panel, large enough to read.
const ROW_SHARE: f32 = 0.85;
/// The row size's own bounds, so a huge or tiny window text size still sets
/// the document as a document.
const ROW_SIZE: std::ops::RangeInclusive<f32> = 9.0..=13.0;

/// The size one cell of the wordmark is drawn at, at a window text size of
/// `text`: whole pixels, so the eleven rows it stacks to land on the pixel
/// grid and a test can name the art's own height exactly.
pub(super) fn mark_size(text: f32) -> f32 {
    (text * MARK_SHARE).round().max(7.)
}

/// The size the document's rows are set at, at a window text size of `text`.
pub(super) fn row_size(text: f32) -> f32 {
    (text * ROW_SHARE).clamp(*ROW_SIZE.start(), *ROW_SIZE.end())
}

/// The colours the wordmark's rows are ramped through: the stage's own field,
/// inked so the letters read over whatever the demo paints behind them.
fn mark_ink(theme: &Theme) -> [u32; 3] {
    Ink::of(theme).field.map(|color| theme.ink(color))
}

impl SpaceportWindow {
    /// The document the box says, over the stage `demo` paints.
    pub(super) fn render_about_nfo(&self, demo: Demo, cx: &mut Context<Self>) -> Div {
        let text = row_size(self.config.ui.size);
        div()
            .debug_selector(|| "about".into())
            .flex()
            .flex_col()
            .items_center()
            .gap(px(text * 0.3))
            .px(px(10.))
            .pt(px(scene::margin(demo, self.config.ui.size)))
            .pb(px(scene::ROOM_BOTTOM))
            .text_font(&self.config.terminal)
            .text_size(px(text))
            .line_height(px(text * 1.2))
            .child(self.render_about_mark())
            .child(
                div()
                    .debug_selector(|| "about-name".into())
                    .child(crate::constants::WINDOW_TITLE), // i18n: data
            )
            .child(self.render_about_fields(text))
            .children(self.render_about_build())
            .child(self.render_about_sites(text))
            .child(self.render_about_fine_print(text))
            .child(self.render_about_close(text, cx))
    }

    /// The mark: the wordmark in block letters, its rows ramped across the
    /// stage's palette, so the box opens on real ANSI art rather than a
    /// picture.
    fn render_about_mark(&self) -> Div {
        let ink = mark_ink(&self.theme);
        let size = mark_size(self.config.ui.size);
        div()
            .debug_selector(|| "about-icon".into())
            .flex()
            .flex_col()
            .items_start()
            .text_size(px(size))
            .line_height(px(size))
            .children(MARK.iter().enumerate().map(|(row, line)| {
                let lit = row as f32 / (MARK.len() - 1) as f32;
                div()
                    .text_color(rgb(ramp(&ink, lit)))
                    .child(line.to_string())
            }))
    }

    /// The fields: which version this is, who made it, where the code lives,
    /// and what it is.
    fn render_about_fields(&self, text: f32) -> Div {
        let theme = &self.theme;
        let muted = rgb(theme.muted);
        div()
            .flex()
            .flex_col()
            .items_start()
            .w_full()
            .child(row("about-version", muted).child(label_text(&fl!(
                "about-version",
                version = crate::APP_VERSION
            ))))
            .child(
                row("about-publisher", rgb(theme.foreground)).child(label_text(&fl!(
                    "about-publisher",
                    name = crate::constants::WINDOW_TITLE,
                    publisher = PUBLISHER
                ))),
            )
            .child(self.about_link(
                "about-repository",
                label_text(&fl!("about-repository", url = REPOSITORY)),
                REPOSITORY,
            ))
            .child(
                row("about-summary", muted)
                    .pt(px(text * 0.4))
                    .child(label_text(&fl!("about-summary"))),
            )
            .child(row("about-upstream-credits", muted).child(label_text(&fl!(
                "about-upstream-credits",
                herdr = HERDR_NAME,
                sheprd = SHEPRD_NAME
            ))))
            .child(row("about-unaffiliated", muted).child(label_text(&fl!(
                "about-unaffiliated",
                herdr = HERDR_NAME,
                sheprd = SHEPRD_NAME
            ))))
    }

    /// Worktree builds are throwaway; name the branch that produced this one.
    fn render_about_build(&self) -> Option<Div> {
        (env!("HERDR_BUILD_WORKTREE") == "1").then(|| {
            let pr = env!("HERDR_BUILD_PR");
            div()
                .debug_selector(|| "about-build".into())
                .flex()
                .flex_col()
                .items_start()
                .w_full()
                .child(
                    row("about-branch", rgb(self.theme.muted)).child(label_text(&fl!(
                        "about-branch",
                        branch = env!("HERDR_BUILD_BRANCH")
                    ))),
                )
                .when(!pr.is_empty(), |build| {
                    build.child(self.about_link(
                        "about-pr",
                        label_text(&fl!("about-pull-request", number = pr)),
                        crate::worktree_banner::pull_request_url(pr),
                    ))
                })
        })
    }

    /// The two upstream teams' own addresses.
    fn render_about_sites(&self, text: f32) -> Div {
        div()
            .flex()
            .flex_col()
            .items_start()
            .w_full()
            .pt(px(text * 0.4))
            .child(self.about_link(
                "about-website",
                label_text(&fl!(
                    "about-upstream-team",
                    name = HERDR_NAME,
                    url = WEBSITE
                )),
                WEBSITE,
            ))
            .child(self.about_link(
                "about-upstream-repository",
                label_text(&fl!(
                    "about-upstream-team",
                    name = SHEPRD_NAME,
                    url = UPSTREAM_REPOSITORY
                )),
                UPSTREAM_REPOSITORY,
            ))
    }

    /// The fine print: this fork's copyright, and the work it stands on.
    fn render_about_fine_print(&self, text: f32) -> Div {
        let ink = rgb(self.theme.muted);
        div()
            .debug_selector(|| "about-copyright".into())
            .flex()
            .flex_col()
            .items_start()
            .w_full()
            .pt(px(text * 0.4))
            .text_color(ink)
            .child(row("about-modifications", ink).child(label_text(&fl!(
                "about-modifications",
                name = crate::constants::WINDOW_TITLE,
                copyright = COPYRIGHT_YEAR,
                publisher = PUBLISHER
            ))))
            .child(row("about-original-copyright", ink).child(label_text(&fl!(
                "about-original-copyright",
                name = SHEPRD_NAME,
                copyright = COPYRIGHT_YEAR,
                author = AUTHOR
            ))))
            .child(self.about_link(
                "about-license",
                label_text(&fl!("about-license", license = LICENSE)),
                LICENSE_URL,
            ))
            .child(self.about_link(
                "about-notices",
                label_text(&fl!("about-notices", url = NOTICE)),
                NOTICE,
            ))
    }

    /// OK, framed as a release file's own sign-off.
    fn render_about_close(&self, text: f32, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = &self.theme;
        div()
            .id("about-close")
            .debug_selector(|| "about-close".into())
            .mt(px(text * 0.6))
            .px(px(text))
            .cursor_pointer()
            .border_1()
            .border_color(rgb(theme.active))
            .hover(|style| style.border_color(crate::menu::accent(theme)))
            .child(fl!("about-ok"))
            .on_click(cx.listener(|this, _, window, cx| {
                cx.stop_propagation();
                this.dismiss_menu(window, cx);
            }))
    }

    /// One row that opens a place: the whole line is the press, and the
    /// underline that appears under it says so.
    fn about_link(
        &self,
        id: &'static str,
        label: impl IntoElement,
        url: impl Into<String>,
    ) -> Stateful<Div> {
        let url = url.into();
        let accent = crate::menu::accent(&self.theme);
        div()
            .id(id)
            .debug_selector(move || id.into())
            .flex_none()
            .w_full()
            .cursor_pointer()
            .text_color(accent)
            .border_b_1()
            .border_color(transparent_black())
            .hover(|style| style.border_color(accent))
            .child(label)
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                cx.open_url(&url);
            })
    }
}

/// A row of the document that opens nothing.
fn row(id: &'static str, ink: Rgba) -> Div {
    div()
        .debug_selector(move || id.into())
        .w_full()
        .text_color(ink)
}

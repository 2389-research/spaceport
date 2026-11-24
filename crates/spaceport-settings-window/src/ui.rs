//! Shared row and control builders for the Settings pages. Pure rendering:
//! they take the theme and the app's [`Labels`] and return elements; state and
//! saves stay on the view.
//!
//! No word is spelled here: what the kit shows comes from [`Labels`], and the
//! ids it names itself are [`MESSAGE_IDS`].
use gpui::{prelude::*, *};
use spaceport_settings::catalog::Entry;
use spaceport_settings::{Theme, corners};
use spaceport_widgets::direction::Logical;
use spaceport_widgets::{accessibility, toggles};
use std::rc::Rc;

pub mod number;
pub mod popover;
pub mod swatches;

pub use number::{Editing, Look, Number, NumberChange, Unit, number, parse_number};
pub use popover::{Dropdown, VALUE_MARK, dropdown};
pub use swatches::{HostColorChange, SWATCH_NONE, parse_hex_input, swatch_row};

pub const ROW_HEIGHT: f32 = 38.;

/// Where a setting is stored: Herdr's own config, which its terminal client
/// reads too, or Spaceport's file, which only this app reads. The tags name
/// the apps rather than the files, since that is what decides who sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Herdr,
    Spaceport,
}

/// The messages this crate names, so the app can word them and check that each
/// is a message in its English files. Nothing here holds a word: the app passes
/// its own through [`Labels`]. Each id is declared once, where it is used.
pub const MESSAGE_IDS: &[&str] = &[
    SWATCH_NONE,
    Source::Herdr.label_id(),
    Source::Spaceport.label_id(),
    Unit::Percent.label_id(),
    Unit::Pixels.label_id(),
    Unit::Seconds.label_id(),
];

/// The app's words, worded in the language the window shows. The kit draws
/// them; it names none itself.
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Message `id` as the window shows it, which the ids this crate names and
    /// a catalogued row's own are read through.
    pub message: fn(&'static str) -> String,
    /// The band a number field accepts, from its low end to its high.
    pub number_range: fn(f32, f32) -> String,
    /// How a label that follows Herdr's value is worded, with [`VALUE_MARK`]
    /// where the value goes.
    pub follow_herdr: fn() -> String,
}

impl Source {
    /// The message naming the app that holds this setting.
    pub const fn label_id(self) -> &'static str {
        match self {
            Self::Herdr => "settings-source-herdr",
            Self::Spaceport => "settings-source-herdr-ui",
        }
    }
}

fn source_tag(theme: &Theme, source: Source, label: impl Into<SharedString>) -> Div {
    let color = match source {
        Source::Herdr => theme.ink(0x8fb8d9),
        Source::Spaceport => theme.ink(0xc9b26f),
    };
    div()
        .text_size(px(9.))
        .line_height(px(13.))
        .px(px(5.))
        .rounded(px(corners::SMALL))
        .border_1()
        .border_color(rgb(theme.active))
        .text_color(rgb(color))
        .font_weight(FontWeight::MEDIUM)
        .child(label.into())
}

/// A section heading, with the tag of the app that holds its settings where
/// `tag` names one: its source and the app's word for it.
pub fn section(theme: &Theme, title: &str, tag: Option<(Source, SharedString)>) -> Div {
    div()
        .row()
        .items_center()
        .gap(px(6.))
        .mt(px(22.))
        .pb(px(6.))
        .border_b_1()
        .border_color(rgb(theme.active))
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(theme.subtext()))
        .child(title.to_uppercase())
        .children(tag.map(|(source, label)| source_tag(theme, source, label)))
}

pub struct Row {
    theme: Theme,
    labels: Labels,
    id: SharedString,
    title: SharedString,
    description: Option<SharedString>,
    tag: Option<Source>,
    disabled: Option<SharedString>,
    controls: Vec<AnyElement>,
}

/// A catalogued row: its id, title and description are its entry's, the
/// ones search matches, so a page states none of them itself.
pub fn row(theme: &Theme, entry: &Entry, labels: &Labels) -> Row {
    let row = data_row(theme, entry.id, (labels.message)(entry.title), labels);
    if entry.description.is_empty() {
        row
    } else {
        row.description((labels.message)(entry.description))
    }
}

/// A row the catalog does not list because it is built from data: a search
/// result, or one row per SSH host.
pub fn data_row(
    theme: &Theme,
    id: impl Into<SharedString>,
    title: impl Into<SharedString>,
    labels: &Labels,
) -> Row {
    Row {
        theme: theme.clone(),
        labels: *labels,
        id: id.into(),
        title: title.into(),
        description: None,
        tag: None,
        disabled: None,
        controls: Vec::new(),
    }
}

impl Row {
    /// Sets the text under the title. On a catalogued row this replaces the
    /// entry's description, for a page that reports state or data there.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn tag(mut self, source: Source) -> Self {
        self.tag = Some(source);
        self
    }
    /// Dims the row's text and shows `reason` in place of the description.
    /// Its controls dim themselves when the caller builds them disabled.
    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled = Some(reason.into());
        self
    }
    pub fn control(mut self, control: impl IntoElement) -> Self {
        self.controls.push(control.into_any_element());
        self
    }
    /// A switch, named for a screen reader by the row's title.
    pub fn switch(self, switch: Stateful<Div>) -> Self {
        let title = self.title.clone();
        self.control(switch.aria_label(title))
    }
}

/// A row is a group named by its title, with its description as the help a
/// screen reader reads after it, so its unnamed controls are heard in context.
impl IntoElement for Row {
    type Element = Stateful<Div>;
    fn into_element(self) -> Stateful<Div> {
        let id = self.id.clone();
        let title = self.title.clone();
        let help = self.disabled.clone().or_else(|| self.description.clone());
        self.into_div()
            .id(ElementId::Name(format!("{id}-row").into()))
            .role(Role::Group)
            .aria_label(title)
            .when_some(help, |row, help| row.aria_description(help))
    }
}

impl Row {
    /// The row without its group, for a caller that makes the whole row one
    /// control, as search results do.
    pub fn into_div(self) -> Div {
        let theme = &self.theme;
        let id = self.id.clone();
        let title_id = format!("{id}-title");
        let desc_id = format!("{id}-description");
        let tag_id = format!("{id}-tag");
        let dimmed = self.disabled.is_some();
        let tracked = id.clone();
        let reason_id = format!("{id}-reason");
        // A disabled row shows its reason instead of the description.
        let (description, desc_id) = match self.disabled {
            Some(reason) => (Some(reason), reason_id),
            None => (self.description, desc_id),
        };
        let labels = self.labels;
        div()
            .debug_selector(move || id.to_string())
            .relative()
            .row()
            .items_center()
            .gap(px(16.))
            .min_h(px(ROW_HEIGHT))
            .py(px(6.))
            .border_b_1()
            .border_color(rgb(theme.active))
            .child(crate::bounds::track_bounds(tracked))
            .child(
                // Only the text dims: a control a disabled row cannot use is
                // built disabled and dims itself, and GPUI multiplies nested
                // opacity, so dimming the whole row would show it at 25 %.
                div()
                    .flex_1()
                    .min_w_0()
                    .when(dimmed, |text| text.opacity(0.5))
                    .child(
                        div()
                            .debug_selector(move || title_id.clone())
                            .row()
                            .items_center()
                            .gap(px(6.))
                            .text_size(px(12.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.title)
                            .children(self.tag.map(|source| {
                                let label = (labels.message)(source.label_id());
                                source_tag(theme, source, label)
                                    .debug_selector(move || tag_id.clone())
                            })),
                    )
                    .children(description.map(|text| {
                        div()
                            .debug_selector(move || desc_id.clone())
                            .text_size(px(11.))
                            .line_height(px(15.))
                            .text_color(rgb(theme.subtext()))
                            .child(text)
                    })),
            )
            .child(
                div()
                    .row()
                    .items_center()
                    .gap(px(8.))
                    .flex_none()
                    .children(self.controls),
            )
    }
}

/// The switch track alone; the row carries the label.
pub fn toggle(
    theme: &Theme,
    id: impl Into<SharedString>,
    checked: bool,
    enabled: bool,
) -> Stateful<Div> {
    let id = id.into();
    let track = toggles::switch(theme, 20., checked)
        .id(ElementId::Name(id.clone()))
        .debug_selector(move || id.to_string())
        .role(Role::Switch)
        .aria_toggled(if checked {
            Toggled::True
        } else {
            Toggled::False
        });
    accessibility::disabled(track, !enabled)
        .when(enabled, |track| track.cursor_pointer())
        .when(!enabled, |track| track.opacity(0.5))
}

/// A bordered text button; the caller adds `.on_click` when it is enabled.
pub fn button(
    theme: &Theme,
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    enabled: bool,
) -> Stateful<Div> {
    let id = id.into();
    let label = label.into();
    let button = div()
        .id(ElementId::Name(id.clone()))
        .debug_selector(move || id.to_string())
        .role(Role::Button)
        .aria_label(label.clone());
    accessibility::disabled(button, !enabled)
        .flex_none()
        .whitespace_nowrap()
        .px(px(10.))
        .py(px(3.))
        .rounded(px(corners::CONTROL))
        .border_1()
        .border_color(rgb(theme.active))
        .bg(rgb(theme.surface))
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgb(theme.active)))
        })
        .when(!enabled, |button| button.opacity(0.5))
        .child(label)
}

pub fn segmented<T: Copy + PartialEq + 'static>(
    theme: &Theme,
    id: &'static str,
    options: &[(T, impl Clone + Into<SharedString>)],
    selected: Option<T>,
    enabled: bool,
    on_pick: impl Fn(T, &mut Window, &mut App) + 'static,
) -> Div {
    let names = options
        .iter()
        .map(|(_, label)| Some(label.clone().into()))
        .collect();
    let options = options
        .iter()
        .map(|(value, label)| (*value, label.clone().into().into_any_element()))
        .collect();
    segments(
        theme,
        id,
        (options, names),
        selected,
        enabled,
        |_| true,
        on_pick,
    )
}

/// Like [`segmented`], with an element per segment (e.g. coloured status marks).
/// `live` says which segments respond while the strip is enabled; the others
/// are dimmed and ignore clicks. Only tests build one now.
#[cfg(test)]
pub(crate) fn segmented_elements<T: Copy + PartialEq + 'static>(
    theme: &Theme,
    id: &'static str,
    options: Vec<(T, AnyElement)>,
    selected: Option<T>,
    enabled: bool,
    live: impl Fn(T) -> bool,
    on_pick: impl Fn(T, &mut Window, &mut App) + 'static,
) -> Div {
    let names = vec![None; options.len()];
    segments(
        theme,
        id,
        (options, names),
        selected,
        enabled,
        live,
        on_pick,
    )
}

/// The strip itself: each segment a radio button, named by `names` where
/// its element is more than plain text.
fn segments<T: Copy + PartialEq + 'static>(
    theme: &Theme,
    id: &'static str,
    (options, names): (Vec<(T, AnyElement)>, Vec<Option<SharedString>>),
    selected: Option<T>,
    enabled: bool,
    live: impl Fn(T) -> bool,
    on_pick: impl Fn(T, &mut Window, &mut App) + 'static,
) -> Div {
    let on_pick = Rc::new(on_pick);
    div()
        .row()
        .flex_none()
        .p(px(2.))
        .rounded(px(corners::CONTROL))
        .border_1()
        .border_color(rgb(theme.active))
        .bg(rgb(theme.surface))
        .when(!enabled, |strip| strip.opacity(0.5))
        .children(options.into_iter().zip(names).enumerate().map(
            |(index, ((value, label), name))| {
                let on = selected == Some(value);
                let segment_live = enabled && live(value);
                let on_pick = on_pick.clone();
                let segment = div()
                    .id((id, index))
                    .role(Role::RadioButton)
                    .when_some(name, |segment, name| segment.aria_label(name))
                    .aria_toggled(if on { Toggled::True } else { Toggled::False });
                accessibility::disabled(segment, !segment_live)
                    .debug_selector(move || format!("{id}-{index}"))
                    .px(px(10.))
                    .py(px(3.))
                    .rounded(px(corners::SMALL))
                    .whitespace_nowrap()
                    .text_color(rgb(if on {
                        theme.foreground
                    } else {
                        theme.subtext()
                    }))
                    .when(on, |seg| seg.bg(rgb(theme.active)))
                    .when(enabled && !segment_live, |seg| seg.opacity(0.5))
                    .when(segment_live, |seg| seg.cursor_pointer())
                    .when(segment_live && !on, move |seg| {
                        seg.on_click(move |_, window, cx| on_pick(value, window, cx))
                    })
                    .child(label)
            },
        ))
}

#[cfg(test)]
mod tests;

// ABOUTME: Applies device grouping while keeping departing choices inert.
// ABOUTME: Captures only visible tag rows and disposable field paint.
//! Grouping a device from its menu. Move to picks the section it lists under;
//! Also tagged adds the tags it carries besides. A new name in the field
//! makes a new group. Both save through the local `device_tags` preference.
use super::*;
use crate::{
    SheprdWindow,
    device_tags::{self, DeviceTags},
    i18n::fl,
    menu::Page,
    search_input::SearchInput,
};
use device_tags::normalized;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    MoveTo,
    AlsoTagged,
}

#[cfg(test)]
mod tests;

pub(crate) struct TagPicker {
    mode: Mode,
    /// The tags to offer; Move to adds No group after them.
    rows: Vec<String>,
    selected: Option<usize>,
    input: Entity<SearchInput>,
    /// The uniform list's snapped extent; the shell remains the only scroller.
    layout: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
}

impl TagPicker {
    fn count(&self) -> usize {
        self.rows.len() + usize::from(self.mode == Mode::MoveTo)
    }
}

pub(super) struct Choice {
    index: usize,
    label: String,
    ticked: bool,
    selected: bool,
}

pub(super) struct Paint {
    title: String,
    rows: Vec<Choice>,
    before: Pixels,
    after: Pixels,
    input: crate::search_input::Paint,
}

impl SheprdWindow {
    pub(super) fn open_device_tags(
        &mut self,
        mode: Mode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = cx.new(SearchInput::new);
        input.update(cx, |input, cx| {
            input.set_placeholder_with(|| fl!("device-tags-new"), cx);
            input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
            window.focus(&input.focus, cx);
        });
        let Some(host) = &mut self.menu.host else {
            return;
        };
        host.tags = Some(TagPicker {
            mode,
            rows: device_tags::offered(&self.device_tags),
            selected: None,
            input,
            layout: Default::default(),
        });
        self.menu.page = Some(Page::DeviceTags);
        cx.notify();
    }

    /// Applies row `index`: Move to closes, as the device has moved; Also
    /// tagged stays open for the next tick.
    fn choose_device_tag(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(host) = &self.menu.host else {
            return;
        };
        let Some(picker) = &host.tags else {
            return;
        };
        let row = picker.rows.get(index).cloned();
        self.apply_device_tag(row, window, cx);
    }

    /// `tag` applied as the page's mode says. `None` is No group.
    fn apply_device_tag(
        &mut self,
        tag: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(host) = &self.menu.host else {
            return;
        };
        let Some(picker) = &host.tags else {
            return;
        };
        let (id, mode) = (host.id.clone(), picker.mode);
        let tags = match (mode, &tag) {
            (Mode::MoveTo, tag) => self.device_tags.moved_to(&id, tag.as_deref()),
            (Mode::AlsoTagged, Some(tag)) => self.device_tags.toggled(&id, tag),
            (Mode::AlsoTagged, None) => return,
        };
        DeviceTags::set(cx, &id, tags);
        self.device_tags = DeviceTags::current(cx);
        if mode == Mode::MoveTo {
            self.dismiss_menu(window, cx);
            return;
        }
        let rows = device_tags::offered(&self.device_tags);
        if let Some(picker) = self.menu.host.as_mut().and_then(|host| host.tags.as_mut()) {
            picker.rows = rows;
            picker
                .input
                .update(cx, |input, cx| input.set_text_at("", 0, cx));
        }
        cx.notify();
    }

    pub(in crate::menu) fn device_tags_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(picker) = self.menu.host.as_mut().and_then(|host| host.tags.as_mut()) else {
            return;
        };
        // Unhandled text must reach the native input system, including IME.
        if picker.input.read(cx).is_composing() {
            return;
        }
        match event.keystroke.key.as_str() {
            "escape" => {
                cx.stop_propagation();
                window.prevent_default();
                self.dismiss_menu(window, cx);
            }
            key @ ("up" | "down") => {
                cx.stop_propagation();
                window.prevent_default();
                let count = picker.count();
                picker.selected = (count > 0).then(|| match (picker.selected, key) {
                    (None, "up") => count - 1,
                    (None, _) => 0,
                    (Some(index), "up") => (index + count - 1) % count,
                    (Some(index), _) => (index + 1) % count,
                });
                cx.notify();
            }
            "enter" => {
                cx.stop_propagation();
                window.prevent_default();
                let typed = normalized(picker.input.read(cx).text());
                match (typed, picker.selected) {
                    (Some(tag), _) => self.apply_device_tag(Some(tag), window, cx),
                    (None, Some(index)) => self.choose_device_tag(index, window, cx),
                    (None, None) => {}
                }
            }
            _ => {}
        }
    }

    pub(super) fn device_tags_paint(
        &self,
        host: &HostMenu,
        frozen: bool,
        cx: &App,
    ) -> Option<Paint> {
        let picker = host.tags.as_ref()?;
        let title = match picker.mode {
            Mode::MoveTo => fl!("device-tags-move-title", device = host.label.as_str()),
            Mode::AlsoTagged => fl!("device-tags-also-title", device = host.label.as_str()),
        };
        let section = self.device_tags.section(&host.id, None);
        let count = picker.count();
        let bounds = picker.layout.get();
        let height = if count > 0 && bounds.size.height > px(0.) {
            bounds.size.height / count as f32
        } else {
            px(self.config.ui.line_height() + 12.)
        };
        let (start, end) = if frozen && bounds.size.height > px(0.) {
            let viewport = host.scroll.bounds();
            let start = ((viewport.top() - bounds.top()) / height).floor().max(0.) as usize;
            let end = ((viewport.bottom() - bounds.top()) / height).ceil().max(0.) as usize;
            (start.min(count), end.min(count))
        } else {
            (0, count)
        };
        // Spacers use the actual snapped extents, not a fractional font-height product.
        let before = height * start as f32;
        let after = height * (count - end) as f32;
        Some(Paint {
            title,
            rows: (start..end)
                .map(|index| {
                    let (label, ticked) = match picker.rows.get(index) {
                        Some(tag) => (
                            tag.clone(),
                            match picker.mode {
                                Mode::MoveTo => {
                                    section.is_some_and(|own| device_tags::same(own, tag))
                                }
                                Mode::AlsoTagged => self.device_tags.has(&host.id, tag),
                            },
                        ),
                        None => (fl!("device-tags-remove-section"), section.is_none()),
                    };
                    Choice {
                        index,
                        label,
                        ticked,
                        selected: picker.selected == Some(index),
                    }
                })
                .collect(),
            before,
            after,
            input: picker.input.read(cx).paint(),
        })
    }

    pub(super) fn render_device_tags(
        &self,
        paint: &Paint,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let row = |choice: &Choice, cx: &mut Context<Self>| {
            let index = choice.index;
            div()
                .id(("device-tags-row", index))
                .debug_selector(move || format!("device-tags-row-{index}"))
                .min_h(px(font.line_height() + 12.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(8.))
                .when(active, |row| row.cursor_pointer())
                .when(choice.selected, |row| row.bg(rgb(theme.active)))
                .when(active, |row| row.hover(|row| row.bg(rgb(theme.active))))
                .child(crate::toggles::checkbox(
                    theme,
                    font.size + 2.,
                    choice.ticked,
                ))
                .child(div().min_w_0().truncate().child(choice.label.clone()))
                .when(active, |row| {
                    row.on_click(cx.listener(move |this, _, window, cx| {
                        this.choose_device_tag(index, window, cx);
                    }))
                })
        };
        let list = div()
            .debug_selector(|| "device-tags-list".into())
            .flex()
            .flex_col()
            .when(paint.before > px(0.), |list| {
                list.child(div().flex_none().h(paint.before))
            })
            .children(paint.rows.iter().map(|choice| row(choice, cx)))
            .when(paint.after > px(0.), |list| {
                list.child(div().flex_none().h(paint.after))
            })
            .when(active, |list| {
                list.when_some(
                    self.menu.host.as_ref().and_then(|host| host.tags.as_ref()),
                    |list, picker| {
                        let layout = picker.layout.clone();
                        list.child(
                            canvas(move |bounds, _, _| layout.set(bounds), |_, _, _, _| {})
                                .absolute()
                                .inset_0(),
                        )
                    },
                )
            });
        div()
            .debug_selector(|| "device-tags".into())
            .flex()
            .flex_col()
            .gap(px(6.))
            .p(px(8.))
            .child(
                div()
                    .debug_selector(|| "device-tags-title".into())
                    .font_weight(FontWeight::SEMIBOLD)
                    .truncate()
                    .child(paint.title.clone()),
            )
            .child(list)
            .when(active, |body| {
                body.when_some(
                    self.menu
                        .host
                        .as_ref()
                        .and_then(|host| host.tags.as_ref())
                        .map(|picker| picker.input.clone()),
                    |body, input| body.child(input),
                )
            })
            .when(!active, |body| {
                body.child(
                    paint
                        .input
                        .render()
                        .debug_selector(|| "device-tags-field-paint".into()),
                )
            })
    }
}

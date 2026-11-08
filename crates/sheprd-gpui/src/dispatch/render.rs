//! Painting host pickers: the best hosts as tiles, every other host in a list
//! behind an "Other" field, drawn from the picker's prepared candidates.

use super::{Candidate, Picker, Repository};
use crate::{SheprdWindow, config::corners, i18n::fl, toggles::radio};
use gpui::{prelude::*, *};

/// Share of CPU at which a meter turns to a warning, and then to an alarm,
/// as the system-load gauges do.
const CPU_WARN: f32 = 70.;
const CPU_CRITICAL: f32 = 90.;
const METER_WIDTH: f32 = 36.;

fn cpu(candidate: &Candidate) -> Option<f32> {
    candidate.load.and_then(|load| load.cpu)
}

fn free_cores(candidate: &Candidate) -> Option<String> {
    let spare = candidate.load?.spare_cores()?.round() as u32;
    Some(fl!("dispatch-cores-free", count = spare))
}

fn agents(candidate: &Candidate) -> String {
    fl!("dispatch-agents-working", count = candidate.working)
}

/// The quiet word a host carries: why it ranks where it does, or what
/// choosing it costs.
fn tag(candidate: &Candidate, first: bool) -> Option<String> {
    if !candidate.online {
        Some(fl!("dispatch-tag-offline"))
    } else if first && !candidate.current {
        Some(fl!("dispatch-tag-best"))
    } else if candidate.current {
        Some(fl!("dispatch-tag-current"))
    } else if candidate.repository == Repository::Missing {
        Some(fl!("dispatch-tag-clone"))
    } else if candidate.picks >= 2 {
        Some(fl!("dispatch-tag-usual"))
    } else {
        None
    }
}

impl SheprdWindow {
    fn dispatch_meter(&self, value: Option<f32>, width: f32) -> Div {
        let theme = &self.theme;
        let fill = value.map(|value| {
            let color = if value >= CPU_CRITICAL {
                theme.ink(theme.palette[1])
            } else if value >= CPU_WARN {
                theme.ink(theme.palette[3])
            } else {
                theme.ink(theme.palette[2])
            };
            div()
                .h_full()
                .w(px(width * value.clamp(0., 100.) / 100.))
                .rounded_full()
                .bg(rgb(color))
        });
        div()
            .w(px(width))
            .h(px(4.))
            .flex_none()
            .rounded_full()
            .bg(rgb(theme.active))
            .children(fill)
    }

    pub(crate) fn pick_host(&mut self, endpoint_id: &str, cx: &mut Context<Self>) {
        let changed = self
            .menu
            .dispatch
            .as_mut()
            .is_some_and(|picker| picker.choose(endpoint_id));
        if changed {
            self.menu.error = None;
        }
        cx.notify();
    }

    fn toggle_host_list(&mut self, cx: &mut Context<Self>) {
        if let Some(picker) = &mut self.menu.dispatch {
            picker.toggle();
            cx.notify();
        }
    }

    fn dispatch_tile(
        &self,
        candidate: &Candidate,
        first: bool,
        chosen: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let size = self.config.ui.size;
        let id = candidate.endpoint_id.clone();
        let selector = format!("dispatch-tile-{id}");
        let muted = rgb(theme.subtext());
        div()
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(3.))
            .p(px(8.))
            .rounded(px(corners::CONTROL))
            .border_1()
            .border_color(rgb(if chosen {
                theme.primary()
            } else {
                theme.active
            }))
            .when(chosen, |tile| tile.bg(rgb(theme.primary_wash())))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.pick_host(&id, cx);
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(radio(theme, size, chosen))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(candidate.label.clone()),
                    ),
            )
            .child(self.dispatch_meter(cpu(candidate), METER_WIDTH).w_full())
            .child(
                div()
                    .truncate()
                    .text_color(muted)
                    .child(free_cores(candidate).unwrap_or_else(|| fl!("dispatch-measuring"))),
            )
            .child(div().truncate().text_color(muted).child(agents(candidate)))
            .child(
                div()
                    .h(px(self.config.ui.line_height()))
                    .truncate()
                    .text_color(rgb(if first && !candidate.current {
                        theme.primary()
                    } else {
                        theme.muted
                    }))
                    .children(tag(candidate, first)),
            )
    }

    /// One host as a list row, choosing it when clicked.
    fn dispatch_row(
        &self,
        candidate: &Candidate,
        first: bool,
        chosen: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let hover = theme.active;
        let id = candidate.endpoint_id.clone();
        let detail = candidate.online.then(|| {
            let free = candidate
                .load
                .and_then(|load| load.spare_cores())
                .map_or_else(|| "?".to_owned(), |spare| format!("{spare:.0}"));
            fl!(
                "dispatch-row-detail",
                free = free,
                working = candidate.working
            )
        });
        let selector = format!("dispatch-row-{id}");
        div()
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(8.))
            .py(px(4.))
            .rounded(px(corners::SMALL))
            .when(chosen, |row| row.bg(rgb(theme.active)))
            .when(candidate.online, |row| {
                row.cursor_pointer()
                    .hover(move |row| row.bg(rgb(hover)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.pick_host(&id, cx);
                    }))
            })
            .when(!candidate.online, |row| row.opacity(0.45))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(candidate.label.clone()),
            )
            .children(
                tag(candidate, first)
                    .map(|tag| div().flex_none().text_color(rgb(theme.muted)).child(tag)),
            )
            .child(if candidate.online {
                self.dispatch_meter(cpu(candidate), METER_WIDTH)
            } else {
                div().w(px(METER_WIDTH)).flex_none()
            })
            .child(
                div()
                    .flex_none()
                    .w(px(self.config.ui.size * 9.))
                    .flex()
                    .justify_end()
                    .text_color(rgb(theme.subtext()))
                    .children(detail),
            )
    }

    /// A bordered list of hosts, opened from the dialog's field.
    fn dispatch_list<'a>(
        &self,
        hosts: impl Iterator<Item = &'a Candidate>,
        first: Option<&str>,
        chosen: &str,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        div()
            .debug_selector(|| "dispatch-list".into())
            .flex()
            .flex_col()
            .p(px(4.))
            .rounded(px(corners::CONTROL))
            .border_1()
            .border_color(rgb(theme.active))
            .bg(rgb(theme.surface))
            .children(hosts.map(|candidate| {
                let first = first == Some(candidate.endpoint_id.as_str());
                self.dispatch_row(candidate, first, candidate.endpoint_id == chosen, cx)
            }))
    }

    /// The dialog's host picker: the best hosts as tiles, then an "Other"
    /// field listing the rest. `caption` lays the field out like the form.
    pub(crate) fn render_host_picker(
        &self,
        picker: &Picker,
        caption: impl Fn(String, AnyElement) -> Div,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let chosen = picker
            .picked()
            .map(|c| c.endpoint_id.clone())
            .unwrap_or_default();
        let first = picker.ranked().next().map(|c| c.endpoint_id.clone());
        let mut tiles = div().flex().gap(px(6.));
        for candidate in picker.best() {
            let is_first = first.as_deref() == Some(candidate.endpoint_id.as_str());
            tiles = tiles.child(self.dispatch_tile(
                candidate,
                is_first,
                candidate.endpoint_id == chosen,
                cx,
            ));
        }
        let mut picker_view = div()
            .debug_selector(|| "dispatch-picker".into())
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(tiles);
        let rest = picker.rest().count();
        if rest == 0 {
            return picker_view;
        }
        let open = picker.open();
        let elsewhere = picker.chosen().filter(|c| !picker.is_best(&c.endpoint_id));
        let hover = theme.active;
        let field = div()
            .id("dispatch-other")
            .debug_selector(|| "dispatch-other".into())
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(10.))
            .py(px(5.))
            .rounded(px(corners::CONTROL))
            .border_1()
            .border_color(rgb(if elsewhere.is_some() {
                theme.primary()
            } else {
                theme.active
            }))
            .cursor_pointer()
            .hover(move |field| field.bg(rgb(hover)))
            .on_click(cx.listener(|this, _, _, cx| {
                cx.stop_propagation();
                this.toggle_host_list(cx);
            }))
            .child(match elsewhere {
                Some(chosen) => div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(chosen.label.clone()),
                None => div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(rgb(theme.muted))
                    .child(fl!("dispatch-more-hosts", count = rest)),
            })
            .children(elsewhere.map(|chosen| self.dispatch_meter(cpu(chosen), METER_WIDTH)))
            .child(
                svg()
                    .path(if open {
                        "icons/chevron-up.svg"
                    } else {
                        "icons/chevron-down.svg"
                    })
                    .size(px(12.))
                    .flex_none()
                    .text_color(rgb(theme.muted)),
            );
        picker_view = picker_view.child(caption(fl!("dispatch-other"), field.into_any_element()));
        if open {
            picker_view =
                picker_view.child(self.dispatch_list(picker.rest(), first.as_deref(), &chosen, cx));
        }
        picker_view
    }
}

// ABOUTME: Captures device picker display without endpoint or session action targets.
// ABOUTME: Preserves measured list positions and clipping after active owners reset.
use super::super::{activity_card::ActivityPaint, core_catalog::CatalogPaint};
use super::*;
use crate::menu::sessions::{NotePaint, SessionPaint};

pub(super) struct ChipPaint {
    pub(super) label: Option<String>,
    pub(super) spoken: String,
    pub(super) unfolded: bool,
}

pub(super) struct DevicePaint {
    pub(super) index: usize,
    pub(super) label: String,
    pub(super) detail: String,
    pub(super) selected: bool,
    pub(super) checked: bool,
    pub(super) enabled: bool,
    pub(super) leading: Option<&'static str>,
    pub(super) online: Option<bool>,
    pub(super) chip: Option<ChipPaint>,
}

pub(super) enum SessionLine {
    Row(SessionPaint),
    Note(NotePaint),
}

pub(super) struct SessionsPaint {
    pub(super) rows: Vec<SessionLine>,
    pub(super) opacity: f32,
}

pub(super) enum Content {
    Header,
    Device(DevicePaint),
    Sessions(SessionsPaint),
    Core(CatalogPaint),
}

pub(super) struct Block {
    pub(super) index: usize,
    pub(super) bounds: Bounds<Pixels>,
    pub(super) content: Content,
}

pub(in crate::menu::devices) struct PickerPaint {
    pub(super) activity: ActivityPaint,
    pub(super) blocks: Vec<Block>,
    pub(super) height: Pixels,
}

impl SheprdWindow {
    pub(in crate::menu::devices) fn devices_picker_paint(&self, cx: &App) -> PickerPaint {
        let lines = self.picker_lines();
        let catalog = self.core_machine_catalog_paint(cx);
        let without_fold = 1
            + lines
                .iter()
                .filter(|line| !matches!(line, Line::Session(_)))
                .count()
            + usize::from(lines.iter().any(|line| matches!(line, Line::Session(_))))
            + usize::from(catalog.is_some());
        // Capture the frame already laid out, even if motion was enabled only at dismissal.
        let folded_was_drawn = self.menu.devices_scroll.children_count() > without_fold;
        let mut blocks = Vec::new();
        let viewport = self.menu.devices_scroll.bounds();
        let offset = self.menu.devices_scroll.offset();
        let mut child = 0;
        let mut push = |mut content, opacity_height: Option<Pixels>| {
            if let Some(mut bounds) = self.menu.devices_scroll.bounds_for_item(child) {
                if let Content::Sessions(group) = &mut content
                    && let Some(height) = opacity_height.filter(|height| *height > px(0.))
                {
                    group.opacity =
                        (f32::from(bounds.size.height) / f32::from(height)).clamp(0., 1.);
                }
                bounds.origin = bounds.origin - viewport.origin + offset;
                if bounds.bottom() > px(0.) && bounds.top() < viewport.size.height {
                    blocks.push(Block {
                        index: child,
                        bounds,
                        content,
                    });
                }
            }
            child += 1;
        };
        push(Content::Header, None);
        let mut ordinal = 0;
        let mut row = 0;
        let mut unfolded = Vec::new();
        for line in lines {
            let selected = self.menu.selected == Some(ordinal);
            match &line {
                Line::Session(Entry::Row { row: choice, spec }) => {
                    unfolded.push(SessionLine::Row(
                        self.session_row_paint(row, choice, selected, spec),
                    ));
                    row += 1;
                }
                Line::Session(entry) => {
                    if let Some(note) = self.session_note_paint(entry) {
                        unfolded.push(SessionLine::Note(note));
                    }
                }
                _ => {
                    if !unfolded.is_empty() {
                        push(
                            Content::Sessions(SessionsPaint {
                                rows: std::mem::take(&mut unfolded),
                                opacity: 1.,
                            }),
                            Some(self.menu.unfolded_height.get()),
                        );
                    }
                    if let Some(index) = self.device_row(&line) {
                        push(
                            Content::Device(self.device_row_paint(&line, index, selected)),
                            None,
                        );
                        if let Line::Device(index) = line
                            && folded_was_drawn
                            && let Some(folding) = self.menu.folding.as_ref().filter(|folding| {
                                self.endpoints
                                    .get(index)
                                    .is_some_and(|endpoint| endpoint.id == folding.device)
                            })
                        {
                            let mut rows = Vec::new();
                            let mut row = 0;
                            for entry in self.device_session_entries(index) {
                                match &entry {
                                    Entry::Row { row: choice, spec } => {
                                        rows.push(SessionLine::Row(
                                            self.session_row_paint(row, choice, false, spec),
                                        ));
                                        row += 1;
                                    }
                                    _ => {
                                        if let Some(note) = self.session_note_paint(&entry) {
                                            rows.push(SessionLine::Note(note));
                                        }
                                    }
                                }
                            }
                            push(
                                Content::Sessions(SessionsPaint { rows, opacity: 1. }),
                                Some(folding.height),
                            );
                        }
                    }
                }
            }
            if line.selectable() {
                ordinal += 1;
            }
        }
        if !unfolded.is_empty() {
            push(
                Content::Sessions(SessionsPaint {
                    rows: unfolded,
                    opacity: 1.,
                }),
                Some(self.menu.unfolded_height.get()),
            );
        }
        if let Some(catalog) = catalog {
            push(Content::Core(catalog), None);
        }
        PickerPaint {
            activity: self.activity_card_paint(),
            blocks,
            height: viewport.size.height,
        }
    }

    pub(super) fn device_row_paint(
        &self,
        line: &Line,
        index: usize,
        selected: bool,
    ) -> DevicePaint {
        let endpoint = match line {
            Line::Device(device) => self.endpoints.get(*device),
            _ => None,
        };
        let (label, detail, checked, enabled) = match (line, endpoint) {
            (Line::All, _) => (
                fl!("devices-all"),
                fl!(
                    "devices-all-detail",
                    count = self.endpoints.len(),
                    connected = self
                        .endpoints
                        .iter()
                        .filter(|endpoint| endpoint.live.status.is_connected())
                        .count()
                ),
                self.device_filter == DeviceScope::All,
                true,
            ),
            (_, Some(endpoint)) => (
                endpoint.label.clone(),
                fl!(
                    "devices-row-detail",
                    place = Self::device_detail(&endpoint.connection.target),
                    state = endpoint.status_label()
                ),
                self.device_filter.picks(&endpoint.id),
                endpoint.enabled(),
            ),
            _ => (
                fl!("devices-add"),
                self.device_setup_unavailable()
                    .map_or_else(add_device_offer, SetupUnavailable::text),
                false,
                self.device_setup_unavailable().is_none(),
            ),
        };
        DevicePaint {
            index,
            label,
            detail,
            selected,
            checked,
            enabled,
            leading: match line {
                Line::All => Some("icons/devices.svg"),
                Line::AddDevice => Some("icons/plus.svg"),
                _ => None,
            },
            online: endpoint.map(|endpoint| endpoint.live.status.is_connected()),
            chip: endpoint.map(|endpoint| ChipPaint {
                label: match &endpoint.connection.target {
                    ConnectTarget::Socket(_) => None,
                    target => Some(target_session(target)),
                },
                spoken: fl!("devices-sessions-toggle", device = endpoint.label.as_str()),
                unfolded: self.menu.unfolded_device.as_ref() == Some(&endpoint.id),
            }),
        }
    }
}

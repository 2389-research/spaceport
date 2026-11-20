// ABOUTME: Captures the device theme's visible choices before the host owner drops.
// ABOUTME: Keeps input paint and plain offsets separate from preview and save tasks.
use super::*;

pub(super) struct Choice {
    pub(super) index: usize,
    pub(super) label: String,
    pub(super) selected: bool,
    pub(super) current: bool,
}

pub(in crate::menu) struct Paint {
    pub(super) label: String,
    pub(super) input: crate::search_input::Paint,
    pub(super) rows: Vec<Choice>,
    pub(super) row_height: Pixels,
    pub(super) offset: Pixels,
    pub(super) status: String,
}

pub(super) fn choice(row: &Row, index: usize, selected: usize, current: &Row) -> Choice {
    Choice {
        index,
        label: match row {
            Row::App => crate::i18n::message(SAME_AS_APP, &[]),
            Row::Named(name) => name.clone(),
        },
        selected: index == selected,
        current: row == current,
    }
}

pub(super) fn status(view: &SheprdWindow, host: &HostMenu, picker: &DeviceThemePicker) -> String {
    if let Some(error) = &picker.error {
        return error.clone();
    }
    if view
        .endpoints
        .get(view.selected_endpoint)
        .is_some_and(|endpoint| endpoint.id == host.id)
    {
        fl!("host-menu-theme-hover-hint")
    } else {
        fl!("host-menu-theme-select-hint")
    }
}

impl SheprdWindow {
    pub(in crate::menu) fn device_theme_paint(&self, cx: &App) -> Option<Paint> {
        if self.menu.page != Some(Page::DeviceTheme) {
            return None;
        }
        let host = self.menu.host.as_ref()?;
        let picker = host.theme.as_ref()?;
        let scroll = picker.scroll.0.borrow();
        let height = scroll
            .last_item_size
            .filter(|_| !picker.filtered.is_empty())
            .map_or(self.config.ui.line_height() + 20., |size| {
                f32::from(size.contents.height) / picker.filtered.len() as f32
            })
            .max(1.);
        let offset = scroll.base_handle.offset().y;
        let start =
            ((-f32::from(offset) / height).floor().max(0.) as usize).min(picker.filtered.len());
        let length = (f32::from(scroll.base_handle.bounds().size.height).max(0.) / height).ceil()
            as usize
            + 1;
        let end = start.saturating_add(length).min(picker.filtered.len());
        let current = current_row(
            self.device_theme_value(&host.id),
            crate::app::light_appearance(cx),
        );
        Some(Paint {
            label: host.label.clone(),
            input: picker.search.read(cx).paint(),
            rows: (start..end)
                .map(|index| choice(&picker.filtered[index], index, picker.selected, &current))
                .collect(),
            row_height: px(height),
            offset: offset + px(height * start as f32),
            status: status(self, host, picker),
        })
    }
}

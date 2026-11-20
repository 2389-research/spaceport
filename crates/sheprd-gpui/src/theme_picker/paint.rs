// ABOUTME: Captures visible theme choices without retaining native input or readers.
// ABOUTME: Preserves list and status offsets as plain data for popup departure.
use super::*;

pub(super) struct Row {
    pub(super) index: usize,
    pub(super) name: String,
    pub(super) selected: bool,
    pub(super) current: bool,
    pub(super) favorite: bool,
    pub(super) last_favorite: bool,
}

pub(crate) struct Paint {
    input: crate::search_input::Paint,
    pub(super) rows: Vec<Row>,
    count: usize,
    total: usize,
    status: String,
    pub(super) row_height: Pixels,
    pub(super) offset: Pixels,
    pub(super) status_offset: Pixels,
}

#[derive(Clone, Copy)]
pub(super) enum Content<'a> {
    Live(&'a ThemePicker),
    Frozen(&'a Paint),
}

impl SheprdWindow {
    pub(crate) fn theme_picker_paint(&self, cx: &App) -> Option<Paint> {
        if self.menu.page != Some(Page::Themes) {
            return None;
        }
        let picker = self.menu.themes.as_ref()?;
        let input = picker.search()?.read(cx).paint();
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
        Some(Paint {
            input,
            rows: (start..end)
                .filter_map(|index| self.theme_picker_row_data(picker, index, cx))
                .collect(),
            count: picker.filtered.len(),
            total: picker.names.len(),
            status: Content::Live(picker).status(),
            row_height: px(height),
            offset: offset + px(height * start as f32),
            status_offset: picker.status_scroll.offset().y,
        })
    }

    pub(super) fn theme_picker_row_data(
        &self,
        picker: &ThemePicker,
        index: usize,
        cx: &App,
    ) -> Option<Row> {
        let name = picker.filtered.get(index)?.clone();
        Some(Row {
            current: name == ThemeName::side(&self.config.theme, crate::app::light_appearance(cx)),
            name,
            index,
            selected: index == picker.selected,
            favorite: index < picker.favorites,
            // A rule closes the starred group when other themes follow it.
            last_favorite: index + 1 == picker.favorites
                && picker.favorites < picker.filtered.len(),
        })
    }
}

impl Content<'_> {
    pub(super) fn active(self) -> bool {
        matches!(self, Self::Live(_))
    }
    pub(super) fn count(self) -> usize {
        match self {
            Self::Live(p) => p.filtered.len(),
            Self::Frozen(p) => p.count,
        }
    }
    pub(super) fn total(self) -> usize {
        match self {
            Self::Live(p) => p.names.len(),
            Self::Frozen(p) => p.total,
        }
    }
    pub(super) fn input(self) -> Option<AnyElement> {
        match self {
            Self::Live(p) => p.search().map(|input| input.clone().into_any_element()),
            Self::Frozen(p) => Some(p.input.render().into_any_element()),
        }
    }
    pub(super) fn status(self) -> String {
        match self {
            Self::Frozen(p) => p.status.clone(),
            Self::Live(p) => {
                if p.saving {
                    fl!("theme-picker-saving")
                } else if let Some(error) = p.error.as_ref().or(p.favorites_error.as_ref()) {
                    error.clone()
                } else if p.accepting {
                    fl!("theme-picker-loading")
                } else {
                    fl!("theme-picker-hint")
                }
            }
        }
    }
}

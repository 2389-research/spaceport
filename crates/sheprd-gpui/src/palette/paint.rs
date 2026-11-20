// ABOUTME: Freezes the palette's visible content without retaining background work.
// ABOUTME: Shares rendering with live palettes while omitting all action ownership.
use super::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub(super) struct Row {
    pub label: SharedString,
    pub detail: SharedString,
    pub badge: SharedString,
    pub depth: usize,
    pub highlights: search::Highlights,
}

#[derive(Clone)]
pub(crate) struct Paint {
    input: crate::search_input::Paint,
    rows: Rc<Vec<Row>>,
    count: usize,
    pub(super) visible: std::ops::Range<usize>,
    pub(super) offset: Pixels,
    pub(super) row_height: Pixels,
    total: usize,
    selected: usize,
    filter: Filter,
    agent_filter: AgentFilter,
    error: Option<String>,
    project_errors: String,
    loading: bool,
    busy: bool,
    preview: Option<preview::Preview>,
    painter: Rc<RefCell<crate::terminal_painter::TerminalPainter>>,
    pub(crate) previews: bool,
}

#[derive(Clone, Copy)]
pub(super) enum Content<'a> {
    Live(&'a Palette),
    Frozen(&'a Paint),
}

impl SheprdWindow {
    pub(crate) fn palette_paint(&self, viewport: Size<Pixels>, cx: &App) -> Option<Paint> {
        let palette = self.menu.palette.as_ref()?;
        let content = Content::Live(palette);
        let scroll = palette.scroll.0.borrow();
        let height = scroll
            .last_item_size
            .filter(|_| !palette.filtered.is_empty())
            .map_or(self.config.ui.line_height() * 2. + 20., |size| {
                f32::from(size.contents.height) / palette.filtered.len() as f32
            });
        let offset = scroll.base_handle.offset().y;
        let start =
            ((-f32::from(offset) / height).floor().max(0.) as usize).min(palette.filtered.len());
        let length = (f32::from(viewport.height).max(0.) / height).ceil() as usize + 1;
        let visible = start..start.saturating_add(length).min(palette.filtered.len());
        Some(Paint {
            input: palette.search.read(cx).paint(),
            rows: Rc::new(visible.clone().map(|index| content.row(index)).collect()),
            count: palette.filtered.len(),
            offset: offset + px(height * start as f32),
            row_height: px(height),
            visible,
            total: palette.entries.len(),
            selected: palette.selected,
            filter: palette.filter,
            agent_filter: palette.agent_filter,
            error: palette.error.clone(),
            project_errors: content.project_errors(),
            loading: palette.loading_projects,
            busy: palette.busy(),
            preview: self.palette_preview(palette),
            painter: palette.preview_painter.clone(),
            previews: self.palette_previews(viewport),
        })
    }
}

impl Content<'_> {
    pub(super) fn active(self) -> bool {
        matches!(self, Self::Live(_))
    }
    pub(super) fn input(self) -> AnyElement {
        match self {
            Self::Live(p) => p.search.clone().into_any_element(),
            Self::Frozen(p) => p.input.render().into_any_element(),
        }
    }
    pub(super) fn count(self) -> usize {
        match self {
            Self::Live(p) => p.filtered.len(),
            Self::Frozen(p) => p.count,
        }
    }
    pub(super) fn total(self) -> usize {
        match self {
            Self::Live(p) => p.entries.len(),
            Self::Frozen(p) => p.total,
        }
    }
    pub(super) fn selected(self) -> usize {
        match self {
            Self::Live(p) => p.selected,
            Self::Frozen(p) => p.selected,
        }
    }
    pub(super) fn filter(self) -> Filter {
        match self {
            Self::Live(p) => p.filter,
            Self::Frozen(p) => p.filter,
        }
    }
    pub(super) fn agent_filter(self) -> AgentFilter {
        match self {
            Self::Live(p) => p.agent_filter,
            Self::Frozen(p) => p.agent_filter,
        }
    }
    pub(super) fn loading(self) -> bool {
        match self {
            Self::Live(p) => p.loading_projects,
            Self::Frozen(p) => p.loading,
        }
    }
    pub(super) fn busy(self) -> bool {
        match self {
            Self::Live(p) => p.busy(),
            Self::Frozen(p) => p.busy,
        }
    }
    pub(super) fn error(self) -> Option<String> {
        match self {
            Self::Live(p) => p.error.clone(),
            Self::Frozen(p) => p.error.clone(),
        }
    }
    pub(super) fn painter(self) -> Rc<RefCell<crate::terminal_painter::TerminalPainter>> {
        match self {
            Self::Live(p) => p.preview_painter.clone(),
            Self::Frozen(p) => p.painter.clone(),
        }
    }
    pub(super) fn project_errors(self) -> String {
        match self {
            Self::Live(p) => p
                .projects
                .errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Frozen(p) => p.project_errors.clone(),
        }
    }
    pub(super) fn action(self, index: usize) -> Option<Action> {
        match self {
            Self::Live(p) => Some(p.entries[p.filtered[index].index].action.clone()),
            Self::Frozen(_) => None,
        }
    }
    pub(super) fn preview(self, view: &SheprdWindow) -> Option<preview::Preview> {
        match self {
            Self::Live(p) => view.palette_preview(p),
            Self::Frozen(p) => p.preview.clone(),
        }
    }
    pub(super) fn row(self, index: usize) -> Row {
        match self {
            Self::Live(p) => {
                let hit = &p.filtered[index];
                let entry = &p.entries[hit.index];
                Row {
                    label: entry.label.clone(),
                    detail: entry.detail.clone(),
                    badge: entry.badge.clone(),
                    depth: if p.query.trim().is_empty() {
                        hit.depth
                    } else {
                        0
                    },
                    highlights: hit.highlights.clone(),
                }
            }
            Self::Frozen(p) => {
                let row = &p.rows[index - p.visible.start];
                Row {
                    label: row.label.clone(),
                    detail: row.detail.clone(),
                    badge: row.badge.clone(),
                    depth: row.depth,
                    highlights: row.highlights.clone(),
                }
            }
        }
    }
}

// ABOUTME: Captures visible tab menu data without retaining tab targets or field owners.
// ABOUTME: Distinguishes action rows, rename drafts, move choices, and submitted work.
use super::*;

pub(super) enum Draft {
    Editing(Option<Box<crate::search_input::Paint>>),
    Pending(Option<SharedString>),
}

pub(super) enum Kind {
    Actions(Vec<SharedString>),
    Rename(Draft),
    Destinations(Vec<SharedString>),
    Moving,
}

pub(crate) struct Paint {
    pub(super) kind: Kind,
    pub(super) selected: Option<usize>,
    pub(super) error: Option<String>,
    pub(super) offset: Point<Pixels>,
}

impl SheprdWindow {
    pub(crate) fn tab_menu_paint(&self, cx: &App) -> Option<Paint> {
        let tab = self.menu.tab.as_ref()?;
        let kind = if tab.moving.is_some() {
            Kind::Moving
        } else {
            match self.menu.page? {
                Page::Tab => Kind::Actions(
                    tab.actions()
                        .into_iter()
                        .map(|action| action.label().into())
                        .collect(),
                ),
                Page::RenameTab => Kind::Rename(if tab.pending.is_some() {
                    Draft::Pending(
                        tab.input
                            .as_ref()
                            .map(|input| input.read(cx).text().to_owned().into()),
                    )
                } else {
                    Draft::Editing(
                        tab.input
                            .as_ref()
                            .map(|input| Box::new(input.read(cx).paint())),
                    )
                }),
                Page::MoveTab => Kind::Destinations(
                    tab.target
                        .choices
                        .iter()
                        .map(|choice| choice.place.clone().into())
                        .collect(),
                ),
                _ => return None,
            }
        };
        let offset = if matches!(kind, Kind::Moving) {
            Point::default()
        } else {
            tab.scroll.offset()
        };
        Some(Paint {
            kind,
            selected: tab.selected,
            error: tab.error.clone(),
            offset,
        })
    }

    pub(crate) fn tab_shell_scroll(&self) -> Option<ScrollHandle> {
        if !matches!(self.menu.page, Some(Page::Tab | Page::RenameTab)) {
            return None;
        }
        self.menu.tab.as_ref().map(|tab| tab.scroll.clone())
    }
}

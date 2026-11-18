// ABOUTME: Captures custom application-menu display without retaining its action tree.
// ABOUTME: Keeps anchors and scroll offsets as plain values after menu ownership ends.
use super::*;
use crate::i18n::fl;

pub(super) enum Row {
    Separator,
    Item {
        name: SharedString,
        disabled: bool,
        checked: bool,
        detail: SharedString,
    },
}

pub(crate) struct Paint {
    pub(super) rows: Vec<Row>,
    pub(super) selected: Option<usize>,
    pub(super) title: SharedString,
    pub(super) back: Option<SharedString>,
    pub(super) offset: Point<Pixels>,
    pub(super) button: Bounds<Pixels>,
    pub(super) bar: Bounds<Pixels>,
    pub(super) content: Bounds<Pixels>,
}

impl SheprdWindow {
    pub(crate) fn application_menu_paint(&self, window: &Window, cx: &App) -> Option<Paint> {
        self.application_menu_paint_for(self.compact_application_bar(window), cx)
    }

    /// Config invalidation has no Window; its anchor comes from the last measured bar.
    pub(crate) fn invalidated_application_menu_paint(&self, cx: &App) -> Option<Paint> {
        let compact = self.menu.application_bar.buttons.borrow().len() == 1;
        self.application_menu_paint_for(compact, cx)
    }

    fn application_menu_paint_for(&self, compact: bool, cx: &App) -> Option<Paint> {
        let menu = self.menu.application.as_ref()?;
        let current = menu.current();
        let rows = match current {
            Some(menu) => menu
                .items
                .iter()
                .map(|item| self.application_row(item, cx))
                .collect(),
            None => menu
                .menus
                .iter()
                .map(|menu| Row::Item {
                    name: menu.name.clone(),
                    disabled: menu.disabled,
                    checked: false,
                    detail: "›".into(),
                })
                .collect(),
        };
        let bar = self.menu.application_bar.bar.get();
        let button = self
            .menu
            .application_bar
            .buttons
            .borrow()
            .get(if compact {
                0
            } else {
                menu.path.first().copied().unwrap_or(0)
            })
            .copied()
            .unwrap_or(bar);
        Some(Paint {
            rows,
            selected: menu.selected,
            title: current.map_or_else(|| fl!("menus-menu").into(), |menu| menu.name.clone()),
            back: current.map(|menu| menu.name.clone()),
            offset: menu.scroll.offset(),
            button,
            bar,
            content: self.menu.application_bar.content.get(),
        })
    }

    fn application_row(&self, item: &OwnedMenuItem, cx: &App) -> Row {
        match item {
            OwnedMenuItem::Separator => Row::Separator,
            OwnedMenuItem::Submenu(menu) => Row::Item {
                name: menu.name.clone(),
                disabled: menu.disabled,
                checked: false,
                detail: "›".into(),
            },
            OwnedMenuItem::Action {
                name,
                action,
                disabled,
                checked,
                ..
            } => Row::Item {
                name: name.clone().into(),
                disabled: *disabled,
                checked: *checked,
                detail: self.application_shortcut(action.as_ref(), cx).into(),
            },
            OwnedMenuItem::SystemMenu(menu) => Row::Item {
                name: menu.name.clone(),
                disabled: true,
                checked: false,
                detail: "".into(),
            },
        }
    }
}

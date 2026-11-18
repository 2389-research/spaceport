// ABOUTME: Captures pane display values without active targets or monitor ownership.
// ABOUTME: Preserves action labels, drafts, destinations, and process projections during departure.
use super::*;

pub(super) enum Draft {
    // keep-sorted start
    Editing(Option<Box<crate::search_input::Paint>>),
    Pending(Option<SharedString>),
    // keep-sorted end
}

pub(super) struct DestinationPaint {
    pub(super) workspace: Option<String>,
    pub(super) place: String,
}

pub(super) enum Kind {
    // keep-sorted start
    Actions(Vec<SharedString>),
    Confirmation(processes::Confirmation),
    Destinations(Vec<DestinationPaint>),
    Moving,
    Processes(processes::Paint),
    Rename(Draft),
    // keep-sorted end
}

pub(crate) struct Paint {
    pub(super) kind: Kind,
    pub(super) selected: Option<usize>,
    pub(super) error: Option<String>,
    pub(super) offset: Point<Pixels>,
}

impl SheprdWindow {
    pub(crate) fn pane_menu_paint(&self, cx: &App) -> Option<Paint> {
        let pane = self.menu.pane.as_ref()?;
        let kind = match self.menu.page? {
            Page::PaneProcesses => Kind::Processes(processes::Paint::capture(pane)?),
            Page::KillProcesses => Kind::Confirmation(processes::Confirmation::capture(pane)?),
            _ if pane.moving.is_some() => Kind::Moving,
            Page::Pane => Kind::Actions(
                pane.actions()
                    .into_iter()
                    .map(|action| action.label(&pane.target).into())
                    .collect(),
            ),
            Page::RenamePane => Kind::Rename(if pane.pending.is_some() {
                Draft::Pending(
                    pane.input
                        .as_ref()
                        .map(|input| input.read(cx).text().to_owned().into()),
                )
            } else {
                Draft::Editing(
                    pane.input
                        .as_ref()
                        .map(|input| Box::new(input.read(cx).paint())),
                )
            }),
            Page::MovePane => Kind::Destinations(
                pane.target
                    .destinations()
                    .into_iter()
                    .map(|choice| DestinationPaint {
                        workspace: choice.workspace,
                        place: choice.place,
                    })
                    .collect(),
            ),
            _ => return None,
        };
        let offset = if matches!(kind, Kind::Moving) {
            Point::default()
        } else {
            pane.scroll.offset()
        };
        Some(Paint {
            kind,
            selected: pane.selected,
            error: pane.error.clone(),
            offset,
        })
    }

    pub(crate) fn pane_shell_scroll(&self) -> Option<ScrollHandle> {
        if !matches!(
            self.menu.page,
            Some(Page::Pane | Page::RenamePane | Page::PaneProcesses | Page::KillProcesses)
        ) {
            return None;
        }
        self.menu.pane.as_ref().map(|pane| pane.scroll.clone())
    }
}

// ABOUTME: Prepares a process confirmation with no worker for native key checks.
// ABOUTME: Exposes bounded assertions without starting or signalling any process.
use super::*;

impl SheprdWindow {
    pub(crate) fn prepare_process_confirmation_check(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(pane) = self
            .live
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.focused_pane_id.clone())
        else {
            return false;
        };
        self.open_pane_menu(&pane, Point::default(), window, cx);
        let Some(pane) = self.menu.pane.as_mut() else {
            return false;
        };
        let identity = Identity {
            pid: 20,
            started: 1,
        };
        pane.processes = Some(PaneProcesses {
            watch: None,
            listing: Some(Listing {
                rows: vec![crate::processes::Row {
                    entry: crate::processes::Entry {
                        identity,
                        parent: None,
                        name: "native-fixture".into(),
                        command: "native-fixture".into(),
                        cpu: None,
                        memory: 0,
                    },
                    depth: 1,
                    foreground: false,
                }],
                hidden: 0,
                cpu: 0.,
                memory: 0,
            }),
            chosen: BTreeSet::from([identity]),
            cursor: None,
            scroll: ScrollHandle::new(),
            error: None,
            report: None,
            ending: false,
        });
        self.menu.page = Some(Page::KillProcesses);
        cx.notify();
        true
    }

    pub(crate) fn process_confirmation_untouched(&self) -> bool {
        self.menu.page == Some(Page::KillProcesses)
            && self
                .menu
                .pane
                .as_ref()
                .and_then(|pane| pane.processes.as_ref())
                .is_some_and(|processes| {
                    processes.watch.is_none()
                        && processes.error.is_none()
                        && !processes.ending
                        && !processes.chosen.is_empty()
                })
    }
}

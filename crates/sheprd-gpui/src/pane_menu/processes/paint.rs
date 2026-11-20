// ABOUTME: Prepares bounded process rows and confirmation labels for shared rendering.
// ABOUTME: Retains no process identity, watch, worker channel, or live scroll handle.
use super::*;

pub(in crate::pane_menu) struct Row {
    pub(super) name: String,
    pub(super) command: String,
    pub(super) pid: String,
    pub(super) cpu: String,
    pub(super) memory: String,
    pub(super) depth: usize,
    pub(super) foreground: bool,
    pub(super) chosen: bool,
    pub(super) choosable: bool,
}

pub(in crate::pane_menu) struct Paint {
    pub(super) title: String,
    pub(super) total: Option<String>,
    pub(super) rows: Option<Vec<Row>>,
    pub(super) hidden: Option<String>,
    pub(super) cursor: Option<usize>,
    pub(super) loading: bool,
    pub(super) status: Option<String>,
    pub(super) end: String,
    pub(super) enabled: bool,
    pub(super) offset: Point<Pixels>,
}

pub(in crate::pane_menu) struct Confirmation {
    pub(super) title: String,
    pub(super) rows: Vec<(String, String)>,
    pub(super) confirm: String,
}

impl Paint {
    pub(in crate::pane_menu) fn capture(pane: &PaneMenu) -> Option<Self> {
        let processes = pane.processes.as_ref()?;
        let count = processes.chosen.len();
        Some(Self {
            title: if pane.target.label.is_empty() {
                fl!("pane-menu-processes-title")
            } else {
                fl!("pane-menu-processes-in", pane = pane.target.label.as_str())
            },
            total: processes.listing.as_ref().map(|listing| {
                fl!(
                    "pane-menu-processes-total",
                    cpu = cpu_text(Some(listing.cpu)),
                    memory = memory_text(listing.memory)
                )
            }),
            rows: processes.listing.as_ref().map(|listing| {
                listing
                    .rows
                    .iter()
                    .map(|row| Row {
                        name: row.entry.name.clone(),
                        command: row.entry.command.clone(),
                        pid: row.entry.identity.pid.to_string(),
                        cpu: cpu_text(row.entry.cpu),
                        memory: memory_text(row.entry.memory),
                        depth: row.depth,
                        foreground: row.foreground,
                        chosen: processes.chosen.contains(&row.entry.identity),
                        choosable: Some(row.entry.identity) != listing.root(),
                    })
                    .collect()
            }),
            hidden: processes
                .listing
                .as_ref()
                .filter(|listing| listing.hidden > 0)
                .map(|listing| fl!("pane-menu-processes-hidden", count = listing.hidden)),
            cursor: processes.cursor,
            loading: processes.listing.is_none() && processes.error.is_none(),
            status: processes
                .error
                .clone()
                .or_else(|| processes.ending.then(|| fl!("pane-menu-processes-ending")))
                .or_else(|| processes.report.clone()),
            end: fl!("pane-menu-processes-end", count = count),
            enabled: count > 0 && !processes.ending,
            offset: processes.scroll.offset(),
        })
    }
}

impl Confirmation {
    pub(in crate::pane_menu) fn capture(pane: &PaneMenu) -> Option<Self> {
        let processes = pane.processes.as_ref()?;
        let count = processes.chosen.len();
        Some(Self {
            title: fl!("pane-menu-processes-confirm-title", count = count),
            rows: processes
                .chosen_rows()
                .map(|row| {
                    (
                        row.entry.name.clone(),
                        fl!("pane-menu-processes-pid", pid = row.entry.identity.pid),
                    )
                })
                .collect(),
            confirm: fl!("pane-menu-processes-confirm", count = count),
        })
    }
}

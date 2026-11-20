// ABOUTME: Captures bounded Core machine and group display without selection targets.
// ABOUTME: Keeps active machine identities separate from the departing catalog values.
use super::*;

pub(super) enum Groups {
    Ready {
        labels: Vec<String>,
        more: Option<String>,
    },
    Message {
        selector: &'static str,
        text: String,
    },
}

pub(super) struct Machine {
    pub(super) label: String,
    pub(super) detail: String,
    pub(super) expanded: bool,
    pub(super) connected: bool,
    pub(super) groups: Option<Groups>,
}

pub(super) enum Body {
    Machines {
        rows: Vec<Machine>,
        more: Option<String>,
    },
    Unavailable {
        reason: String,
        stale: bool,
    },
}

pub(in crate::menu::devices) struct CatalogPaint {
    pub(super) body: Body,
}

impl SheprdWindow {
    pub(super) fn prepare_core_catalog(&self, cx: &App) -> Option<(CatalogPaint, Vec<String>)> {
        if !self.core_enabled() || !core_terminals::machine_catalog_started(cx) {
            return None;
        }
        let machines = core_terminals::machine_catalog(cx);
        let selected = core_terminals::selected_remote_machine(cx);
        let mut actions = Vec::new();
        let body = if let Some(machines) = machines {
            let rows = machines
                .iter()
                .take(MAX_DISPLAY_MACHINES)
                .map(|machine| {
                    actions.push(machine.machine.id.clone());
                    let expanded = selected.as_deref() == Some(machine.machine.id.as_str());
                    let groups = expanded.then(|| {
                        match core_terminals::remote_document_status(cx, &machine.machine.id) {
                            Some(core_terminals::RemoteDocumentView::Ready(workspaces)) => {
                                Groups::Ready {
                                    labels: workspaces
                                        .iter()
                                        .take(20)
                                        .map(|group| short(&group.label))
                                        .collect(),
                                    more: (workspaces.len() > 20).then(|| {
                                        fl!(
                                            "core-terminals-machine-groups-more",
                                            count = workspaces.len() - 20
                                        )
                                    }),
                                }
                            }
                            Some(core_terminals::RemoteDocumentView::Stale) => Groups::Message {
                                selector: "core-remote-groups-stale",
                                text: fl!("core-terminals-machine-groups-stale"),
                            },
                            Some(core_terminals::RemoteDocumentView::Failed(error)) => {
                                Groups::Message {
                                    selector: "core-remote-groups-error",
                                    text: fl!(
                                        "core-terminals-machine-groups-error",
                                        reason = short(error)
                                    ),
                                }
                            }
                            Some(core_terminals::RemoteDocumentView::Loading) | None => {
                                Groups::Message {
                                    selector: "core-remote-groups-loading",
                                    text: fl!("core-terminals-machine-groups-loading"),
                                }
                            }
                        }
                    });
                    Machine {
                        label: short(&machine.machine.name),
                        detail: format!(
                            "{} · {}",
                            short(&machine.machine.target),
                            short(&machine.state)
                        ),
                        expanded,
                        connected: machine.state == "connected",
                        groups,
                    }
                })
                .collect();
            Body::Machines {
                rows,
                more: (machines.len() > MAX_DISPLAY_MACHINES).then(|| {
                    fl!(
                        "core-terminals-machine-more",
                        count = machines.len() - MAX_DISPLAY_MACHINES
                    )
                }),
            }
        } else {
            Body::Unavailable {
                reason: fl!(
                    "core-terminals-machine-unavailable",
                    reason = unavailable_reason(core_terminals::machine_catalog_error(cx))
                ),
                stale: selected.is_some(),
            }
        };
        Some((CatalogPaint { body }, actions))
    }
}

// ABOUTME: Captures WSL setup rows and confirmation text without effect ownership.
// ABOUTME: Keeps only visible distribution labels and numeric body/list offsets.
use super::*;

pub(super) struct Choice {
    pub(super) index: usize,
    pub(super) distro: String,
    pub(super) added: bool,
    pub(super) checked: bool,
}

pub(super) enum Rows {
    // keep-sorted start
    Choices {
        rows: Vec<Choice>,
        before: Pixels,
        after: Pixels,
    },
    Message {
        text: String,
        danger: bool,
    },
    // keep-sorted end
}

pub(in crate::menu::devices) struct Paint {
    pub(super) rail: super::super::rail::Paint,
    pub(super) rows: Rows,
    pub(super) session: crate::search_input::Paint,
    pub(super) error: Option<String>,
    pub(super) status: Option<String>,
    pub(super) ready: bool,
    pub(super) checking: bool,
    pub(super) offset: Point<Pixels>,
}

pub(in crate::menu::devices) struct RemovalPaint {
    pub(super) note: String,
    pub(super) offset: Point<Pixels>,
}

impl SheprdWindow {
    pub(in crate::menu::devices) fn add_wsl_paint(&self, frozen: bool, cx: &App) -> Option<Paint> {
        let setup = self.menu.wsl_setup.as_ref()?;
        let saved = self.saved_distros();
        let rows = match &setup.distros {
            Distros::Loading => Rows::Message {
                text: fl!("devices-wsl-looking"),
                danger: false,
            },
            Distros::Failed(error) => Rows::Message {
                text: fl!("devices-wsl-list-failed", error = error.as_str()),
                danger: true,
            },
            Distros::Ready(distros) if distros.is_empty() => Rows::Message {
                text: fl!("devices-wsl-none"),
                danger: false,
            },
            Distros::Ready(distros) => {
                let count = distros.len();
                let bounds = setup.list_bounds.get();
                let gap = px(2.);
                let height = if bounds.size.height > px(0.) {
                    (bounds.size.height - gap * (count - 1) as f32) / count as f32
                } else {
                    px(self.config.ui.line_height() + 12.)
                };
                let stride = height + gap;
                let (start, end) = if frozen && bounds.size.height > px(0.) {
                    let viewport = setup.scroll.bounds();
                    let start = ((viewport.top() - bounds.top()) / stride).floor().max(0.) as usize;
                    let end = ((viewport.bottom() - bounds.top()) / stride).ceil().max(0.) as usize;
                    (start.min(count), end.min(count))
                } else {
                    (0, count)
                };
                Rows::Choices {
                    rows: (start..end)
                        .map(|index| Choice {
                            index,
                            distro: distros[index].clone(),
                            added: saved.contains(&distros[index]),
                            checked: setup.selected.as_ref() == Some(&distros[index]),
                        })
                        .collect(),
                    before: if start > 0 {
                        stride * start as f32 - gap
                    } else {
                        px(0.)
                    },
                    after: if end < count {
                        stride * (count - end) as f32 - gap
                    } else {
                        px(0.)
                    },
                }
            }
        };
        Some(Paint {
            rail: self.device_rail_paint(super::super::rail::Source::Wsl),
            rows,
            session: setup.session.read(cx).paint(),
            error: setup.error.clone(),
            status: match &setup.step {
                Step::Checking(distro) => {
                    Some(fl!("devices-wsl-checking", distro = distro.as_str()))
                }
                Step::Form => None,
            },
            ready: setup.step == Step::Form && setup.selected.is_some(),
            checking: matches!(setup.step, Step::Checking(_)),
            offset: setup.scroll.offset(),
        })
    }

    pub(in crate::menu::devices) fn remove_wsl_paint(&self) -> Option<RemovalPaint> {
        let distro = self.menu.wsl_remove.as_ref()?;
        Some(RemovalPaint {
            note: fl!("devices-wsl-remove-note", distro = distro.as_str()),
            offset: self.menu.wsl_remove_scroll.offset(),
        })
    }
}

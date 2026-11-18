// ABOUTME: Captures popup ownership for legacy, Core, and client-only windows.
// ABOUTME: Keeps client action validity separate from legacy runtime authorization.
use crate::{SheprdWindow, browser::CoreGroupKey};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Origin {
    // keep-sorted start
    Core { epoch: u64, key: CoreGroupKey },
    Legacy { epoch: u64, generation: u64 },
    Window { epoch: u64 },
    // keep-sorted end
}

impl Origin {
    pub(super) fn capture(view: &SheprdWindow) -> Self {
        let epoch = view.selection_epoch;
        if let Some(endpoint) = view.endpoints.get(view.selected_endpoint) {
            Self::Legacy {
                epoch,
                generation: endpoint.generation,
            }
        } else if let Some(key) = view.selected_core_group() {
            Self::Core {
                epoch,
                key: key.clone(),
            }
        } else {
            Self::Window { epoch }
        }
    }

    pub(super) fn current(&self, view: &SheprdWindow) -> bool {
        match self {
            Self::Core { epoch, key } => {
                *epoch == view.selection_epoch && view.selected_core_group() == Some(key)
            }
            Self::Legacy { epoch, generation } => {
                *epoch == view.selection_epoch
                    && view
                        .endpoints
                        .get(view.selected_endpoint)
                        .is_some_and(|endpoint| endpoint.generation == *generation)
            }
            Self::Window { epoch } => {
                *epoch == view.selection_epoch
                    && view.selected_core_group().is_none()
                    && view.endpoints.get(view.selected_endpoint).is_none()
            }
        }
    }

    pub(super) fn legacy_current(&self, view: &SheprdWindow) -> bool {
        matches!(self, Self::Legacy { .. }) && self.current(view)
    }
}

// ABOUTME: Captures Core workspace labels and status without edit or observation ownership.
// ABOUTME: Moves plain draft data while preserving accepted writes and exact target controllers.
use super::*;

pub(super) enum ActionsPaint {
    Empty,
    Changed(String),
    Ready {
        label: String,
        id: String,
        archive_label: String,
        selected: MenuAction,
        error: Option<String>,
    },
}

pub(super) struct DialogPaint {
    pub(super) title: String,
    pub(super) submit: String,
    pub(super) note: Option<String>,
    pub(super) named: bool,
    pub(super) armed: bool,
    pub(super) deleting: bool,
    pub(super) status: Option<String>,
    pub(super) error: Option<String>,
    pub(super) input: Option<DialogInput>,
}

pub(super) enum Content {
    Actions(ActionsPaint),
    Dialog(DialogPaint),
}

pub(crate) struct WorkspacePaint {
    pub(super) content: Content,
    pub(super) offset: Pixels,
}

impl SheprdWindow {
    pub(crate) fn core_workspace_paint(&mut self, cx: &App) -> Option<WorkspacePaint> {
        let content = match self.menu.page? {
            Page::CoreWorkspaceActions => Content::Actions(self.prepare_core_workspace_actions(cx)),
            Page::CoreWorkspaceCreate | Page::CoreWorkspaceRename | Page::CoreWorkspaceDelete => {
                let mut paint = self.prepare_core_workspace_dialog(cx);
                if paint.named {
                    paint.input = Some(self.menu.input.take().unwrap_or_default());
                }
                Content::Dialog(paint)
            }
            _ => return None,
        };
        Some(WorkspacePaint {
            content,
            offset: self.workspace_shell_offset(),
        })
    }

    pub(super) fn prepare_core_workspace_actions(&self, cx: &App) -> ActionsPaint {
        let Some(target) = self.core_terminals.workspace_ui.target else {
            return ActionsPaint::Empty;
        };
        let (label, archived) = match self.core_workspace_access(cx) {
            Access::Ready(document)
                if document.version == target.version
                    && Self::core_workspace_epoch(cx) == Some(target.epoch) =>
            {
                match document
                    .value
                    .workspaces()
                    .iter()
                    .find(|row| row.id == target.id)
                {
                    Some(row) => (row.label.clone(), row.archived),
                    None => return ActionsPaint::Changed(fl!("core-workspace-changed")),
                }
            }
            _ => return ActionsPaint::Changed(fl!("core-workspace-changed")),
        };
        let id = target.id.to_string();
        let selected = self.core_terminals.workspace_ui.selected;
        let archive_label = if archived {
            fl!("core-workspace-restore")
        } else {
            fl!("core-workspace-archive")
        };
        ActionsPaint::Ready {
            label,
            id,
            archive_label,
            selected,
            error: self.menu.error.clone(),
        }
    }

    pub(super) fn prepare_core_workspace_dialog(&self, cx: &App) -> DialogPaint {
        let status = if self.core_terminals.workspace_ui.pending.is_some() {
            Some(fl!("core-workspace-editing"))
        } else {
            match self.core_workspace_access(cx) {
                Access::Error(error) => Some(fl!(
                    "core-workspace-error",
                    reason = bounded(&error.to_string())
                )),
                Access::Missing => Some(fl!("core-workspace-stale")),
                Access::Ready(document)
                    if self
                        .core_terminals
                        .workspace_ui
                        .target
                        .is_some_and(|target| {
                            target.version != document.version
                                || Self::core_workspace_epoch(cx) != Some(target.epoch)
                                || !document
                                    .value
                                    .workspaces()
                                    .iter()
                                    .any(|row| row.id == target.id)
                        }) =>
                {
                    Some(fl!("core-workspace-changed"))
                }
                Access::Ready(document) if EditGuard::blocks(cx, document.version) => {
                    Some(fl!("core-workspace-uncertain"))
                }
                Access::Ready(_)
                    if !self
                        .core_terminals
                        .hub
                        .backend()
                        .is_some_and(|backend| !backend.core().is_lost()) =>
                {
                    Some(fl!("core-workspace-connect"))
                }
                Access::Ready(_) => None,
            }
        };
        let named = matches!(
            self.menu.page,
            Some(Page::CoreWorkspaceCreate | Page::CoreWorkspaceRename)
        );
        let armed =
            status.is_none()
                && (!named
                    || self.menu.input.as_ref().is_some_and(|input| {
                        !input.text.trim().is_empty() && input.marked.is_none()
                    }));
        let title = match self.menu.page {
            Some(Page::CoreWorkspaceRename) => fl!("core-workspace-rename-title"),
            Some(Page::CoreWorkspaceDelete) => fl!("core-workspace-delete-title"),
            _ => fl!("menu-page-new-workspace"),
        };
        let submit = match self.menu.page {
            Some(Page::CoreWorkspaceRename) => fl!("core-workspace-rename"),
            Some(Page::CoreWorkspaceDelete) => fl!("core-workspace-delete"),
            _ => fl!("workspace-dialog-create"),
        };
        let note = if !named {
            self.core_terminals.workspace_ui.target.and_then(|target| {
                let Access::Ready(document) = self.core_workspace_access(cx) else {
                    return None;
                };
                document
                    .value
                    .workspaces()
                    .iter()
                    .find(|row| row.id == target.id)
                    .map(|row| {
                        fl!(
                            "core-workspace-delete-note",
                            name = row.label.as_str(),
                            id = target.id.to_string()
                        )
                    })
            })
        } else {
            None
        };
        DialogPaint {
            title,
            submit,
            note,
            named,
            armed,
            deleting: self.menu.page == Some(Page::CoreWorkspaceDelete),
            status,
            error: self.menu.error.clone(),
            input: None,
        }
    }
}

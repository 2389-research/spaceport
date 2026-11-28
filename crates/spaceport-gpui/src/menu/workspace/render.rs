//! Painting a workspace dialog: the words its action asks for, this window's
//! field, and the handlers its two buttons run. The form itself is
//! `spaceport_menu::form`'s.

use crate::{
    SpaceportWindow,
    i18n::fl,
    menu::{WorkspaceAction, WorkspaceMenuAction, danger, workspace_action_icon},
};
use gpui::{prelude::*, *};
use spaceport_menu::form::{self, Button, Buttons, Dialog};

impl SpaceportWindow {
    pub(in crate::menu) fn render_workspace_dialog(
        &self,
        action: WorkspaceAction,
        cx: &mut Context<Self>,
    ) -> Div {
        let Some(target) = &self.menu.target else {
            return div();
        };
        let (title, note, submit): (String, String, String) = match action {
            WorkspaceAction::Rename => (
                fl!("workspace-dialog-rename-title"),
                fl!("workspace-dialog-rename-note"),
                fl!("workspace-dialog-rename"),
            ),
            WorkspaceAction::NewTab => (
                fl!("workspace-dialog-new-tab-title"),
                fl!("workspace-dialog-new-tab-note"),
                fl!("workspace-dialog-create"),
            ),
            WorkspaceAction::NewWorkspace => (
                fl!("workspace-dialog-new-workspace-title"),
                fl!("workspace-dialog-new-workspace-note"),
                fl!("workspace-dialog-create"),
            ),
            WorkspaceAction::Note => (
                fl!("workspace-dialog-note-title"),
                fl!("workspace-dialog-note-note"),
                fl!("workspace-dialog-save"),
            ),
        };
        let dialog = Dialog {
            buttons: Buttons {
                cancel: fl!("workspace-dialog-cancel"),
                submit,
            },
            error: self.menu.error.clone(),
            icon: workspace_action_icon(WorkspaceMenuAction::Dialog(action)),
            note,
            target: target.label.clone(),
            title,
        };
        let field = self
            .menu
            .input
            .is_some()
            .then(|| self.render_dialog_input(cx).into_any_element());
        form::dialog(
            &dialog,
            &crate::menu::chrome::text(&self.config.ui),
            &self.theme,
            danger(&self.theme),
            field,
            |button, element| match button {
                Button::Cancel => element.on_click(cx.listener(|this, _, window, cx| {
                    cx.stop_propagation();
                    this.dismiss_menu(window, cx);
                })),
                Button::Submit => element.on_click(cx.listener(|this, _, window, cx| {
                    cx.stop_propagation();
                    this.submit_workspace_dialog(window, cx);
                })),
            },
        )
    }
}

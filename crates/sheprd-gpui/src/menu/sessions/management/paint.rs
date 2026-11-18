// ABOUTME: Captures session form display without session targets or input entities.
// ABOUTME: Retains only destination text, draft paint, warning text, and errors.
use super::*;

pub(super) enum Kind {
    Create { destination: String },
    Delete { name: String },
}

pub(super) struct EditText {
    pub(super) kind: Kind,
    pub(super) error: Option<String>,
}

pub(in crate::menu) struct EditPaint {
    pub(super) text: EditText,
    pub(super) input: Option<crate::search_input::Paint>,
    pub(super) offset: Pixels,
}

impl SheprdWindow {
    pub(in crate::menu) fn session_edit_paint(&self, cx: &App) -> Option<EditPaint> {
        Some(EditPaint {
            text: self.session_edit_text()?,
            input: match self.menu.session_edit.as_ref()? {
                Edit::Create { input, .. } => Some(input.read(cx).paint()),
                Edit::Delete { .. } => None,
            },
            offset: self.menu.session_edit_scroll.offset().y,
        })
    }

    pub(super) fn session_edit_text(&self) -> Option<EditText> {
        let kind = match self.menu.session_edit.as_ref()? {
            Edit::Create { target, .. } => Kind::Create {
                destination: match target {
                    Target::Local => fl!("sessions-menu-this-machine"),
                    Target::Device {
                        host: RemoteHost::Ssh(target),
                        ..
                    } => target.clone(),
                    Target::Device {
                        host: RemoteHost::Wsl(distro),
                        ..
                    } => fl!("sessions-menu-wsl", distribution = distro.as_str()),
                },
            },
            Edit::Delete { row, .. } => Kind::Delete {
                name: match row {
                    Row::Local(name) | Row::Device { session: name, .. } => name.clone(),
                    Row::Add(_) => String::new(),
                },
            },
        };
        Some(EditText {
            kind,
            error: self.menu.error.clone(),
        })
    }
}

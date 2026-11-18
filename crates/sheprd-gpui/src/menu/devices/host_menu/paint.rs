// ABOUTME: Captures host popup display data without live input or worker ownership.
// ABOUTME: Keeps shell offsets and width decisions stable after the host resets.
use super::*;

pub(super) struct Header {
    pub(super) label: String,
    pub(super) target: String,
    pub(super) session: String,
}

pub(super) struct Row {
    pub(super) index: usize,
    pub(super) action: Action,
    pub(super) label: String,
    pub(super) toggle: Option<bool>,
    pub(super) selected: bool,
    pub(super) theme: Option<String>,
    pub(super) error: Option<String>,
}

pub(super) enum Content {
    // keep-sorted start
    Forward(forwards::Paint),
    Host {
        header: Header,
        rows: Vec<Row>,
        forwards: Vec<forwards::Row>,
    },
    Remove(String),
    Rename(rename::Paint),
    Tags(tags::Paint),
    // keep-sorted end
}

pub(in crate::menu) struct Paint {
    pub(super) content: Content,
    pub(super) offset: Point<Pixels>,
}

impl Paint {
    pub(in crate::menu) fn forwards(&self) -> bool {
        matches!(&self.content, Content::Host { forwards, .. } if !forwards.is_empty())
    }
}

impl SheprdWindow {
    pub(in crate::menu) fn host_shell_scroll(&self) -> Option<ScrollHandle> {
        matches!(
            self.menu.page,
            Some(Page::Host | Page::DeviceTags | Page::RemoveDevice)
        )
        .then(|| self.menu.host.as_ref().map(|host| host.scroll.clone()))
        .flatten()
    }

    pub(in crate::menu) fn host_paint(&self, cx: &App) -> Option<Paint> {
        self.prepare_host_paint(true, cx)
    }

    pub(super) fn prepare_host_paint(&self, frozen: bool, cx: &App) -> Option<Paint> {
        let host = self.menu.host.as_ref()?;
        let content = match self.menu.page? {
            Page::Host => {
                let server_keys =
                    self.config.keybinding_source(&host.id) == KeybindingSource::Server;
                let device_theme = self.device_theme_value(&host.id);
                let muted = crate::notification_mutes::Mutes::of(cx)
                    .is_some_and(|mutes| mutes.host(&host.id));
                Content::Host {
                    header: Header {
                        label: host.label.clone(),
                        target: host.target.clone(),
                        session: host.session.clone(),
                    },
                    rows: host
                        .actions()
                        .iter()
                        .enumerate()
                        .map(|(index, &(action, message))| Row {
                            index,
                            action,
                            label: crate::i18n::message(message, &[]),
                            toggle: match action {
                                Action::Mute => Some(muted),
                                Action::ServerKeybindings => Some(server_keys),
                                _ => None,
                            },
                            selected: host.selected == Some(index),
                            theme: device_theme
                                .filter(|_| action == Action::Theme)
                                .map(str::to_owned),
                            error: (action == Action::ServerKeybindings)
                                .then(|| self.server_keybindings_error(&host.id))
                                .flatten()
                                .map(|error| {
                                    fl!("host-menu-local-keybindings", error = error.to_string())
                                }),
                        })
                        .collect(),
                    forwards: self.host_forward_rows(host),
                }
            }
            Page::RenameDevice => Content::Rename(rename::Paint::new(host, cx)),
            Page::ForwardPort => Content::Forward(forwards::Paint::new(host, cx)),
            Page::DeviceTags => Content::Tags(self.device_tags_paint(host, frozen, cx)?),
            Page::RemoveDevice => Content::Remove(removal_note(host)),
            _ => return None,
        };
        Some(Paint {
            content,
            offset: self
                .host_shell_scroll()
                .map_or(Point::default(), |scroll| scroll.offset()),
        })
    }
}

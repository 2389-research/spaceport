// ABOUTME: Authors native chat accessibility data through the shared transcript model.
// ABOUTME: Observes queued answer intent without pretending a preview has a transport.
use super::PANE;
use crate::SheprdWindow;
use anyhow::{Context as _, Result, ensure};
use sheprd_chat_protocol::{Change, ClientMessage, Command, EntryKind, ServerMessage, Transcript};

#[derive(serde::Deserialize)]
pub(super) struct Fixture {
    pub(super) draft: String,
    pub(super) transcript: Transcript,
}

pub(super) fn fixture() -> Result<Fixture> {
    serde_json::from_str(include_str!(
        "../../../tests/support/native-chat-accessibility.json"
    ))
    .context("decode native chat accessibility fixture")
}

pub(super) struct Probe {
    fixture: Fixture,
    answered: usize,
    menu_open: bool,
    tool_open: bool,
}

impl Probe {
    pub(super) fn new() -> Result<Self> {
        Ok(Self {
            fixture: fixture()?,
            answered: 0,
            menu_open: false,
            tool_open: false,
        })
    }

    pub(super) fn complete(&self) -> bool {
        self.answered == self.fixture.transcript.permissions.len()
    }

    pub(super) fn observe(&mut self, view: &mut SheprdWindow, cx: &gpui::App) -> Result<()> {
        let pane = view
            .chats
            .pane_mut(PANE)
            .context("native chat pane missing")?;
        let input = pane
            .composer
            .as_ref()
            .context("native chat composer missing")?;
        ensure!(
            input.read(cx).text() == self.fixture.draft,
            "native chat draft changed"
        );
        let menu_open = pane.menu().is_some();
        if self.menu_open != menu_open {
            self.menu_open = menu_open;
            eprintln!(
                "CHAT_ACCESSIBILITY menu={}", // i18n: data
                if menu_open { "open" } else { "closed" }
            );
        }
        let tool = self
            .fixture
            .transcript
            .entries
            .iter()
            .find(|entry| matches!(entry.kind, EntryKind::Tool(_)))
            .context("native chat tool missing")?;
        let tool_open = pane.expanded.contains(&tool.id);
        if self.tool_open != tool_open {
            self.tool_open = tool_open;
            eprintln!(
                "CHAT_ACCESSIBILITY tool={}", // i18n: data
                if tool_open { "expanded" } else { "collapsed" }
            );
        }
        // This inspects the real queued commands once. No linked agent receives them.
        for message in pane.session.resend() {
            let ClientMessage::Command { command, id } = message else {
                anyhow::bail!("native chat produced an unexpected message");
            };
            let expected = self
                .fixture
                .transcript
                .permissions
                .get(self.answered)
                .context("native chat queued an extra answer")?;
            let option = expected
                .options
                .first()
                .context("native chat option missing")?;
            ensure!(
                command
                    == Command::Answer {
                        permission: expected.id,
                        option: option.id.clone()
                    },
                "native chat queued an unexpected command"
            );
            pane.session
                .receive(ServerMessage::Ack { id, rejected: None })?;
            let mut cursor = pane.session.since().context("native chat epoch missing")?;
            cursor.seq = cursor
                .seq
                .checked_add(1)
                .context("native chat cursor overflow")?;
            pane.session.receive(ServerMessage::Change {
                change: Change::PermissionResolved { id: expected.id },
                cursor,
            })?;
            self.answered += 1;
            eprintln!(
                "CHAT_ACCESSIBILITY answer={} queued=true draft=preserved", // i18n: data
                self.answered
            );
        }
        Ok(())
    }
}

//! The `[chat]` table: what chats started from the app run with.
//!
//! ```toml
//! [chat]
//! glide = true
//! launcher = "plus"
//! layout = "cards"
//! max_width = 800 # logical pixels; 0 uses the full available width
//!
//! [chat.env]
//! GH_HOST = "mycompany.ghe.com"
//! ANTHROPIC_BASE_URL = "https://llm.example.com"
//! ```
//!
//! `env` goes into the environment of the tab a chat starts in, on this
//! machine or an SSH host, so an agent reaches a company endpoint that the
//! pane's shell does not set up. Values sit in the config file as written,
//! so a token belongs in the shell or the agent's own sign-in instead.
//! `launcher` picks where tab strips offer a one-click chat with each
//! installed agent. `layout` picks how a chat view draws its entries and composer. `glide`
//! scrolls smoothly to a streaming reply's new text; off, the default, the
//! view jumps to it as each chunk lands.

mod launcher;

use serde::Deserialize;
use std::collections::BTreeMap;

pub use launcher::ChatLauncher;

/// The centered reading column, unless the user chooses full width with zero.
pub const DEFAULT_MAX_WIDTH: u16 = 800;
pub const MAX_WIDTH_RANGE: std::ops::RangeInclusive<u16> = 320..=4_000;
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ChatConfig {
    pub env: BTreeMap<String, String>,
    pub glide: bool,
    pub launcher: ChatLauncher,
    pub layout: ChatLayoutMode,
    /// Maximum reading-column width in logical pixels; zero uses all available space.
    #[serde(deserialize_with = "deserialize_max_width")]
    pub max_width: u16,
    /// Explicit transcript sidecars. Never starts or replaces the pane's agent.
    pub mirrors: Vec<TranscriptMirror>,
    /// Show running Codex/Claude conversations over their existing terminal.
    /// Discovery is opt-in and never starts or resumes an agent.
    pub render_existing: bool,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            env: BTreeMap::new(),
            glide: false,
            launcher: ChatLauncher::default(),
            layout: ChatLayoutMode::default(),
            max_width: DEFAULT_MAX_WIDTH,
            mirrors: Vec::new(),
            render_existing: false,
        }
    }
}

fn deserialize_max_width<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> crate::Result<u16, D::Error> {
    let width = u16::deserialize(deserializer)?;
    check_max_width(width).map_err(serde::de::Error::custom)?;
    Ok(width)
}

pub fn check_max_width(width: u16) -> crate::Result<()> {
    if width == 0 || MAX_WIDTH_RANGE.contains(&width) {
        Ok(())
    } else {
        Err(super::Error::InvalidChatWidth)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TranscriptMirror {
    pub pane: String,
    pub endpoint: String,
    /// Omit for a local daemon; otherwise the exact saved SSH target.
    pub host: Option<String>,
}

/// Declares each layout once: its variant, the config value that selects
/// it, and how the app titles it, in the order switchers list them.
macro_rules! chat_layouts {
    ($($(#[$doc:meta])* $variant:ident => $name:literal, $label:literal;)+) => {
        /// How a chat view draws its entries and composer. Each one is a
        /// `chat::view::layout::ChatLayout`; what a chat does is the same in
        /// all.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub enum ChatLayoutMode {
            $($(#[$doc])* $variant,)+
        }

        impl ChatLayoutMode {
            /// Every layout, in the order switchers list them.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            /// `ALL`'s names, for errors that list what a config may say.
            const NAMES: &'static [&'static str] = &[$($name,)+];

            /// The config value that selects it.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }

            /// The message naming it.
            pub const fn message(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }
        }
    };
}

chat_layouts! {
    /// Entries on a line down the left, an avatar where the speaker
    /// changes, and the settings inside the composer's card.
    #[default]
    Timeline => "timeline", "chat-layout-timeline";
    /// The user's messages in bubbles on the right and the agent's as
    /// plain text, with the settings under the composer, like Claude's app.
    Bubbles => "bubbles", "chat-layout-bubbles";
    /// The terminal's font throughout, prompts marked `›`, the agent and
    /// its tools `⏺`, and a status line under the prompt, like Claude
    /// Code's own screen.
    Terminal => "terminal", "chat-layout-terminal";
    /// Each prompt as a heading with an accent bar, the reply as prose
    /// under it, and a quiet rule closing each turn.
    Document => "document", "chat-layout-document";
    /// Each turn in one card: the prompt on top, the reply under a
    /// hairline, how it ended at its foot.
    Cards => "cards", "chat-layout-cards";
    /// Blue bubbles on the right for the user, gray ones on the left for
    /// the agent, each run ending in a tail, like Apple's Messages.
    IMessage => "imessage", "chat-layout-imessage";
    /// One column at a book's measure: prompts and steps as quiet notes
    /// hung from a gutter mark, the agent's prose in full ink, and earlier
    /// turns muted until pointed at, like iA Writer.
    IA => "ia", "chat-layout-ia";
    /// The terminal's font, prompts and open tool calls as tinted blocks
    /// behind a bar, finished calls folded to one line, and a status row
    /// under the composer, like OpenCode's own screen.
    OpenCode => "opencode", "chat-layout-opencode";
}

impl ChatLayoutMode {
    /// The layout after this one, for a switcher that steps through them.
    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|&mode| mode == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

impl<'de> Deserialize<'de> for ChatLayoutMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> crate::Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        // `pill` was offered by builds before it was retired; a config that
        // saved it opens in its closest layout rather than failing to load.
        if name == "pill" {
            return Ok(Self::Bubbles);
        }
        Self::ALL
            .iter()
            .copied()
            .find(|mode| mode.name() == name)
            .ok_or_else(|| serde::de::Error::unknown_variant(&name, Self::NAMES))
    }
}

impl ChatConfig {
    /// Refuses mirrors a chat view cannot follow. The environment is checked
    /// where a chat is started, against the names the app sets itself.
    pub fn check(&self) -> crate::Result<()> {
        if self.mirrors.len() > 16
            || self.mirrors.iter().any(|mirror| {
                mirror.pane.is_empty()
                    || mirror.pane.len() > 256
                    || mirror
                        .endpoint
                        .parse::<spaceport_chat_protocol::EndpointId>()
                        .is_err()
                    || mirror
                        .host
                        .as_ref()
                        .is_some_and(|host| host.is_empty() || host.len() > 1024)
            })
        {
            return Err(crate::Error::InvalidChatMirror);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

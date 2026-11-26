// ABOUTME: Localizes chat connection, command, and helper failures.
// ABOUTME: Keeps local delivery uncertainty distinct from wire refusals.
//! Agent chats and starting one: `error-chat.ftl`. The attach side's own
//! failures, the program's refusals, the card catalog's and the environment
//! table's word themselves.

use super::Localize;
use crate::{
    chat::{Error, Program, StartError, page, pasted, visual},
    i18n::fl,
};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Attach(error) => error.localize(),
            Self::Environment(error) => error.localize(),
            Self::Pasted(error) => error.localize(),
            Self::Start(error) => error.localize(),
            // keep-sorted end
        }
    }
}

impl Localize for StartError {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::AgentMissing(agent) => {
                fl!(
                    "error-chat-start-agent-missing",
                    agent = agent.display_name()
                )
            }
            Self::CoreTerminalRequired => fl!("error-chat-start-core-terminal-required"),
            Self::Disabled => fl!("error-chat-start-disabled"),
            Self::Environment(error) => error.localize(),
            Self::NotConnected => fl!(
                "error-chat-start-not-connected",
                app = crate::constants::WINDOW_TITLE
            ),
            Self::NotLocal => fl!("error-chat-start-not-local"),
            Self::NoWorkspace => fl!("error-chat-start-no-workspace"),
            Self::ProgramMissing => fl!(
                "error-chat-start-program-missing",
                app = crate::constants::WINDOW_TITLE,
                program = Program::file_name()
            ),
            Self::ProgramUnknown => fl!(
                "error-chat-start-program-unknown",
                app = crate::constants::WINDOW_TITLE,
                program = Program::file_name()
            ),
            Self::Request(error) => error.localize(),
            Self::Response => fl!("error-chat-start-response"),
            Self::SlotBusy => fl!("error-chat-start-slot-busy"),
            Self::Start(error) => error.localize(),
            Self::Timeout => fl!("error-chat-start-timeout"),
            // keep-sorted end
        }
    }
}

impl Localize for pasted::Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Folder => fl!("error-chat-pasted-folder"),
            Self::Write(_) => fl!("error-chat-pasted-write"),
            // keep-sorted end
        }
    }
}

impl Localize for page::Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Folder => fl!("error-chat-page-folder"),
            // The system's own words.
            Self::Io(error) => error.to_string(),
            Self::NotFile => fl!("error-chat-page-not-file"),
            Self::TooLarge => fl!("error-chat-page-too-large"),
            // keep-sorted end
        }
    }
}

impl Localize for visual::Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Base64(_) => fl!("error-chat-visual-base64"),
            Self::Decode(_) => fl!("error-chat-visual-decode"),
            Self::Diagram(_) => fl!("error-chat-visual-diagram"),
            Self::Fetch(_) => fl!("error-chat-visual-fetch"),
            // The formula renderer's own words.
            Self::Math(reason) => fl!("error-chat-visual-math", reason = reason.as_str()),
            Self::MathLimit => fl!("error-chat-visual-math-limit"),
            Self::MathParse(_) => fl!("error-chat-visual-math-parse"),
            Self::NotAnImage => fl!("error-chat-visual-not-an-image"),
            Self::Panicked => fl!("error-chat-visual-panicked"),
            Self::Raster(_) => fl!("error-chat-visual-raster"),
            // keep-sorted end
        }
    }
}

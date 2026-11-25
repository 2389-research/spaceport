//! A refused chat command, from `error-chat.ftl`: the program's own name for
//! why it would not take a prompt, a permission answer, or a setting.
// i18n: migrated

use crate::{MAX_ATTACHMENTS, Rejection};
use spaceport_i18n::{Localize, fl};

impl Localize for Rejection {
    fn localize(&self) -> String {
        match self {
            Self::EmptyPrompt => fl!("error-chat-rejection-empty-prompt"),
            Self::PromptTooLong => fl!("error-chat-rejection-prompt-too-long"),
            Self::NotReady => fl!("error-chat-rejection-not-ready"),
            Self::NotWorking => fl!("error-chat-rejection-not-working"),
            Self::UnknownPermission => fl!("error-chat-rejection-unknown-permission"),
            Self::UnknownOption => fl!("error-chat-rejection-unknown-option"),
            Self::UnknownMode => fl!("error-chat-rejection-unknown-mode"),
            Self::UnknownSetting => fl!("error-chat-rejection-unknown-setting"),
            Self::UnknownQueued => fl!("error-chat-rejection-unknown-queued"),
            Self::AgentUnavailable => fl!("error-chat-rejection-agent-unavailable"),
            Self::TooManyAttachments => fl!(
                "error-chat-rejection-too-many-attachments",
                count = MAX_ATTACHMENTS
            ),
            Self::QueueFull => fl!("error-chat-rejection-queue-full"),
            Self::UnknownFile => fl!("error-chat-rejection-unknown-file"),
            Self::Unsupported => fl!("error-chat-rejection-unsupported"),
            Self::Other => fl!("error-chat-rejection-other"),
        }
    }
}

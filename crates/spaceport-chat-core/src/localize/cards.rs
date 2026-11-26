//! The interactive cards an agent's JSON describes, from `error-chat.ftl`.

use crate::widgets;
use spaceport_i18n::{Localize, fl};

impl Localize for widgets::Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::ActionGone => fl!("error-chat-card-action-gone"),
            Self::Component => fl!("error-chat-card-component"),
            Self::ComponentCount => fl!("error-chat-card-component-count"),
            Self::FieldId => fl!("error-chat-card-field-id"),
            Self::FormValue => fl!("error-chat-card-form-value"),
            Self::Json(_) => fl!("error-chat-card-json"),
            Self::Title => fl!("error-chat-card-title"),
            Self::TooLarge => fl!("error-chat-card-too-large"),
            Self::TooManyFields => fl!("error-chat-card-too-many-fields"),
            // keep-sorted end
        }
    }
}

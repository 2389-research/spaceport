//! A refused delivery, from `error-chat.ftl`: the local uncertainty of a
//! prompt this window handed over, apart from the program's own refusal.

use crate::session::Refusal;
use spaceport_i18n::{Localize, fl};

impl Localize for Refusal {
    fn localize(&self) -> String {
        match self {
            Self::DeliveryUnconfirmed => fl!("error-chat-delivery-unconfirmed"),
            Self::Remote(rejection) => rejection.localize(),
            Self::SessionChanged => fl!("error-chat-session-changed"),
        }
    }
}

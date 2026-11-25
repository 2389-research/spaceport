#![allow(clippy::unwrap_used)]

use super::WeakInputHandler;
use crate::search_input::SearchInput;
use gpui::{
    AppContext, Bounds, ClipboardItem, InputHandler, TestAppContext, TextInputConfiguration, point,
    px, size,
};

mod lifecycle;

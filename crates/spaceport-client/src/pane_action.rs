// ABOUTME: Maps addressed pane edits to the request accepted by Herdr.
// ABOUTME: Desktop and mobile capture the desired zoom mode when offering the action.
use crate::Method;
use serde_json::{Value, json};

pub enum Action<'a> {
    SplitRight,
    SplitDown,
    SetZoom(bool),
    Rename(&'a str),
}

impl Action<'_> {
    pub fn request(&self, pane: &str) -> (Method, Value) {
        match self {
            Self::SplitRight | Self::SplitDown => (
                Method::PaneSplit,
                json!({"target_pane_id":pane, "direction":if matches!(self, Self::SplitRight) { "right" } else { "down" }, "focus":true}),
            ),
            Self::SetZoom(enabled) => (
                Method::PaneZoom,
                json!({"pane_id":pane, "mode":if *enabled { "on" } else { "off" }}),
            ),
            Self::Rename(label) => (
                Method::PaneRename,
                json!({"pane_id":pane, "label":label.trim()}),
            ),
        }
    }
}

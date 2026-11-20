// ABOUTME: Defines semantic chat entries, plan steps, and named keyboard controls.
// ABOUTME: Reuses prepared transcript text without retaining another transcript model.
use crate::{chat::selection::Document, config::Theme, i18n::fl, notifications::safe_text};
use gpui::{
    Div, Role, SharedString, Stateful, StatefulInteractiveElement, accesskit::Node, prelude::*,
};
use sheprd_chat_protocol::{EntryId, PlanEntry, PlanStatus};

pub(super) fn button<E: StatefulInteractiveElement>(
    theme: &Theme,
    label: impl Into<SharedString>,
    control: E,
) -> E {
    crate::accessibility::focusable(
        control.role(Role::Button).aria_label(label),
        theme.ink(theme.primary()),
    )
}

pub(super) fn entry(id: EntryId, name: String, document: &Document, content: Div) -> Stateful<Div> {
    content
        .id(("chat-accessible-entry", id.0))
        .role(Role::Article)
        .aria_label(name)
        .when_some(document.entry(id), |content, projection| {
            content.a11y_synthetic_children(move |builder| {
                let bounds = builder.parent_node().bounds();
                for (index, text) in projection.texts().enumerate() {
                    let mut node = text_node(text);
                    if let Some(bounds) = bounds {
                        node.set_bounds(bounds);
                    }
                    builder.push_child(builder.synthetic_node_id(index), node);
                }
            })
        })
}

fn text_node(text: &str) -> Node {
    let mut node = Node::new(Role::Label);
    node.set_value(text);
    node
}

pub(super) fn plan_step(index: usize, step: &PlanEntry, content: Div) -> Stateful<Div> {
    let status = match step.status {
        PlanStatus::Completed => fl!("chat-tool-state-done"),
        PlanStatus::InProgress => fl!("chat-tool-state-running"),
        PlanStatus::Pending => fl!("chat-tool-state-pending"),
    };
    content
        .id(("chat-accessible-plan-step", index))
        .role(Role::ListItem)
        .aria_label(format!("{status}: {}", safe_text(&step.content, 300)))
}

#[cfg(test)]
mod tests;

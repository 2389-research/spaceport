// ABOUTME: Checks metadata on the semantic producers used by actual chat views.
// ABOUTME: Keeps assistive-technology qualification separate from headless metadata checks.
#![allow(clippy::unwrap_used)]
use super::{button, entry, plan_step, text_node};
use crate::{
    chat::{prepared::Prepared, selection::Document},
    config::Theme,
};
use gpui::{Element, InteractiveElement, IntoElement, Role, accesskit, div};
use sheprd_chat_protocol::{Entry, EntryId, EntryKind, PlanEntry, PlanStatus, Transcript};
use std::{collections::HashSet, rc::Rc};

#[test]
fn readable_nodes_share_the_visible_prepared_projection_and_follow_disclosure() {
    let id = EntryId(9);
    let source = format!("Visible first line {} hidden tail", "x".repeat(600));
    let mut transcript = Transcript::new("Fixture agent");
    transcript.entries = vec![Entry {
        id,
        kind: EntryKind::Thought {
            text: source.clone(),
        },
    }];
    let mut prepared = Prepared::default();
    prepared.update(&transcript, &[(id, 1)], |_, _| false);
    let folded = prepared.text().entry(id).unwrap();
    assert!(Rc::ptr_eq(&folded, &prepared.text().entry(id).unwrap()));
    fn read<'a>(texts: impl Iterator<Item = &'a str>) -> String {
        texts
            .map(text_node)
            .map(|node| node.value().unwrap().to_owned())
            .collect()
    }
    assert!(read(folded.texts()).contains("Visible first line"));
    assert!(!read(folded.texts()).contains("hidden tail"));
    prepared.set_expanded(&HashSet::from([id]));
    prepared.update(&transcript, &[(id, 1)], |_, _| false);
    let expanded = prepared.text().entry(id).unwrap();
    assert!(!Rc::ptr_eq(&folded, &expanded));
    assert_eq!(read(expanded.texts()), source);
}

#[test]
fn controls_expose_named_button_nodes() {
    let element = button(&Theme::default(), "Allow once", div().id("answer")).into_element();
    assert_eq!(element.a11y_role(), Some(Role::Button));
    let mut node = accesskit::Node::new(Role::Button);
    element.write_a11y_info(&mut node);
    assert_eq!(node.label(), Some("Allow once"));
}

#[test]
fn entries_expose_named_article_nodes() {
    let element = entry(
        EntryId(7),
        "Fixture agent".into(),
        &Document::default(),
        div(),
    );
    assert_eq!(element.a11y_role(), Some(Role::Article));
    let mut node = accesskit::Node::new(Role::Article);
    element.write_a11y_info(&mut node);
    assert_eq!(node.label(), Some("Fixture agent"));
}

#[test]
fn plan_steps_name_their_text_and_state() {
    let step = PlanEntry {
        content: "Run fixture checks".into(),
        priority: Default::default(),
        status: PlanStatus::InProgress,
    };
    let element = plan_step(0, &step, div());
    assert_eq!(element.a11y_role(), Some(Role::ListItem));
    let mut node = accesskit::Node::new(Role::ListItem);
    element.write_a11y_info(&mut node);
    assert!(
        node.label()
            .is_some_and(|text| text.contains(&step.content))
    );
    assert!(node.label().is_some_and(|text| text.contains("running")));
}

use super::*;
use std::rc::Rc;

mod retention;

const FORM: &str = r#"{
  "title":"Choose a run",
  "components":[
    {"type":"choice","id":"model","label":"Model","options":["small","large"]},
    {"type":"toggle","id":"cache","label":"Use cache","value":true},
    {"type":"slider","id":"count","label":"Samples","min":0,"max":10,"step":0.5,"value":2},
    {"type":"input","id":"note","label":"Note","value":"initial"},
    {"type":"action","label":"Prepare","prompt":"Use these settings"}
  ]
}"#;

#[test]
fn directional_controls_are_rejected_after_json_decoding() {
    for escaped in [r"\u202e", r"\u2066", r"\u2067", r"\u2068", r"\u2069"] {
        let source = format!(
            r#"{{"components":[{{"type":"action","label":"Review {escaped}hidden","prompt":"Continue"}}]}}"#
        );
        assert!(matches!(Document::parse(&source), Err(Error::Component)));
    }
    assert!(
        Document::parse(r#"{"components":[{"type":"text","text":"first\n\tsecond"}]}"#).is_ok()
    );
    let document = Document::parse(FORM).unwrap();
    let mut values = document.defaults();
    assert!(!document.set(
        &mut values,
        "note",
        Value::Text("safe\u{202e}hidden".into())
    ));
    assert_eq!(
        bounded_input("left\u{202e}middle\u{2066}right\u{2069}"),
        "leftmiddleright"
    );
}

#[test]
fn forms_keep_edits_across_renders_but_reset_on_source_change_and_prune() {
    use spaceport_chat_protocol::EntryId;
    let first = Rc::new(Document::parse(FORM).unwrap());
    let identical = Rc::new(Document::parse(FORM).unwrap());
    let key = (EntryId(4), 2);
    let mut state = State::default();
    assert!(state.form(key, &first).is_none());
    let form = state.edit(key, &first);
    first.set(&mut form.values, "note", Value::Text("edited".into()));
    assert_eq!(
        state.form(key, &identical).unwrap().values["note"],
        Value::Text("edited".into())
    );
    let changed = Rc::new(Document::parse(&FORM.replace("initial", "revised")).unwrap());
    assert!(state.form(key, &changed).is_none());
    assert_eq!(
        state.edit(key, &changed).values["note"],
        Value::Text("revised".into())
    );
    state.retain(&crate::chat::prepared::Prepared::default());
    assert!(state.form(key, &changed).is_none());
}

#[test]
fn slider_and_text_edits_stay_bounded_with_unicode() {
    assert_eq!(slider_value(-1., 0., 10., 0.5), 0.);
    assert_eq!(slider_value(2., 0., 10., 0.5), 10.);
    assert_eq!(slider_value(0.33, 0., 10., 0.5), 3.5);
    assert_eq!(slider_value(1., 0., 10., 3.), 10.);
    let input = bounded_input(&format!("\u{1b}\n{}", "λ".repeat(MAX_INPUT)));
    assert_eq!(input.len(), MAX_INPUT);
    assert!(input.chars().all(|ch| ch == 'λ'));
}

#[test]
fn static_cards_metrics_tables_and_charts_are_validated() {
    assert!(Document::parse(include_str!("example.json")).is_ok());
    let source = r#"{"components":[
      {"type":"card","title":"Overview","text":"A summary"},
      {"type":"metric","label":"Accuracy","value":"98%","detail":"Held-out sample"},
      {"type":"table","columns":["Model","Score"],"rows":[["A","0.98"]]},
      {"type":"chart","label":"Scores","points":[{"label":"A","value":0.98}]}
    ]}"#;
    assert!(Document::parse(source).is_ok());
}

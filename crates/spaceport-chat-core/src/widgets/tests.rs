use super::*;

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
fn actions_include_validated_current_values_as_literal_json() {
    let document = Document::parse(FORM).unwrap();
    let mut values = document.defaults();
    assert_eq!(values["model"], Value::Text("small".into()));
    assert!(document.set(&mut values, "model", Value::Text("large".into())));
    assert!(document.set(
        &mut values,
        "note",
        Value::Text("literal \"quotes\" and {{model}}".into())
    ));
    let prompt = document.prompt("Use these settings", &values).unwrap();
    let (_, json) = prompt.split_once("Selected values:\n").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["model"], "large");
    assert_eq!(parsed["note"], "literal \"quotes\" and {{model}}");
    assert_eq!(parsed["cache"], true);
    assert_eq!(parsed["count"], 2.);
    assert!(matches!(
        document.prompt("run something else", &values),
        Err(Error::ActionGone)
    ));
}

#[test]
fn unknown_catalog_fields_and_executable_components_are_rejected() {
    for source in [
        r#"{"components":[{"type":"script","text":"alert(1)"}]}"#,
        r#"{"components":[{"type":"text","text":"ok","url":"https://example.org"}]}"#,
        r#"{"components":[{"type":"action","label":"Run","prompt":"ok","mode":"shell"}]}"#,
        r#"{"components":[],"on_load":"send"}"#,
    ] {
        assert!(matches!(Document::parse(source), Err(Error::Json(_))));
    }
}

#[test]
fn malformed_streaming_and_oversized_sources_never_become_cards() {
    assert!(matches!(
        Document::parse("{\"components\":["),
        Err(Error::Json(_))
    ));
    assert!(matches!(
        Document::parse(&" ".repeat(MAX_SOURCE + 1)),
        Err(Error::TooLarge)
    ));
    assert!(matches!(
        Document::parse(r#"{"components":[]}"#),
        Err(Error::ComponentCount)
    ));
    assert!(matches!(
        Document::parse(&format!(
            r#"{{"title":"{}","components":[{{"type":"text","text":"ok"}}]}}"#,
            "x".repeat(MAX_LABEL + 1)
        )),
        Err(Error::Title)
    ));
    assert!(matches!(
        Document::parse(
            r#"{"components":[{"type":"toggle","id":"x","label":"A"},{"type":"input","id":"x","label":"B"}]}"#
        ),
        Err(Error::FieldId)
    ));
    for source in [
        r#"{"components":[{"type":"text","text":"bad\u001b[2J"}]}"#,
        r#"{"components":[{"type":"slider","id":"x","label":"X","min":2,"max":1,"value":1}]}"#,
        r#"{"components":[{"type":"slider","id":"x","label":"X","min":0,"max":1,"step":0,"value":0}]}"#,
        r#"{"components":[{"type":"choice","id":"x","label":"X","options":["a"],"value":"b"}]}"#,
        r#"{"components":[{"type":"table","columns":["A","B"],"rows":[["one"]]}]}"#,
        r#"{"components":[{"type":"chart","label":"X","points":[{"label":"A","value":-1}]}]}"#,
    ] {
        assert!(
            matches!(Document::parse(source), Err(Error::Component)),
            "{source}"
        );
    }
}

#[test]
fn field_values_cannot_escape_the_catalog_or_its_bounds() {
    let document = Document::parse(FORM).unwrap();
    let mut values = document.defaults();
    let unchanged = values.clone();
    for (id, value) in [
        ("model", Value::Text("not offered".into())),
        ("count", Value::Number(f64::NAN)),
        ("count", Value::Number(11.)),
        ("cache", Value::Text("true".into())),
        ("note", Value::Text("x".repeat(MAX_INPUT + 1))),
        ("command", Value::Text("unexpected".into())),
    ] {
        assert!(!document.set(&mut values, id, value));
    }
    assert_eq!(values, unchanged);
    values.insert("command".into(), Value::Text("unexpected".into()));
    assert!(matches!(
        document.prompt("Use these settings", &values),
        Err(Error::FormValue)
    ));
}

#[test]
fn slider_values_land_on_clean_step_decimals() {
    assert_eq!(slider_value(0.35, 0., 1., 0.05), 0.35);
    assert_eq!(slider_value(0.35, 0., 1., 0.05).to_string(), "0.35");
    let mut value = 0.6;
    for _ in 0..7 {
        value = snap(value + 0.05, 0., 1., 0.05);
    }
    assert_eq!(value, 0.95);
    assert_eq!(snap(0.1 + 0.2, 0., 1., 0.1).to_string(), "0.3");
    assert_eq!(snap(2.4, 0.5, 10., 1.), 2.5);
    assert_eq!(snap(f64::MAX / 2., 0., f64::MAX, 0.5), f64::MAX / 2.);
}

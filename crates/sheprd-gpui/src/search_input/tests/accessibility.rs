// ABOUTME: Checks assistive search replacement against the field's actual editing state.
// ABOUTME: Preserves single-line normalization and refuses stale actions on masked fields.
use super::*;
use gpui::accesskit::ActionData;

#[gpui::test]
fn assistive_value_replaces_the_whole_unicode_query_and_clears_composition(
    cx: &mut gpui::TestAppContext,
) {
    let input = cx.new(SearchInput::new);
    input.update(cx, |input, cx| {
        input.set_text_at("old 😀 query", 4, cx);
        input.edit.marked = Some(0..3);
        input.set_accessible_value(Some(&ActionData::Value("ł\n😀\rfont".into())), cx);
        assert_eq!(input.text(), "ł😀font");
        assert_eq!(
            input.edit.selection(),
            input.text().len()..input.text().len()
        );
        assert!(input.edit.marked.is_none());
        input.set_accessible_value(Some(&ActionData::Value(String::new().into())), cx);
        assert!(input.text().is_empty());
        assert_eq!(input.edit.selection(), 0..0);
    });
}

#[gpui::test]
fn masked_fields_and_non_value_actions_do_not_change_the_query(cx: &mut gpui::TestAppContext) {
    let input = cx.new(SearchInput::new);
    input.update(cx, |input, cx| {
        input.set_text_at("keep", 2, cx);
        input.set_accessible_value(None, cx);
        input.set_accessible_value(Some(&ActionData::NumericValue(1.0)), cx);
        assert_eq!(input.text(), "keep");
        assert_eq!(input.edit.selection(), 2..2);
        input.set_masked(true, cx);
        input.set_accessible_value(Some(&ActionData::Value("replace".into())), cx);
        assert_eq!(input.text(), "keep");
        assert_eq!(input.edit.selection(), 2..2);
    });
}

#[gpui::test]
fn assistive_replacement_notifies_the_search_owner_only_when_text_changes(
    cx: &mut gpui::TestAppContext,
) {
    let input = cx.new(SearchInput::new);
    let changed = std::rc::Rc::new(std::cell::Cell::new(0));
    let _subscription = cx.update(|cx| {
        let changed = changed.clone();
        cx.subscribe(&input, move |_, _: &Changed, _| {
            changed.set(changed.get() + 1)
        })
    });
    for text in ["font", "font", ""] {
        input.update(cx, |input, cx| {
            input.set_accessible_value(Some(&ActionData::Value(text.into())), cx);
        });
        cx.run_until_parked();
        assert_eq!(changed.get(), if text.is_empty() { 2 } else { 1 });
    }
}

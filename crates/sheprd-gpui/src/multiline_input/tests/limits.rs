// ABOUTME: Checks composer byte limits through native editing and restored drafts.
// ABOUTME: Preserves selection and marked text when a replacement exceeds the prompt contract.
use super::*;
use gpui::EntityInputHandler;
use sheprd_chat_protocol::MAX_PROMPT_SIZE;

#[gpui::test]
fn oversized_native_edits_preserve_the_draft_and_its_selection(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.set_text_at("keep this", 4, cx);
            input.edit.select(0..4);
            input.replace_text_in_range(None, &"é".repeat(MAX_PROMPT_SIZE / 2), window, cx);
            assert!(
                input.text() == "keep this",
                "oversized insertion changed the draft"
            );
            assert!(input.error.is_some());
            assert_eq!(input.edit.selection(), 0..4);
            assert!(input.edit.marked.is_none());
        })
    });
}

#[gpui::test]
fn exact_limit_replacement_counts_normalized_newlines_and_utf8_bytes(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.set_text_at("x", 1, cx);
            input.edit.select(0..1);
            let text = "é".repeat(MAX_PROMPT_SIZE / 2 - 1) + "\r\n!";
            input.replace_text_in_range(None, &text, window, cx);
            assert_eq!(input.text().len(), MAX_PROMPT_SIZE);
            assert!(input.text().ends_with("\n!"));
            input.replace_text_in_range(None, "x", window, cx);
            assert_eq!(input.text().len(), MAX_PROMPT_SIZE);
            input.clear(cx);
        })
    });
}

#[gpui::test]
fn oversized_ime_and_restored_drafts_do_not_replace_existing_text(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.set_text_at("draft", 5, cx);
            input.replace_and_mark_text_in_range(None, "é", Some(0..1), window, cx);
            let text = input.text().to_owned();
            let marked = input.edit.marked.clone();
            let selection = input.edit.selection();
            input.replace_and_mark_text_in_range(
                None,
                &"x".repeat(MAX_PROMPT_SIZE + 1),
                None,
                window,
                cx,
            );
            assert!(
                input.text() == text,
                "oversized IME input changed the draft"
            );
            assert_eq!(input.edit.marked, marked);
            assert_eq!(input.edit.selection(), selection);
            input.set_text_at(&"x".repeat(MAX_PROMPT_SIZE + 1), 0, cx);
            assert!(
                input.text() == text,
                "oversized restoration changed the draft"
            );
        })
    });
}

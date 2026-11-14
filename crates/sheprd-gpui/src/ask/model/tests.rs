// ABOUTME: Checks that resetting a running conversation requires an explicit choice.
// ABOUTME: Guards idle opening and single-turn composer behavior.
use super::*;

#[test]
fn new_chat_preserves_the_running_conversation_until_confirmed() {
    let mut interaction = Interaction::default();
    assert!(!interaction.new_chat(true));
    assert!(interaction.confirm_new);
}

#[test]
fn idle_new_chat_does_not_need_confirmation() {
    let mut interaction = Interaction::default();
    assert!(interaction.new_chat(false));
    assert!(!interaction.confirm_new);
}

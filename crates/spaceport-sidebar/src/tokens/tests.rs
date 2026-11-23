use super::*;

fn unstyled(kind: TokenKind) -> ResolvedToken {
    ResolvedToken {
        kind,
        style: TokenStyle::default(),
    }
}

#[test]
fn budgets_drop_leftmost_text_first_then_share_the_rest() {
    let fixed = |kind: &TokenKind| match kind {
        TokenKind::StateIcon => 1,
        _ => 0,
    };
    let row = [
        unstyled(TokenKind::StateIcon),
        unstyled(TokenKind::Text("remote".into(), TextRole::Secondary)),
        unstyled(TokenKind::Text("herdr-gpui".into(), TextRole::Workspace)),
    ];
    // The icon stays fixed while the two text tokens share remaining room.
    assert_eq!(budgets(&row, fixed, 30), [Some(1), Some(6), Some(10)]);
    assert_eq!(budgets(&row, fixed, 20), [Some(1), Some(6), Some(9)]);
    assert_eq!(budgets(&row, fixed, 8), [Some(1), Some(2), Some(1)]);
    assert_eq!(budgets(&row, fixed, 6), [Some(1), None, Some(4)]);
    assert_eq!(budgets(&row, fixed, 4), [Some(1), None, Some(2)]);
    assert_eq!(budgets(&row, fixed, 0), [Some(1), None, None]);
    assert_eq!(budgets(&[], fixed, 5), Vec::<Option<usize>>::new());
}

#[test]
fn budgets_give_wide_state_labels_their_display_width() {
    // An emoji state label occupies two cells; a one-cell budget clips it to "…".
    let row = [
        unstyled(TokenKind::Text("🟡".into(), TextRole::Status)),
        unstyled(TokenKind::Text("~".into(), TextRole::Workspace)),
    ];
    assert_eq!(budgets(&row, |_| 0, 30), [Some(2), Some(1)]);
}

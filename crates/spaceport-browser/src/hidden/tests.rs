use super::*;

#[test]
fn hidden_tabs_are_kept_per_workspace_until_they_close() {
    let scope = Scope::endpoint("local");
    let key = (scope.clone(), "w1".to_owned());
    let other = (scope.clone(), "w2".to_owned());
    let (a, b) = (Pick::Herdr("a".into()), Pick::Herdr("b".into()));
    let mut hidden = Hidden::default();
    hidden.hide(key.clone(), [a.clone(), b.clone(), a.clone()]);
    hidden.hide(other.clone(), [a.clone()]);
    assert!(hidden.hides(&key, &a) && hidden.hides(&key, &b));
    assert_eq!(hidden.0[&key].len(), 2);

    hidden.retain_open(&key, std::slice::from_ref(&b));
    assert!(!hidden.hides(&key, &a) && hidden.hides(&key, &b));
    hidden.retain_open(&key, &[]);
    assert!(!hidden.0.contains_key(&key));

    hidden.forget_workspaces(&scope, &["w2".to_owned()]);
    assert!(!hidden.hides(&other, &a));
}

#[test]
fn a_tab_shown_again_is_no_longer_hidden() {
    let key = (Scope::endpoint("local"), "w1".to_owned());
    let (a, b) = (Pick::Herdr("a".into()), Pick::Herdr("b".into()));
    let mut hidden = Hidden::default();
    hidden.hide(key.clone(), [a.clone(), b.clone()]);
    hidden.show(&key, &a);
    assert!(!hidden.hides(&key, &a) && hidden.hides(&key, &b));
    hidden.show(&key, &b);
    assert!(hidden.is_empty());
}

#![allow(clippy::unwrap_used)]
use super::*;

#[core::prelude::v1::test]
fn prepared_lines_follow_the_body() {
    let prepared = Prepared::default();
    let first = prepared.lines("- one");
    assert!(Rc::ptr_eq(&first, &prepared.lines("- one")));
    assert_eq!(prepared.lines("- two")[0].text, "two");
}

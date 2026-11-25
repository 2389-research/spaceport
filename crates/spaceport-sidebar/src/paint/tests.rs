use super::*;
use gpui::div;
use std::{cell::Cell, rc::Rc as Shared};

#[test]
fn the_plain_seams_are_the_documented_functions() {
    // `label` is the `text` field called, not a wrapper of its own, so a
    // consumer that swaps in its own painters changes what a row paints.
    assert!(std::ptr::fn_addr_eq(
        PLAIN.text,
        plain_text as fn(SharedString) -> AnyElement
    ));
    assert!(std::ptr::fn_addr_eq(
        PLAIN.mark_layer,
        in_place as fn(f32, f32, Rc<dyn Fn() -> AnyElement>) -> AnyElement
    ));
}

#[test]
fn a_plain_mark_is_built_where_it_sits() {
    let built = Shared::new(Cell::new(false));
    let called = built.clone();
    let _element = PLAIN.layered(12., 0.5, move || {
        called.set(true);
        div().into_any_element()
    });
    assert!(built.get(), "the seam must build its mark, not layer it");
}

#[test]
fn painters_are_copy_so_a_row_context_needs_no_lifetime_for_them() {
    fn copies(painters: Painters) -> (Painters, Painters) {
        (painters, painters)
    }
    let (a, b) = copies(PLAIN);
    assert!(std::ptr::fn_addr_eq(a.text, b.text));
}

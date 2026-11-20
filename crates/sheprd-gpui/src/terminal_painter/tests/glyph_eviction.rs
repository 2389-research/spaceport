// ABOUTME: Exercises cache adaptation through actual terminal painting after saturation.
// ABOUTME: Preserves hot oversized working sets while testing bounded ASCII and Unicode replacement.
use super::*;
use std::{cell::RefCell, rc::Rc};

fn frame(symbols: impl IntoIterator<Item = String>) -> FrameData {
    let cells: Vec<_> = symbols.into_iter().map(|symbol| cell(&symbol)).collect();
    assert!(!cells.is_empty());
    let mut columns = cells.len().min(100);
    while !cells.len().is_multiple_of(columns) {
        columns -= 1;
    }
    let width = u16::try_from(columns).unwrap_or(100);
    FrameData {
        width,
        height: u16::try_from(cells.len().div_ceil(usize::from(width))).unwrap_or(1),
        cells,
        cursor: None,
        hyperlinks: Vec::new(),
        graphics: Vec::new(),
    }
}

fn paint(painter: &Rc<RefCell<TerminalPainter>>, frame: FrameData, cx: &mut VisualTestContext) {
    let painter = painter.clone();
    cx.draw(Point::default(), size(px(800.), px(600.)), |_, _| {
        canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let font = font("Menlo");
                let cell_width = painter.borrow_mut().cell_width(&font, window, cx);
                painter.borrow_mut().paint_frame(
                    &frame,
                    bounds.origin,
                    None,
                    Ground::Opaque,
                    cell_width,
                    &font,
                    &[],
                    &[],
                    None,
                    None,
                    window,
                    cx,
                );
            },
        )
        .size_full()
    });
}

fn symbols(count: usize) -> impl Iterator<Item = String> {
    (0..count)
        .filter_map(|index| char::from_u32(0x4e00 + index as u32))
        .map(|symbol| symbol.to_string())
}

#[gpui::test]
fn a_full_cache_adapts_after_old_glyphs_stop_being_painted(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| Empty);
    let painter = Rc::new(RefCell::new(TerminalPainter::default()));
    paint(
        &painter,
        frame(std::iter::once("X".into()).chain(symbols(glyphs::CACHE_LIMIT - 1))),
        cx,
    );
    assert_eq!(painter.borrow().glyphs.len(), glyphs::CACHE_LIMIT);
    paint(&painter, frame(["新".into()]), cx);
    assert!(
        painter
            .borrow()
            .glyphs
            .iter()
            .any(|(_, symbol, _)| symbol == "X"),
        "the previous paint stays protected"
    );
    paint(&painter, frame(["新".into()]), cx);
    assert!(
        painter.borrow().glyphs.get(0, "新").is_some(),
        "cold entries must make room for current glyphs"
    );
    assert!(
        painter.borrow().glyphs.get(0, "X").is_none(),
        "the replaced ASCII index must clear"
    );
    paint(&painter, frame(["界".into()]), cx);
    assert!(painter.borrow().glyphs.get(0, "界").is_some());
    assert!(painter.borrow().glyphs.get(0, "新").is_some());
    assert!(
        painter.borrow().glyphs.get(0, "一").is_none(),
        "the replaced Unicode index must clear"
    );
    assert_eq!(painter.borrow().glyphs.len(), glyphs::CACHE_LIMIT);
    assert_eq!(painter.borrow().glyphs.iter().count(), glyphs::CACHE_LIMIT);
}

#[gpui::test]
fn an_oversized_repeated_screen_does_not_churn_its_cached_working_set(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| Empty);
    let painter = Rc::new(RefCell::new(TerminalPainter::default()));
    let grid = frame(symbols(glyphs::CACHE_LIMIT * 5 / 4));
    paint(&painter, grid.clone(), cx);
    let keys = |painter: &TerminalPainter| {
        let mut keys: Vec<_> = painter
            .glyphs
            .iter()
            .map(|(style, symbol, _)| (style, symbol))
            .collect();
        keys.sort_unstable();
        keys
    };
    let before = keys(&painter.borrow());
    paint(&painter, grid, cx);
    assert_eq!(keys(&painter.borrow()), before);
    assert_eq!(painter.borrow().glyphs.len(), glyphs::CACHE_LIMIT);
}

// ABOUTME: Preserves Core paint invalidation, text routing, and IME geometry across cached frames.
// ABOUTME: Exercises actual view painting and dispatch with the existing recording terminal worker.
use super::paste::terminal;
use crate::{
    config::Theme,
    core_terminals::{
        CoreTerminalView,
        selection::{Cell, Selection},
        view::PaneLook,
        worker::Command,
    },
    terminal_painter::Ground,
    window::native_images::Session,
};
use gpui::{Bounds, Entity, EntityInputHandler, TestAppContext, VisualTestContext, px, rgb, size};
use sheprd_backend::{ClientPaneInputEvent, Size, SurfaceUpdate};
use sheprd_terminal::{Damage, Emulator};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

struct Painted {
    view: Entity<CoreTerminalView>,
    received: Arc<Mutex<Vec<Command>>>,
    probe: Session,
}

fn frame(text: &str) -> sheprd_backend::FrameData {
    let mut emulator = Emulator::new(Size::new(80, 24).unwrap(), 0);
    emulator.feed(text.as_bytes());
    let Damage::Full(frame) = emulator.take_damage() else {
        panic!("a new emulator must produce its screen");
    };
    frame
}

fn painted(cx: &mut TestAppContext) -> (Painted, &mut VisualTestContext) {
    let (view, received, cx) = terminal(cx);
    let probe = cx
        .update(|window, _| Session::start(window.window_handle().window_id()))
        .unwrap();
    view.update(cx, |view, cx| {
        let mut look = view.look.clone();
        look.cell_width = 10.;
        look.cell_height = 20.;
        look.ground = Ground::Opaque;
        view.painter
            .borrow_mut()
            .set_appearance(14., 20., Theme::default());
        view.set_look(look, cx);
        view.screen
            .apply(SurfaceUpdate::Frame(frame("cache fixture")));
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(probe.frame().unwrap().cells > 0);
    probe.reset().unwrap();
    (
        Painted {
            view,
            received,
            probe,
        },
        cx,
    )
}

#[gpui::test]
fn changed_core_frame_reaches_the_painted_scene(cx: &mut TestAppContext) {
    let (case, cx) = painted(cx);
    case.view.update(cx, |view, cx| {
        view.screen.apply(SurfaceUpdate::Frame(frame(
            "\x1b[48;2;18;52;86mchanged cells\x1b[0m",
        )));
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(case.probe.frame().unwrap().cells > 0);
    cx.update(|window, _| {
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == rgb(0x123456).into()),
            "the changed frame's actual cell background must reach the scene"
        );
    });
}

#[gpui::test]
fn painter_appearance_changes_repaint_without_new_cells(cx: &mut TestAppContext) {
    let (case, cx) = painted(cx);
    for (font_size, cell_height, foreground) in [
        (14., 20., 0xabcdef),
        (18., 20., 0xabcdef),
        (18., 24., 0xabcdef),
    ] {
        case.probe.reset().unwrap();
        case.view.update(cx, |view, cx| {
            view.painter.borrow_mut().set_appearance(
                font_size,
                cell_height,
                Theme {
                    foreground,
                    ..Theme::default()
                },
            );
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(case.probe.frame().unwrap().cells > 0);
    }
}

#[gpui::test]
fn selection_and_veiled_cursor_changes_repaint_settled_cells(cx: &mut TestAppContext) {
    let (case, cx) = painted(cx);
    case.view.update(cx, |view, cx| {
        view.selection = Some(Selection::line(
            view.screen.frame.as_ref().unwrap(),
            Cell { row: 0, column: 0 },
        ));
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(case.probe.frame().unwrap().cells > 0);
    case.probe.reset().unwrap();
    case.view.update(cx, |view, cx| {
        view.selection = None;
        view.set_pane(Some(PaneLook {
            corners: (4., 4.),
            veil: Some(100),
        }));
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(case.probe.frame().unwrap().cells > 0);
}

#[gpui::test]
fn text_dispatch_survives_repeated_core_frames(cx: &mut TestAppContext) {
    let (case, cx) = painted(cx);
    for _ in 0..4 {
        case.view.update(cx, |_, cx| cx.notify());
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
    assert_eq!(case.probe.frame().unwrap().cells, 0);
    cx.simulate_input("z");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let inputs: Vec<_> = case
            .received
            .lock()
            .unwrap()
            .iter()
            .filter_map(|command| match command {
                Command::Input(input) => Some(input.clone()),
                _ => None,
            })
            .collect();
        if !inputs.is_empty() {
            assert_eq!(inputs, [ClientPaneInputEvent::TextCommit("z".into())]);
            break;
        }
        assert!(Instant::now() < deadline, "missing dispatched Core text");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[gpui::test]
fn missing_core_frame_removes_the_old_cell_scene(cx: &mut TestAppContext) {
    let (case, cx) = painted(cx);
    case.view.update(cx, |view, cx| {
        view.screen.apply(SurfaceUpdate::Frame(frame(
            "\x1b[48;2;18;52;86mvisible cells\x1b[0m",
        )));
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, _| {
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == rgb(0x123456).into())
        );
    });
    case.probe.reset().unwrap();
    case.view.update(cx, |view, cx| {
        view.screen.frame = None;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(case.probe.frame().unwrap().cells, 0);
    cx.update(|window, _| {
        assert!(
            window
                .painted_quads()
                .iter()
                .all(|quad| quad.background != rgb(0x123456).into())
        );
    });
}

#[gpui::test]
fn moved_core_canvas_updates_resize_and_ime_bounds(cx: &mut TestAppContext) {
    let (case, cx) = painted(cx);
    let mut observations = Vec::new();
    for (extent, pane) in [
        (size(px(800.), px(600.)), None),
        (
            size(px(400.), px(300.)),
            Some(PaneLook {
                corners: (4., 4.),
                veil: None,
            }),
        ),
    ] {
        case.view.update(cx, |view, cx| {
            view.set_pane(pane);
            cx.notify();
        });
        cx.simulate_resize(extent);
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        observations.push(case.view.update_in(cx, |view, window, cx| {
            (
                view.bounds().unwrap(),
                view.size().unwrap(),
                view.bounds_for_range(0..0, Bounds::default(), window, cx)
                    .unwrap(),
            )
        }));
    }
    let [
        (old_bounds, old_size, old_ime),
        (new_bounds, new_size, new_ime),
    ] = observations.as_slice()
    else {
        panic!("the geometry control needs both views");
    };
    assert!(new_size.cols() < old_size.cols());
    assert!(new_size.rows() < old_size.rows());
    assert_ne!(old_bounds.origin, new_bounds.origin);
    assert_eq!(
        new_ime.origin - old_ime.origin,
        new_bounds.origin - old_bounds.origin
    );
    assert!(new_bounds.contains(&new_ime.origin));
}

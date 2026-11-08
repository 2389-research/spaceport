//! An observer of a private Core terminal cannot type or size its program.

use super::*;
use sheprd_backend::{
    Backend, ClientPaneInputEvent, Role, Size, Spawn, SurfaceUpdate, Terminal as _,
};

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn observer_input_and_resize_leave_the_program_for_its_active_view() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut sandbox = Sandbox::start(binary);
    let backend = crate::CoreBackend::new(sandbox.connect(), 1000);
    let info = backend
        .spawn(&Spawn {
            argv: vec![
                "sh".into(),
                "-c".into(),
                "IFS= read -r line; printf 'accepted:%s\\n' \"$line\"; sleep 30".into(),
            ],
            ..Spawn::default()
        })
        .unwrap();
    let before = backend.core().terminal_get(info.id.as_str()).unwrap();

    let mut observer = backend
        .attach(&info.id, Size::new(9, 3).unwrap(), Role::Observer)
        .unwrap();
    observer
        .input(&ClientPaneInputEvent::TextCommit("wrong".into()))
        .unwrap();
    observer
        .input(&ClientPaneInputEvent::Key {
            code: sheprd_protocol::ClientKeyCode::Enter,
            modifiers: 0,
            kind: sheprd_protocol::ClientKeyKind::Press,
            repeat_count: 1,
            shifted_codepoint: None,
            generated_text: None,
            tracks_release: false,
            physical_key_id: None,
            windows_record: None,
        })
        .unwrap();
    observer.resize(Size::new(10, 4).unwrap()).unwrap();
    observer.focus(true).unwrap();
    let after = backend.core().terminal_get(info.id.as_str()).unwrap();
    assert_eq!((after.cols, after.rows), (before.cols, before.rows));

    let mut active = backend
        .attach(&info.id, Size::new(40, 6).unwrap(), Role::Active)
        .unwrap();
    active
        .input(&ClientPaneInputEvent::TextCommit("right".into()))
        .unwrap();
    active
        .input(&ClientPaneInputEvent::Key {
            code: sheprd_protocol::ClientKeyCode::Enter,
            modifiers: 0,
            kind: sheprd_protocol::ClientKeyKind::Press,
            repeat_count: 1,
            shifted_codepoint: None,
            generated_text: None,
            tracks_release: false,
            physical_key_id: None,
            windows_record: None,
        })
        .unwrap();

    let mut screen = Vec::new();
    let mut width = 0;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        for update in active.next(Duration::from_millis(100)).unwrap() {
            match update {
                SurfaceUpdate::Frame(frame) => {
                    width = usize::from(frame.width);
                    screen = frame.cells;
                }
                SurfaceUpdate::Rows { rows, .. } => {
                    for row in rows {
                        let start = usize::from(row.y) * width + usize::from(row.x);
                        for (offset, cell) in row.cells.into_iter().enumerate() {
                            if let Some(slot) = screen.get_mut(start + offset) {
                                *slot = cell;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let text: String = screen
            .iter()
            .map(|cell: &sheprd_protocol::CellData| cell.symbol.as_str())
            .collect();
        assert!(!text.contains("accepted:wrong"), "{text}");
        if text.contains("accepted:right") {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "active input had no answer: {text}"
        );
    }

    drop(observer);
    drop(active);
    backend.kill(&info.id).unwrap();
    sandbox.stop().unwrap();
}

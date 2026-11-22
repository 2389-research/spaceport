//! The pinned Core sends an announced observer grid before its replacement paint.
use super::*;

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn a_private_core_orders_observer_resize_before_replacement_paint() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut sandbox = Sandbox::start(binary);
    let core = sandbox.connect();
    assert!(
        core.hello()
            .features
            .iter()
            .any(|feature| feature == "resize_notifications")
    );
    let argv = ["sh".to_owned(), "-c".to_owned(), "sleep 30".to_owned()];
    let started = core
        .run(&Run {
            argv: &argv,
            ..Run::default()
        })
        .unwrap();
    let active = core
        .attach(&Attach {
            cols: 80,
            rows: 24,
            terminal: &started.address,
            ..Attach::default()
        })
        .unwrap();
    let observer = core
        .attach(&Attach {
            cols: 80,
            observe: true,
            resize_notifications: true,
            rows: 24,
            terminal: &started.address,
            ..Attach::default()
        })
        .unwrap();
    assert!(observer.resize_notifications());

    let initial_deadline = Instant::now() + Duration::from_secs(5);
    let (mut initial_size, mut initial_paint) = (false, false);
    while !(initial_size && initial_paint) {
        assert!(
            Instant::now() < initial_deadline,
            "initial observer paint missing"
        );
        let chunk = observer.read(Duration::from_millis(100)).unwrap();
        initial_size |= chunk
            .resizes
            .iter()
            .any(|resize| (resize.size.cols(), resize.size.rows()) == (80, 24));
        initial_paint |= !chunk.bytes.is_empty();
    }

    active.resize(90, 25).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut announced = false;
    let mut painted = false;
    while !painted {
        assert!(Instant::now() < deadline, "replacement paint missing");
        let chunk = observer.read(Duration::from_millis(100)).unwrap();
        let mut offset = 0;
        for resize in &chunk.resizes {
            if announced && resize.at > offset {
                assert_eq!(chunk.bytes[offset], 0x18, "paint starts with CAN");
                painted = true;
            }
            if (resize.size.cols(), resize.size.rows()) == (90, 25) {
                announced = true;
            }
            offset = resize.at;
        }
        if announced && chunk.bytes.len() > offset {
            assert_eq!(chunk.bytes[offset], 0x18, "paint starts with CAN");
            painted = true;
        }
    }
    assert!(announced);
    drop(observer);
    drop(active);
    drop(core);
    sandbox.stop().unwrap();
}

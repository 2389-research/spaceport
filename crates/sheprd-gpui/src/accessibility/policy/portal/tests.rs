// ABOUTME: Checks portal preference types, absent keys, and unknown enum values.
// ABOUTME: Uses the actual D-Bus variant decoder consumed by Linux policy reads.
#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn portal_values_keep_unknown_keys_and_wrong_types_distinct() {
    let mut settings = Settings::new();
    assert_eq!(decode(&settings), Preferences::default());
    settings.insert(
        APPEARANCE.into(),
        HashMap::from([
            ("reduced-motion".into(), OwnedValue::from(1_u32)),
            ("contrast".into(), OwnedValue::from(0_u32)),
        ]),
    );
    assert_eq!(
        decode(&settings),
        Preferences {
            reduce_motion: Some(true),
            high_contrast: Some(false)
        }
    );
    let values = settings.get_mut(APPEARANCE).unwrap();
    values.insert("reduced-motion".into(), OwnedValue::from(99_u32));
    values.insert("contrast".into(), OwnedValue::from(true));
    assert_eq!(
        decode(&settings),
        Preferences {
            reduce_motion: Some(false),
            high_contrast: None
        }
    );
}

struct Portal;
#[zbus::interface(name = "org.freedesktop.portal.Settings")]
impl Portal {
    fn read_all(&self, namespaces: Vec<String>) -> Settings {
        assert_eq!(namespaces, [APPEARANCE]);
        HashMap::from([(
            APPEARANCE.into(),
            HashMap::from([
                ("reduced-motion".into(), OwnedValue::from(1_u32)),
                ("contrast".into(), OwnedValue::from(1_u32)),
            ]),
        )])
    }
}

#[test]
fn portal_read_all_uses_the_real_dbus_wire() {
    use zbus::connection::Builder;
    let (finish, finished) = std::sync::mpsc::channel::<()>();
    let (timeout, timed_out) = futures_channel::oneshot::channel();
    let timer = std::thread::spawn(move || {
        if finished
            .recv_timeout(std::time::Duration::from_secs(5))
            .is_err()
        {
            let _ = timeout.send(());
        }
    });
    let result = futures_lite::future::block_on(async {
        let read = async {
            let (server, client) = std::os::unix::net::UnixStream::pair().unwrap();
            let server = Builder::async_io_unix_stream(server)
                .server(zbus::Guid::generate())
                .unwrap()
                .p2p()
                .serve_at("/org/freedesktop/portal/desktop", Portal)
                .unwrap()
                .build();
            let client = Builder::async_io_unix_stream(client).p2p().build();
            let (server, client) = futures_lite::future::zip(server, client).await;
            let _server = server.unwrap();
            let client = client.unwrap();
            Some(read_from(&client).await.unwrap())
        };
        futures_lite::future::race(read, async {
            let _ = timed_out.await;
            None
        })
        .await
    });
    drop(finish);
    timer.join().unwrap();
    assert_eq!(
        result,
        Some(Preferences {
            reduce_motion: Some(true),
            high_contrast: Some(true)
        })
    );
}

#[test]
fn a_deadline_drops_a_stalled_read_and_preserves_unknown_preferences() {
    struct ReadGuard(std::rc::Rc<std::cell::Cell<bool>>);
    impl Drop for ReadGuard {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    let dropped = std::rc::Rc::new(std::cell::Cell::new(false));
    let guard = ReadGuard(dropped.clone());
    let read = async move {
        let _guard = guard;
        std::future::pending::<zbus::Result<Preferences>>().await
    };
    let result = futures_lite::future::block_on(with_deadline(read, std::future::ready(())));
    assert_eq!(result, Preferences::default());
    assert!(dropped.get());
}

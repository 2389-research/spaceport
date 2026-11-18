// ABOUTME: Verifies explicit missing-Herdr recovery through a real framed connection.
// ABOUTME: Keeps the guidance until a replacement handshake and coherent picture arrive.
use super::*;
use crate::presentation::Presentation;

#[test]
fn explicit_retry_clears_missing_guidance_after_a_real_replacement_handshake() {
    let (mut endpoint, old_server) = connected_endpoint(LOCAL);
    let mut presentation = Presentation::default();
    let before = presentation.picture(&endpoint.live).unwrap();
    lose_herdr(&mut endpoint);
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.herdr_missing() && endpoint.connection.retry_ready()
    });
    presentation.hold();
    endpoint.set_retry_at_for_test(Instant::now());
    assert!(
        !endpoint.may_dial(),
        "waiting must not restart a missing installation"
    );
    assert!(endpoint.herdr_missing());
    drop(old_server);

    let generation = endpoint.generation;
    let mut server = start_endpoint(&mut endpoint);
    assert!(
        endpoint.generation > generation,
        "the explicit retry owns a fresh attempt"
    );
    assert!(
        endpoint.herdr_missing(),
        "a socket accept alone is not successful recovery"
    );
    send_welcome(&mut server, None);
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.serving()
    });
    assert!(!endpoint.herdr_missing());
    assert!(endpoint.outage().is_none());
    presentation.resume();
    let held = presentation.picture(&endpoint.live).unwrap();
    assert!(Arc::ptr_eq(&held.frame, &before.frame));
    assert!(
        presentation.stale(),
        "keep the card until the replacement picture is ready"
    );

    let mut frame = (*surface(endpoint.live.snapshot.as_ref().unwrap())).clone();
    frame.surface_revision = before.frame.surface_revision + 1;
    frame.frame.cells = vec![
        CellData {
            symbol: " ".into(),
            fg: 0,
            bg: 0,
            modifier: 0,
            skip: false,
            hyperlink: None,
        };
        usize::from(frame.frame.width) * usize::from(frame.frame.height)
    ];
    frame.frame.cells[0].symbol = "R".into();
    write_message(
        &mut server.stream,
        &ServerMessage::PaneSurface(frame),
        MAX_GRAPHICS_FRAME_SIZE,
    )
    .unwrap();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.live.surface_ready()
    });
    let ready = presentation.picture(&endpoint.live).unwrap();
    assert_eq!(ready.frame.frame.cells[0].symbol, "R");
    assert!(!presentation.stale());
    assert!(!endpoint.herdr_missing());
    assert!(endpoint.outage().is_none());
}

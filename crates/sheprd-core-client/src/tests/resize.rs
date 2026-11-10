//! Confirmed observer size changes stay ordered with terminal bytes.
use super::{MockCore, connected, every_method, hello_reply, pair, terminal};
use crate::{Core, CoreBackend, End, Violation, frame::Kind, stream::Route, wire::Attach};
use serde_json::json;
use sheprd_backend::{Backend, Role, Size, SurfaceUpdate, Terminal as _, TerminalId};
use std::{io::Write as _, thread, time::Duration};

const WAIT: Duration = Duration::from_secs(5);

#[test]
fn resize_frames_have_exactly_one_valid_bounded_grid() {
    let route = Route::new(true);
    for payload in [
        &[][..],
        &[80, 0, 24][..],
        &[80, 0, 24, 0, 0][..],
        &[0, 0, 24, 0][..],
        &[1, 16, 24, 0][..],
        &[160, 15, 160, 15][..],
    ] {
        assert_eq!(route.push_resize(payload), Err(Violation::InvalidResize));
    }
}

#[test]
fn resize_markers_cannot_grow_without_a_reader() {
    let route = Route::new(true);
    let mut limit = None;
    for count in 0..8192 {
        route.push(b"x").unwrap();
        if let Err(error) = route.push_resize(&[80, 0, 24, 0]) {
            limit = Some((count, error));
            break;
        }
    }
    assert!(matches!(limit, Some((count, Violation::TooManyResizes)) if count > 0));
}

fn supporting_core() -> (Core, MockCore) {
    let (client, server) = pair();
    let mut reply = hello_reply("0.1.13", &every_method());
    reply["result"]["features"] = json!(["resize_notifications"]);
    let server = thread::spawn(move || MockCore::accept(server, &reply));
    let client = Core::over(client).unwrap();
    let (server, hello) = server.join().unwrap();
    assert_eq!(hello["params"]["features"], json!(["resize_notifications"]));
    (client, server)
}

#[test]
fn confirmed_observer_keeps_resize_before_later_data_and_credits_only_bytes() {
    let (client, mut core) = supporting_core();
    let attach = thread::spawn(move || {
        let attachment = client.attach(&Attach {
            cols: 80,
            observe: true,
            resize_notifications: true,
            rows: 24,
            terminal: "t1",
            ..Attach::default()
        });
        (client, attachment)
    });
    let request = core.request();
    assert_eq!(request["params"]["resize_notifications"], true);
    core.answer(
        &request,
        json!({"channel": 5, "terminal": terminal("t1"),
        "resize_notifications": true}),
    );
    core.send(5, Kind::Resize, &[80, 0, 24, 0]);
    core.send(5, Kind::Data, b"a");
    core.send(5, Kind::Resize, &[30, 0, 6, 0]);
    core.send(5, Kind::Data, b"b");

    let (_client, attachment) = attach.join().unwrap();
    let attachment = attachment.unwrap();
    assert!(attachment.resize_notifications());
    let mut seen = Vec::new();
    while seen.len() < 4 {
        let chunk = attachment.read(WAIT).unwrap();
        let mut start = 0;
        for resize in chunk.resizes {
            seen.extend(
                chunk.bytes[start..resize.at]
                    .iter()
                    .map(|byte| format!("{}", *byte as char)),
            );
            seen.push(format!("{}x{}", resize.size.cols(), resize.size.rows()));
            start = resize.at;
        }
        seen.extend(
            chunk.bytes[start..]
                .iter()
                .map(|byte| format!("{}", *byte as char)),
        );
    }
    assert_eq!(seen, ["80x24", "a", "30x6", "b"]);
    let mut credited = 0;
    while credited < 2 {
        let credit = core.frame();
        assert_eq!((credit.channel, credit.kind), (5, Kind::Credit));
        credited += u32::from_le_bytes(credit.payload.try_into().unwrap());
    }
    assert_eq!(credited, 2);
}

#[test]
fn malformed_resize_ends_the_connection_without_delivering_later_data() {
    let (client, mut core) = supporting_core();
    let attach = thread::spawn(move || {
        let attachment = client.attach(&Attach {
            cols: 80,
            observe: true,
            resize_notifications: true,
            rows: 24,
            terminal: "t1",
            ..Attach::default()
        });
        (client, attachment)
    });
    let request = core.request();
    core.answer(
        &request,
        json!({"channel": 5, "terminal": terminal("t1"),
        "resize_notifications": true}),
    );
    let (_client, attachment) = attach.join().unwrap();
    let attachment = attachment.unwrap();
    // Queue both frames together: a malformed resize may close the socket
    // before the peer can make a second write.
    let mut frames = crate::frame::encode(5, Kind::Resize, &[90, 0, 25]).unwrap();
    frames.extend(crate::frame::encode(5, Kind::Data, b"unsafe").unwrap());
    core.socket.write_all(&frames).unwrap();
    let chunk = attachment.read(WAIT).unwrap();
    assert!(matches!(chunk.end, Some(End::Lost)));
    assert!(chunk.bytes.is_empty());
}

#[test]
fn an_unrequested_resize_is_a_protocol_error_even_if_reply_claims_support() {
    let (client, mut core) = supporting_core();
    let attach = thread::spawn(move || {
        let attachment = client.attach(&Attach {
            cols: 80,
            rows: 24,
            terminal: "t1",
            ..Attach::default()
        });
        (client, attachment)
    });
    let request = core.request();
    assert!(request["params"].get("resize_notifications").is_none());
    core.answer(
        &request,
        json!({"channel": 5, "terminal": terminal("t1"),
        "resize_notifications": true}),
    );
    let (_client, attachment) = attach.join().unwrap();
    let attachment = attachment.unwrap();
    assert!(!attachment.resize_notifications());
    core.send(5, Kind::Resize, &[80, 0, 24, 0]);
    assert!(matches!(
        attachment.read(WAIT).unwrap().end,
        Some(End::Lost)
    ));
}

#[test]
fn a_core_without_the_hello_feature_gets_the_old_attach_request() {
    let (client, mut core) = connected();
    let attach = thread::spawn(move || {
        let attachment = client.attach(&Attach {
            cols: 80,
            observe: true,
            resize_notifications: true,
            rows: 24,
            terminal: "t1",
            ..Attach::default()
        });
        (client, attachment)
    });
    let request = core.request();
    assert!(request["params"].get("resize_notifications").is_none());
    core.answer(&request, json!({"channel": 5, "terminal": terminal("t1")}));
    let (_client, attachment) = attach.join().unwrap();
    assert!(!attachment.unwrap().resize_notifications());
}

#[test]
fn confirmed_observer_resizes_its_emulator_before_replacement_paint() {
    let (client, mut core) = supporting_core();
    let backend = CoreBackend::new(client, 100);
    let mut observer = thread::scope(|scope| {
        let attach = scope.spawn(|| {
            backend.attach(
                &TerminalId::new("t1").unwrap(),
                Size::new(20, 4).unwrap(),
                Role::Observer,
            )
        });
        let request = core.request();
        assert_eq!(request["params"]["observe"], true);
        assert_eq!(request["params"]["resize_notifications"], true);
        core.answer(
            &request,
            json!({"channel": 5, "terminal": terminal("t1"),
            "resize_notifications": true}),
        );
        attach.join().unwrap().unwrap()
    });
    core.send(5, Kind::Resize, &[20, 0, 4, 0]);
    core.send(5, Kind::Data, b"old");
    core.send(5, Kind::Resize, &[30, 0, 6, 0]);
    core.send(5, Kind::Data, b"\x18\x1b[2J\x1b[Hnew");

    let mut painted = false;
    for _ in 0..12 {
        let updates = observer.next(Duration::from_millis(100)).unwrap();
        painted |= updates.iter().any(|update| match update {
            SurfaceUpdate::Frame(frame) if (frame.width, frame.height) == (30, 6) => {
                frame.cells[0].symbol == "n" && frame.cells[1].symbol == "e"
            }
            _ => false,
        });
        if painted {
            break;
        }
    }
    assert!(painted, "replacement paint used the announced grid");
}

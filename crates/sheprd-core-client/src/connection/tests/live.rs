// ABOUTME: Verifies abandoned streams against an explicitly selected Core runtime.
// ABOUTME: Holds actual replies until timeout and observes successful CLOSE writes.

use super::*;
use crate::{tests::live::Sandbox, wire::Run};

const WAIT: Duration = Duration::from_secs(5);

struct HeldAnswer {
    socket: sheprd_client::Stream,
    framed: bool,
    hold: Option<(Sender<()>, Receiver<()>)>,
    written: Decoder,
    closes: Sender<u32>,
}

impl Read for HeldAnswer {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let count = self.socket.read(bytes)?;
        if count > 0 {
            if self.framed {
                if let Some((ready, release)) = self.hold.take() {
                    ready.send(()).map_err(io::Error::other)?;
                    release.recv_timeout(WAIT).map_err(io::Error::other)?;
                }
            } else {
                self.framed = bytes[..count].contains(&b'\n');
            }
        }
        Ok(count)
    }
}

impl Write for HeldAnswer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.socket.write(bytes)?;
        if self.framed {
            self.written.push(&bytes[..written]);
            while let Some(frame) = self.written.next_frame().map_err(io::Error::other)? {
                if frame.kind == Kind::Close {
                    self.closes
                        .try_send(frame.channel)
                        .map_err(io::Error::other)?;
                }
            }
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.socket.flush()
    }
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn real_core_late_streams_close_without_ending_the_terminal() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let sandbox = Sandbox::start(binary);
    let control = sandbox.connect();
    let argv = ["sh".to_owned(), "-c".to_owned(), "sleep 30".to_owned()];
    let terminal = control
        .run(&Run {
            argv: &argv,
            ..Run::default()
        })
        .unwrap();
    let endpoint = Endpoint::resolve(|name| sandbox.var(name)).unwrap();

    for method in [Method::TerminalAttach, Method::EventsSubscribe] {
        let (ready, held) = crossbeam_channel::bounded(1);
        let (release, gate) = crossbeam_channel::bounded(1);
        let (closes, closed) = crossbeam_channel::bounded(1);
        let client = Arc::new(
            Core::over(HeldAnswer {
                socket: connect_stream(&endpoint.socket).unwrap(),
                framed: false,
                hold: Some((ready, gate)),
                written: Decoder::default(),
                closes,
            })
            .unwrap(),
        );
        let params = match method {
            Method::TerminalAttach => json!({"terminal": terminal.address, "cols": 80, "rows": 24}),
            _ => json!({}),
        };
        let call = thread::spawn({
            let client = Arc::clone(&client);
            move || client.call_with_timeout(method, &params, false, Duration::from_millis(100))
        });
        held.recv_timeout(WAIT)
            .expect("Core did not answer the submitted call");
        assert!(matches!(call.join().unwrap(), Err(Error::Timeout(found)) if found == method));
        release.send(()).unwrap();
        assert!(
            closed
                .recv_timeout(WAIT)
                .expect("late channel was not closed")
                > 0
        );
        assert!(!client.is_lost());
        assert!(locked(&client.shared.pending).is_empty());
        assert!(locked(&client.shared.routes).is_empty());
        let alive = control.terminal_get(&terminal.address).unwrap();
        assert_eq!(alive.address, terminal.address);
        assert_eq!(alive.state, wire::TerminalState::Live);
    }
}

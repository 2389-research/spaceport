// ABOUTME: Tests activity acknowledgements and retries through the reporter worker.
// ABOUTME: Uses an isolated socket and coordinated failures without a live daemon.

#![cfg(unix)]

use super::*;
use std::os::unix::net::UnixListener;

struct Fixture {
    reporter: Option<Reporter>,
    requests: crossbeam_channel::Receiver<Value>,
    server: Option<thread::JoinHandle<()>>,
    _dir: tempfile::TempDir,
}

impl Fixture {
    fn start(fail: &'static str, release: Option<crossbeam_channel::Receiver<()>>) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("api.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let (sender, requests) = crossbeam_channel::unbounded::<Value>();
        let server = thread::spawn(move || {
            let mut failed = false;
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut line = String::new();
                BufReader::new(stream.try_clone().unwrap())
                    .read_line(&mut line)
                    .unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                let fails = !failed && request["method"] == fail;
                let closing = request["method"] == "pane.release_agent";
                sender.send(request).unwrap();
                let response = if fails {
                    failed = true;
                    if let Some(release) = &release {
                        release.recv_timeout(Duration::from_secs(10)).unwrap();
                    }
                    json!({"error": {"code": "busy"}})
                } else {
                    json!({"result": {"type": "ok"}})
                };
                writeln!(stream, "{response}").unwrap();
                if closing {
                    break;
                }
            }
        });
        let reporter = Reporter::start(
            Target {
                pane: "p_1".to_owned(),
                socket,
            },
            "Claude".to_owned(),
            None,
        )
        .unwrap();
        Self {
            reporter: Some(reporter),
            requests,
            server: Some(server),
            _dir: dir,
        }
    }

    fn report(&self, activity: Activity) {
        self.reporter.as_ref().unwrap().handle().report(activity);
    }

    fn until(&self, method: &str) -> Value {
        loop {
            let request = self.requests.recv_timeout(Duration::from_secs(5)).unwrap();
            if request["method"] == method {
                return request;
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(reporter) = self.reporter.take() {
            reporter.close();
        }
        self.server.take().unwrap().join().unwrap();
    }
}

#[test]
fn a_failed_activity_is_retried_without_another_session_update() {
    let fixture = Fixture::start("pane.report_agent", None);
    fixture.report(Activity::Idle);
    let first = fixture.until("pane.report_agent");
    let retry = fixture.until("pane.report_agent");
    assert_eq!(first["params"]["state"], "idle");
    assert_eq!(retry["params"]["state"], "idle");
    assert!(retry["params"]["seq"].as_u64() > first["params"]["seq"].as_u64());
    fixture.until("pane.get");
}

#[test]
fn a_failed_authority_check_keeps_the_activity_pending() {
    let fixture = Fixture::start("pane.get", None);
    fixture.report(Activity::Working);
    fixture.until("pane.report_agent");
    fixture.until("pane.get");
    assert_eq!(
        fixture.until("pane.report_agent")["params"]["state"],
        "working"
    );
    fixture.until("pane.get");
}

#[test]
fn newer_activity_supersedes_an_in_flight_failure() {
    let (release, wait) = crossbeam_channel::bounded(1);
    let fixture = Fixture::start("pane.report_agent", Some(wait));
    fixture.report(Activity::Working);
    fixture.until("pane.report_agent");
    fixture.report(Activity::Blocked);
    fixture.report(Activity::Idle);
    release.send(()).unwrap();
    assert_eq!(
        fixture.until("pane.report_agent")["params"]["state"],
        "idle"
    );
    fixture.until("pane.get");
}

#[test]
fn closing_withdraws_after_a_failed_report_without_retrying() {
    let mut fixture = Fixture::start("pane.report_agent", None);
    fixture.report(Activity::Working);
    fixture.until("pane.report_agent");
    fixture.until("pane.report_metadata");
    fixture.reporter.take().unwrap().close();
    let methods: Vec<_> = fixture
        .requests
        .try_iter()
        .map(|request| request["method"].clone())
        .collect();
    assert_eq!(
        methods,
        [json!("pane.report_metadata"), json!("pane.release_agent")]
    );
}

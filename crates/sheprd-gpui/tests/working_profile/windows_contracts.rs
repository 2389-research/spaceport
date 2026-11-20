// ABOUTME: Requires Windows CPU clocks and native facts through the real profile schema.
// ABOUTME: Keeps platform qualification distinct from macOS AppKit observations.
use super::{measurements, report};
use anyhow::Result;
use serde_json::json;
use std::time::Duration;

#[path = "../../src/working_profile/pipe.rs"]
mod pipe;

#[test]
fn windows_cpu_clock_preserves_100ns_ticks() -> Result<()> {
    assert_eq!(
        measurements::cpu_ticks("12500001\r\n")?,
        Duration::new(1, 250_000_100)
    );
    Ok(())
}

#[test]
fn windows_cpu_ticks_refuse_invalid_or_overflowing_timespan_values() {
    for value in ["", "-1", "+1", "1.5", "1 2", "9223372036854775808"] {
        assert!(
            measurements::cpu_ticks(value).is_err(),
            "accepted {value:?}"
        );
    }
}

#[test]
fn windows_observation_does_not_invent_appkit_facts() -> Result<()> {
    let observation: report::Observation = serde_json::from_value(json!({
        "request_id":1,"gui_pid":42,"release":true,"window_id":1,
        "reduce_motion":false,"mark_layer_render_calls":0,
        "window_active":true,"window_visible":true,
        "native_key":null,"app_active":null,"app_hidden":null,
        "native_activation":"Win32 HWND supplied by the real window",
        "windows_hwnd":1234,"workspace":null,"synthetic_working_panes":[],"groups":[]
    }))?;
    let actual = serde_json::to_value(observation)?;
    assert!(actual["native_key"].is_null());
    assert_eq!(actual["windows_hwnd"], 1234);
    Ok(())
}

#[test]
fn private_pipe_refuses_rebinding_and_buffers_a_whole_bounded_reply() -> Result<()> {
    use std::io::{Read as _, Write as _};
    let directory = tempfile::Builder::new().prefix("hprofile-").tempdir()?;
    let endpoint = report::endpoint(directory.path())?;
    let listener = pipe::Listener::bind(&endpoint)?;
    assert!(pipe::Listener::bind(&endpoint).is_err());
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    let server = std::thread::spawn(move || -> Result<()> {
        let mut stream = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        let mut request = [0; 3];
        stream.read_exact(&mut request)?;
        anyhow::ensure!(&request == b"go\n", "wrong pipe request");
        stream.write_all(&vec![b'x'; report::MAX_LINE as usize + 1])?;
        send.send(())?;
        Ok(())
    });
    let mut client = sheprd_client::Stream::connect_owned(&endpoint)?;
    client.set_read_timeout(Some(Duration::from_secs(3)))?;
    client.set_write_timeout(Some(Duration::from_secs(3)))?;
    client.write_all(b"go\n")?;
    // Completion before reading proves the private server's whole-reply buffer policy.
    receive.recv_timeout(Duration::from_secs(3))?;
    let mut reply = vec![0; report::MAX_LINE as usize + 1];
    client.read_exact(&mut reply)?;
    assert!(reply.iter().all(|byte| *byte == b'x'));
    server
        .join()
        .map_err(|_| anyhow::anyhow!("profile pipe server panicked"))??;
    Ok(())
}

#[test]
fn private_pipe_idle_read_has_a_real_deadline() -> Result<()> {
    use std::io::Read as _;
    let directory = tempfile::Builder::new().prefix("hprofile-").tempdir()?;
    let endpoint = report::endpoint(directory.path())?;
    let listener = pipe::Listener::bind(&endpoint)?;
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    let server = std::thread::spawn(move || -> Result<()> {
        let mut stream = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_millis(20)))?;
        let result = stream.read(&mut [0]);
        send.send(result)?;
        Ok(())
    });
    let _client = sheprd_client::Stream::connect_owned(&endpoint)?;
    let result = receive.recv_timeout(Duration::from_secs(3))?;
    let Err(error) = result else {
        anyhow::bail!("idle pipe read unexpectedly completed");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    server
        .join()
        .map_err(|_| anyhow::anyhow!("profile pipe server panicked"))??;
    Ok(())
}

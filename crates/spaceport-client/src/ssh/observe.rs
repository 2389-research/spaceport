// ABOUTME: Opens a running Herdr session's client socket for an inactive observer.
// ABOUTME: Uses Herdr's socket-only byte forwarder so observation cannot start a daemon.

use super::{CANDIDATES, await_ready, compatible_status, quote};
use crate::Result;
use std::{
    io::{Read, Write},
    path::Path,
    sync::atomic::AtomicBool,
    time::Instant,
};

pub(crate) fn command(session: &str) -> String {
    let script = format!(
        r#"{CANDIDATES}
    [ -n "$path" ] && [ -x "$path" ] || continue
    client=$("$path" status client --json </dev/null) || continue
    server=$("$path" --session {session} status server --json </dev/null) || continue
    printf '%s\nherdr-observer:%s\n%s\n' "$client" "$server" 'herdr-remote-output-ready:1'
    IFS= read -r socket || exit 1
    case "$socket" in
        /*) HERDR_SOCKET_PATH="$socket" exec "$path" remote-api-bridge;;
    esac
done
exit 1"#,
        session = quote(session)
    );
    format!("/bin/sh -c {}", quote(&script))
}

fn client_socket(banner: &[u8], session: &str) -> Option<String> {
    compatible_status(banner)?;
    let status = banner
        .split(|byte| *byte == b'\n')
        .find_map(|line| line.strip_prefix(b"herdr-observer:"))?;
    let server: serde_json::Value = serde_json::from_slice(status).ok()?;
    let named = server.get("session")?;
    let matches_session = if session == "default" {
        named.is_null()
    } else {
        named.as_str() == Some(session)
    };
    if server["running"] != true || server["endpoint_compatible"] != true || !matches_session {
        return None;
    }
    let socket = server["socket"].as_str()?;
    let path = Path::new(socket);
    if !socket.starts_with('/')
        || socket.len() > 4096
        || socket.chars().any(char::is_control)
        || path.file_name()? != "herdr.sock"
    {
        return None;
    }
    path.with_file_name("herdr-client.sock")
        .to_str()
        .map(str::to_owned)
}

pub(crate) fn handshake(
    stream: &mut (impl Read + Write),
    stop: &AtomicBool,
    session: &str,
) -> Result<()> {
    let started = Instant::now();
    loop {
        let banner = await_ready(stream, stop, started)?;
        if let Some(socket) = client_socket(&banner, session) {
            writeln!(stream, "{socket}")?;
            return Ok(());
        }
        stream.write_all(b"skip\n")?;
    }
}

#[cfg(test)]
mod tests;

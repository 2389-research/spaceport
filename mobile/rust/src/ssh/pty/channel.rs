// ABOUTME: Resolves the confirmed pane and attaches using the negotiated Herdr executable.
// ABOUTME: Keeps stdout, input, resize and close on a single bounded SSH PTY channel.
use super::open::{Opener, SshChannel};
use super::*;
use russh::ChannelMsg;
use spaceport_client::shell_quote;
use std::time::Duration;
use tokio::time::timeout;

pub(super) const SETUP: Duration = Duration::from_secs(10);

pub(super) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b":._-".contains(&c))
}

pub(super) async fn stopped(stop: &mut watch::Receiver<bool>) {
    let _ = stop.wait_for(|s| *s).await;
}

async fn ack(channel: &mut SshChannel) -> Result<(), MobileError> {
    loop {
        match channel.wait().await {
            Some(ChannelMsg::Success) => return Ok(()),
            Some(ChannelMsg::WindowAdjusted { .. }) => (),
            _ => return Err(MobileError::TerminalUnavailable),
        }
    }
}

async fn lookup(channel: &mut SshChannel, herdr: &str, pane: &str) -> Result<String, MobileError> {
    channel
        .exec(true, format!("{herdr} pane get {}", shell_quote(pane)))
        .await
        .map_err(|_| MobileError::TerminalUnavailable)?;
    ack(channel).await?;
    let mut data = Vec::new();
    let mut success = false;
    loop {
        match channel.wait().await {
            Some(ChannelMsg::Data { data: chunk }) if data.len() + chunk.len() <= 256 * 1024 => {
                data.extend(chunk)
            }
            Some(ChannelMsg::ExitStatus { exit_status }) => success = exit_status == 0,
            Some(ChannelMsg::Close) | None => break,
            Some(
                ChannelMsg::ExtendedData { .. }
                | ChannelMsg::Eof
                | ChannelMsg::WindowAdjusted { .. },
            ) => (),
            _ => return Err(MobileError::TerminalLookup),
        }
    }
    let json: serde_json::Value =
        serde_json::from_slice(&data).map_err(|_| MobileError::TerminalLookup)?;
    let info = &json["result"]["pane"];
    if !success || info["pane_id"].as_str() != Some(pane) {
        return Err(MobileError::TerminalLookup);
    }
    info["terminal_id"]
        .as_str()
        .filter(|id| valid_id(id))
        .map(str::to_owned)
        .ok_or(MobileError::TerminalLookup)
}

async fn allocate_pty(ch: &mut SshChannel, (cols, rows): (u16, u16)) -> Result<(), MobileError> {
    ch.request_pty(true, "xterm-256color", cols.into(), rows.into(), 0, 0, &[])
        .await
        .map_err(|_| MobileError::TerminalUnavailable)?;
    ack(ch).await
}

pub(super) async fn run(
    request: &mut Request,
    opener: &Opener,
    path: &str,
    session: &str,
) -> Result<(), MobileError> {
    if *request.stop.borrow() {
        return Ok(());
    }
    request.binding.current()?;
    let herdr = format!(
        "exec {} --session {}",
        shell_quote(path),
        shell_quote(session)
    );
    let mut query = opener.open(&mut request.stop).await?;
    let mut terminal_channel = None;
    let mut open_stop = request.stop.clone();
    let reserve = async {
        match opener.open(&mut open_stop).await {
            Ok(channel) => allocate_pty(terminal_channel.insert(channel), request.size).await,
            // Some hosts allow only the bridge plus one SSH session channel.
            Err(MobileError::TerminalUnavailable) => Ok(()),
            Err(error) => Err(error),
        }
    };
    let preparation = async {
        tokio::try_join!(
            lookup(&mut query, &herdr, &request.binding.target.pane_id),
            reserve
        )
    };
    let prepared = tokio::select! {
        biased;
        _ = stopped(&mut request.stop) => Err(MobileError::TerminalEnded),
        result = timeout(SETUP, preparation) => result.unwrap_or(Err(MobileError::TerminalTimeout)),
    };
    let _ = timeout(SETUP, query.close()).await;
    let setup = async {
        let (terminal, ()) = prepared?;
        let ch = match &mut terminal_channel {
            Some(ch) => ch,
            None => {
                // The lookup has released its slot. Retry only the channel open.
                let ch = terminal_channel.insert(opener.open(&mut open_stop).await?);
                allocate_pty(ch, request.size).await?;
                ch
            }
        };
        request.binding.current()?;
        ch.exec(
            true,
            format!("{herdr} terminal attach {}", shell_quote(&terminal)),
        )
        .await
        .map_err(|_| MobileError::TerminalUnavailable)?;
        ack(ch).await
    };
    let result = tokio::select! {
        biased;
        _ = stopped(&mut request.stop) => Err(MobileError::TerminalEnded),
        result = timeout(SETUP, setup) => result.unwrap_or(Err(MobileError::TerminalTimeout)),
    };
    let Some(ch) = terminal_channel else {
        return result;
    };
    let (mut reader, writer) = ch.split();
    let result = match result {
        Err(error) => Err(error),
        Ok(()) => {
            let receive = async {
                while let Some(message) = reader.wait().await {
                    match message {
                        ChannelMsg::Data { data } => {
                            for chunk in data.chunks(PACKET) {
                                request.binding.ready.store(true, Ordering::Release);
                                request
                                    .output
                                    .send(chunk.to_vec())
                                    .await
                                    .map_err(|_| MobileError::TerminalEnded)?;
                                request.binding.wake_reader();
                            }
                        }
                        ChannelMsg::ExitStatus { exit_status } if exit_status != 0 => {
                            return Err(MobileError::TerminalUnavailable);
                        }
                        ChannelMsg::ExitSignal { .. } => return Err(MobileError::TerminalEnded),
                        ChannelMsg::Close => break,
                        _ => (),
                    }
                }
                Ok(())
            };
            let send = async {
                while let Some(command) = request.input.recv().await {
                    request.binding.current()?;
                    match command {
                        Command::Write(data) => writer.data_bytes(data).await,
                        Command::Resize(cols, rows) => {
                            writer.window_change(cols.into(), rows.into(), 0, 0).await
                        }
                    }
                    .map_err(|_| MobileError::TerminalEnded)?;
                }
                Ok(())
            };
            tokio::select! {
                biased;
                _ = stopped(&mut request.stop) => Ok(()),
                result = receive => result,
                result = send => result,
            }
        }
    };
    let _ = timeout(SETUP, writer.close()).await;
    result
}

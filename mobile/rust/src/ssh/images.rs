// ABOUTME: Stages a terminal draft attachment privately on the existing SSH host.
// ABOUTME: Confirms retention before paste and discards staged files when a target changes.
use super::transport::{ConnectedSsh, exec};
use crate::MobileError;
use russh::ChannelMsg;
use spaceport_client::staging::{DISCARD, RETAIN, RETAINED, is_staged_file, retain_script};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    sync::{Semaphore, oneshot, watch},
    time::timeout,
};

const LIMIT: Duration = Duration::from_secs(45);
const UNAVAILABLE: MobileError = MobileError::ImageTransfer;

/// The name the staged file keeps inside its staging directory.
const NAME: &str = "screenshot.png";

fn command(name: &str, size: usize) -> String {
    let script = retain_script(name, size as u64);
    format!("exec /bin/sh -c {}", spaceport_client::shell_quote(&script))
}

async fn read_line(channel: &mut russh::Channel<russh::client::Msg>) -> Option<String> {
    let mut line = Vec::new();
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { data }
                if line.len() + data.len() <= spaceport_client::staging::LINE_LIMIT =>
            {
                line.extend(data);
                if line.last() == Some(&b'\n') {
                    line.pop();
                    return String::from_utf8(line).ok();
                }
            }
            ChannelMsg::WindowAdjusted { .. } => (),
            _ => break,
        }
    }
    None
}

fn valid_filename(name: &str) -> bool {
    !matches!(name, "" | "." | "..")
        && name.len() <= 255
        && !name.contains('/')
        && !name.chars().any(char::is_control)
}

#[derive(uniffi::Object)]
pub struct MobileFileTransfer {
    cancelled: watch::Sender<bool>,
    bytes: AtomicU64,
}

#[uniffi::export]
impl MobileFileTransfer {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            cancelled: watch::channel(false).0,
            bytes: AtomicU64::new(0),
        })
    }
    pub fn cancel(&self) {
        self.cancelled.send_replace(true);
    }
    pub fn bytes_sent(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }
}

#[derive(uniffi::Object)]
pub struct StagedImage {
    path: String,
    keep: Mutex<Option<oneshot::Sender<()>>>,
}

#[uniffi::export]
impl StagedImage {
    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn paste_path(&self) -> String {
        if self
            .path
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"/_-.+".contains(&c))
        {
            self.path.clone()
        } else {
            spaceport_client::shell_quote(&self.path)
        }
    }

    /// Call only after the terminal's original input route accepts the path.
    pub fn keep(&self) {
        if let Some(keep) = self.keep.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = keep.send(());
        }
    }
}

pub(crate) struct ImageStager {
    slot: Arc<Semaphore>,
}

impl Default for ImageStager {
    fn default() -> Self {
        Self {
            slot: Arc::new(Semaphore::new(1)),
        }
    }
}

impl ImageStager {
    pub(crate) fn stage(
        &self,
        transport: &ConnectedSsh,
        png: Vec<u8>,
        transfer: Arc<MobileFileTransfer>,
    ) -> Result<Arc<StagedImage>, MobileError> {
        if !png.starts_with(b"\x89PNG\r\n\x1a\n")
            || png.len() > spaceport_client::protocol::MAX_CLIPBOARD_IMAGE_PAYLOAD
        {
            return Err(UNAVAILABLE);
        }
        self.stage_file(transport, NAME.to_owned(), png, transfer)
    }

    pub(crate) fn stage_file(
        &self,
        transport: &ConnectedSsh,
        name: String,
        bytes: Vec<u8>,
        transfer: Arc<MobileFileTransfer>,
    ) -> Result<Arc<StagedImage>, MobileError> {
        if !valid_filename(&name)
            || bytes.len() > spaceport_client::protocol::MAX_CLIPBOARD_IMAGE_PAYLOAD
        {
            return Err(UNAVAILABLE);
        }
        let mut cancelled = transfer.cancelled.subscribe();
        if *cancelled.borrow_and_update() {
            return Err(MobileError::FileCancelled);
        }
        let (runtime, session) = transport.get()?;
        let permit = self
            .slot
            .clone()
            .try_acquire_owned()
            .map_err(|_| UNAVAILABLE)?;
        let (result, response) = oneshot::channel();
        runtime.spawn(async move {
            let _permit = permit;
            let open = tokio::select! {
                _ = cancelled.changed() => { let _ = result.send(Err(MobileError::FileCancelled)); return; }
                open = timeout(LIMIT, exec(&session, command(&name, bytes.len()))) => open,
            };
            let Ok(Ok(mut channel)) = open else {
                let _ = result.send(Err(UNAVAILABLE));
                return;
            };
            let stage = async {
                for chunk in bytes.chunks(64 * 1024) {
                    channel.data_bytes(chunk.to_vec()).await.ok()?;
                    transfer.bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                }
                let path = read_line(&mut channel).await?;
                is_staged_file(&path, &name).then_some(path)
            };
            let uploaded = tokio::select! {
                biased;
                _ = cancelled.changed() => None,
                r = timeout(LIMIT, stage) => r.ok().flatten(),
            };
            let prepared = match uploaded {
                Some(path) => {
                    // If the SSH connection closes after this acknowledgement the path survives.
                    let retain = async {
                        channel
                            .data_bytes(format!("{RETAIN}\n").into_bytes())
                            .await
                            .ok()?;
                        read_line(&mut channel).await
                    };
                    match timeout(LIMIT, retain).await {
                        Ok(Some(ack)) if ack == RETAINED => Ok(path),
                        _ => Err(MobileError::ImageRetentionUnconfirmed),
                    }
                }
                None if *cancelled.borrow() => Err(MobileError::FileCancelled),
                None => Err(UNAVAILABLE),
            };
            let accepted = match prepared {
                Ok(_) if *cancelled.borrow() => { let _ = result.send(Err(MobileError::FileCancelled)); false }
                Ok(path) => {
                    let (keep, decision) = oneshot::channel();
                    let image = Arc::new(StagedImage {
                        path,
                        keep: Mutex::new(Some(keep)),
                    });
                    result.send(Ok(image)).is_ok() && decision.await.is_ok()
                }
                Err(error) => {
                    let _ = result.send(Err(error));
                    false
                }
            };
            if !accepted {
                let discard = format!("{DISCARD}\n").into_bytes();
                let _ = timeout(LIMIT, channel.data_bytes(discard)).await;
            }
            // Let the shell keep or clean up before closing the SSH channel.
            let _ = timeout(LIMIT, async {
                channel.eof().await?;
                while let Some(message) = channel.wait().await {
                    if matches!(message, ChannelMsg::Close | ChannelMsg::ExitStatus { .. }) {
                        break;
                    }
                }
                channel.close().await
            })
            .await;
        });
        // The entire SSH runtime can stop before its worker reports which phase failed.
        response
            .blocking_recv()
            .map_err(|_| MobileError::ImageRetentionUnconfirmed)?
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod file_tests;

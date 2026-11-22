// ABOUTME: Stages an iPhone screenshot privately on the existing SSH host.
// ABOUTME: Confirms retention before paste and discards staged files when a target changes.
use super::transport::{HostKey, exec};
use crate::MobileError;
use russh::{ChannelMsg, client};
use std::{
    sync::{Arc, Mutex, OnceLock, Weak},
    time::Duration,
};
use tokio::{
    runtime::Handle,
    sync::{Semaphore, oneshot},
    time::timeout,
};

const LIMIT: Duration = Duration::from_secs(45);
const UNAVAILABLE: MobileError = MobileError::ImageTransfer;

fn command(size: usize) -> String {
    // The acknowledgement arrives only after head exits. No read-ahead can consume it.
    let script = format!(
        r#"set -eu
umask 077
dir=$(mktemp -d /tmp/herdr-upload.XXXXXXXXXXXX)
file="$dir/screenshot.png"
trap 'rm -f "$file"; rmdir "$dir"' EXIT
trap 'exit 1' HUP INT TERM
head -c {size} > "$file"
[ "$(wc -c < "$file")" -eq {size} ]
printf '%s\n' "$file"
IFS= read -r decision
[ "$decision" = retain ]
trap - EXIT
printf 'retained\n'
if IFS= read -r decision && [ "$decision" = discard ]; then
    rm -f "$file"
    rmdir "$dir"
fi
"#
    );
    format!("exec /bin/sh -c {}", spaceport_client::shell_quote(&script))
}

async fn read_line(channel: &mut russh::Channel<client::Msg>) -> Option<String> {
    let mut line = Vec::new();
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { data } if line.len() + data.len() <= 128 => {
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

fn valid_path(path: &str) -> bool {
    path.strip_prefix("/tmp/herdr-upload.")
        .and_then(|s| s.strip_suffix("/screenshot.png"))
        .is_some_and(|s| s.len() == 12 && s.bytes().all(|b| b.is_ascii_alphanumeric()))
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

    /// Call only after the terminal's original input route accepts the path.
    pub fn keep(&self) {
        if let Some(keep) = self.keep.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = keep.send(());
        }
    }
}

pub(crate) struct ImageStager {
    session: OnceLock<(Handle, Weak<client::Handle<HostKey>>)>,
    slot: Arc<Semaphore>,
}

impl Default for ImageStager {
    fn default() -> Self {
        Self {
            session: OnceLock::new(),
            slot: Arc::new(Semaphore::new(1)),
        }
    }
}

impl ImageStager {
    pub(super) fn connect(&self, session: &Arc<client::Handle<HostKey>>) {
        let _ = self
            .session
            .set((Handle::current(), Arc::downgrade(session)));
    }

    pub(crate) fn stage(&self, png: Vec<u8>) -> Result<Arc<StagedImage>, MobileError> {
        if !png.starts_with(b"\x89PNG\r\n\x1a\n")
            || png.len() > spaceport_client::protocol::MAX_CLIPBOARD_IMAGE_PAYLOAD
        {
            return Err(UNAVAILABLE);
        }
        let (runtime, weak) = self.session.get().ok_or(MobileError::Disconnected)?;
        let session = weak.upgrade().ok_or(MobileError::Disconnected)?;
        let permit = self
            .slot
            .clone()
            .try_acquire_owned()
            .map_err(|_| UNAVAILABLE)?;
        let (result, response) = oneshot::channel();
        runtime.spawn(async move {
            let _permit = permit;
            let open = timeout(LIMIT, exec(&session, command(png.len()))).await;
            let Ok(Ok(mut channel)) = open else {
                let _ = result.send(Err(UNAVAILABLE));
                return;
            };
            let stage = async {
                channel.data_bytes(png).await.ok()?;
                let path = read_line(&mut channel).await?;
                valid_path(&path).then_some(path)
            };
            let prepared = match timeout(LIMIT, stage).await {
                Ok(Some(path)) => {
                    // If the SSH connection closes after this acknowledgement the path survives.
                    let retain = async {
                        channel.data_bytes(b"retain\n".as_slice()).await.ok()?;
                        read_line(&mut channel).await
                    };
                    match timeout(LIMIT, retain).await {
                        Ok(Some(ack)) if ack == "retained" => Ok(path),
                        _ => Err(MobileError::ImageRetentionUnconfirmed),
                    }
                }
                _ => Err(UNAVAILABLE),
            };
            let accepted = match prepared {
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
                let _ = timeout(LIMIT, channel.data_bytes(b"discard\n".as_slice())).await;
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

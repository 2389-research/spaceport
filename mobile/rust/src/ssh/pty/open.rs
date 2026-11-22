// ABOUTME: Opens cancellable SSH channels on the current authenticated session.
// ABOUTME: Reaps late confirmations while bounding unresolved requests across rapid switches.
use super::{HostKey, MobileError, client};
use russh::Channel;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, oneshot, watch};

pub(super) type SshChannel = Channel<client::Msg>;

pub(super) struct Opener {
    handle: Arc<client::Handle<HostKey>>,
    slots: Arc<Semaphore>,
}

impl Opener {
    pub(super) fn new(handle: Arc<client::Handle<HostKey>>) -> Self {
        Self {
            handle,
            slots: Arc::new(Semaphore::new(2)),
        }
    }

    pub(super) async fn open(
        &self,
        stop: &mut watch::Receiver<bool>,
    ) -> Result<SshChannel, MobileError> {
        let acquire = async {
            let permit = self
                .slots
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| MobileError::TerminalEnded)?;
            let (deliver, reply) = oneshot::channel();
            let handle = self.handle.clone();
            tokio::spawn(async move {
                // SSH has no channel ID we can close until the server answers the open.
                let result = handle
                    .channel_open_session()
                    .await
                    .map(|channel| Offer(Some((channel, permit))))
                    .map_err(|_| MobileError::TerminalUnavailable);
                let _ = deliver.send(result);
            });
            let mut offer = reply.await.map_err(|_| MobileError::TerminalEnded)??;
            offer
                .0
                .take()
                .map(|(channel, _)| channel)
                .ok_or(MobileError::TerminalEnded)
        };
        tokio::select! {
            biased;
            _ = super::channel::stopped(stop) => Err(MobileError::TerminalEnded),
            result = tokio::time::timeout(super::channel::SETUP, acquire) => result.unwrap_or(Err(MobileError::TerminalTimeout)),
        }
    }
}

/// If the recipient cancelled, close the confirmed channel before releasing its slot.
struct Offer(Option<(SshChannel, OwnedSemaphorePermit)>);

impl Drop for Offer {
    fn drop(&mut self) {
        if let Some((channel, permit)) = self.0.take() {
            tokio::spawn(async move {
                let _ = channel.close().await;
                drop(permit);
            });
        }
    }
}

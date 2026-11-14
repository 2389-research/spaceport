// ABOUTME: Owns Ask helper processes and checkpoint files off the UI thread.
// ABOUTME: Archives conversations by identity and reaps only its own helper child.
use super::error::{Error, Result};
use sheprd_chat_protocol::{
    EndpointId,
    ask::{self, Checkpoint, Outcome},
};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

const LAUNCH_TIMEOUT: Duration = Duration::from_secs(10);
// Covers the helper's three-second cancel deadline and two-second Pi reaping grace.
const HELPER_STOP_TIMEOUT: Duration = Duration::from_secs(7);
// A queued launch may consume its full startup budget before shutdown begins.
const FINISH_TIMEOUT: Duration =
    Duration::from_secs(LAUNCH_TIMEOUT.as_secs() + HELPER_STOP_TIMEOUT.as_secs() + 1);

pub(super) enum Command {
    Launch,
    Stop,
    New,
    Quit,
}
pub(super) enum Event {
    Restored(Option<Box<Checkpoint>>),
    Ready(PathBuf),
    StopSaved,
    Failed(Error),
}

pub(super) struct Worker {
    pub(super) commands: SyncSender<Command>,
    pub(super) events: Receiver<Event>,
    finished: Receiver<()>,
}

impl Worker {
    pub(super) fn start(root: PathBuf, broker: PathBuf) -> Result<Self> {
        Self::from_owner(Owner {
            root,
            broker,
            conversation: None,
            child: None,
            diagnostics: None,
        })
    }

    fn from_owner(mut owner: Owner) -> Result<Self> {
        let (commands, receiver) = mpsc::sync_channel(4);

        let (sender, events) = mpsc::sync_channel(4);
        let (finish, finished) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("spaceport-ask".into())
            .spawn(move || {
                match owner.restore() {
                    Ok(checkpoint) => {
                        let _ = sender.send(Event::Restored(checkpoint.map(Box::new)));
                    }
                    Err(error) => {
                        let _ = sender.send(Event::Failed(error));
                    }
                }
                while let Ok(command) = receiver.recv() {
                    let result = match command {
                        Command::Launch => owner.launch().map(Event::Ready),
                        Command::Stop => owner.stop_intent().map(|()| Event::StopSaved),
                        Command::New => owner
                            .stop()
                            .and_then(|()| owner.new_chat())
                            .map(|()| Event::Restored(None)),
                        Command::Quit => {
                            let _ = owner.stop();
                            break;
                        }
                    };
                    let event = result.unwrap_or_else(Event::Failed);
                    if sender.send(event).is_err() {
                        break;
                    }
                }
                let _ = owner.stop();
                let _ = finish.send(());
            })?;
        Ok(Self {
            commands,
            events,
            finished,
        })
    }

    pub(super) fn finish(self) {
        let Self {
            commands,
            events,
            finished,
        } = self;
        drop(events);
        let _ = commands.try_send(Command::Quit);
        drop(commands);
        let _ = finished.recv_timeout(FINISH_TIMEOUT);
    }
}

struct Owner {
    root: PathBuf,
    broker: PathBuf,
    conversation: Option<Uuid>,
    child: Option<Child>,
    diagnostics: Option<thread::JoinHandle<Vec<u8>>>,
}

impl Owner {
    fn directory(&self) -> Result<PathBuf> {
        self.conversation
            .map(|id| self.root.join(id.to_string()))
            .ok_or(Error::InvalidState)
    }

    fn restore(&mut self) -> Result<Option<Checkpoint>> {
        let pointer = match fs::read_to_string(self.root.join("current")) {
            Ok(pointer) => pointer,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        self.conversation = Some(Uuid::parse_str(pointer.trim()).map_err(|_| Error::InvalidState)?);
        let mut checkpoint = read_checkpoint(&self.directory()?)?;
        if let Some(checkpoint) = &checkpoint
            && Some(checkpoint.conversation_id) != self.conversation
        {
            return Err(Error::InvalidState);
        }
        if self.directory()?.join(ask::STOP_FILE).try_exists()?
            && let Some(saved) = &mut checkpoint
        {
            if let Some(request) = &mut saved.request {
                request.phase = ask::Phase::Stopped;
            }
            if saved.outcome != Outcome::Unknown {
                saved.outcome = Outcome::Stopped;
                saved.transcript.status = sheprd_chat_protocol::Status::Idle;
                saved.transcript.turn = None;
            }
        }
        Ok(checkpoint)
    }

    fn stop_intent(&self) -> Result<()> {
        let directory = self.directory()?;
        if directory.join(ask::STOP_FILE).try_exists()? {
            return Ok(());
        }
        fs::write(directory.join("stop.next"), b"stop")?;
        fs::rename(directory.join("stop.next"), directory.join(ask::STOP_FILE))?;
        Ok(())
    }

    fn new_chat(&mut self) -> Result<()> {
        let id = Uuid::new_v4();
        fs::create_dir_all(self.root.join(id.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.root, fs::Permissions::from_mode(0o700))?;
            fs::set_permissions(
                self.root.join(id.to_string()),
                fs::Permissions::from_mode(0o700),
            )?;
        }
        fs::write(self.root.join("current.next"), id.to_string())?;
        fs::rename(self.root.join("current.next"), self.root.join("current"))?;
        self.conversation = Some(id);
        Ok(())
    }

    fn launch(&mut self) -> Result<PathBuf> {
        if self.conversation.is_none() {
            self.new_chat()?;
        }
        let directory = self.directory()?;
        let previous = read_checkpoint(&directory)?;
        let endpoint = match &previous {
            Some(saved) => saved
                .endpoint
                .parse::<EndpointId>()
                .map_err(|_| Error::InvalidState)?,
            None => EndpointId::random(),
        };
        let address = sheprd_chat_protocol::endpoint_address(&endpoint).ok_or(Error::NoState)?;
        if let Some(child) = &mut self.child {
            if child.try_wait()?.is_none() {
                return Ok(address);
            }
            self.child = None;
        }
        if previous
            .as_ref()
            .is_some_and(|saved| matches!(saved.outcome, Outcome::Unknown | Outcome::Answering))
        {
            return Err(Error::InvalidState);
        }
        let helper = crate::chat::start::locate().ok_or(Error::MissingHelper)?;
        let _ = fs::remove_file(directory.join(ask::QUIT_FILE));
        let id = self.conversation.ok_or(Error::InvalidState)?;
        self.child = Some(
            sheprd_client::new_command(helper)
                .arg("ask")
                .arg("--state-dir")
                .arg(&directory)
                .arg("--endpoint")
                .arg(endpoint.as_str())
                .arg("--broker")
                .arg(&self.broker)
                .arg("--conversation")
                .arg(id.to_string())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()?,
        );
        let stderr = self
            .child
            .as_mut()
            .and_then(|child| child.stderr.take())
            .ok_or(Error::NotReady)?;
        match thread::Builder::new()
            .name("spaceport-ask-stderr".into())
            .spawn(move || bounded_diagnostic(stderr))
        {
            Ok(reader) => self.diagnostics = Some(reader),
            Err(error) => {
                self.stop()?;
                return Err(error.into());
            }
        }
        let deadline = Instant::now() + LAUNCH_TIMEOUT;
        while Instant::now() < deadline {
            if let Some(saved) = read_checkpoint(&directory)?
                && previous
                    .as_ref()
                    .is_none_or(|old| old.helper_run_id != saved.helper_run_id)
            {
                return Ok(address);
            }
            if let Some(child) = &mut self.child
                && let Some(status) = child.try_wait()?
            {
                self.child = None;
                let bytes = self
                    .diagnostics
                    .take()
                    .and_then(|reader| reader.join().ok())
                    .unwrap_or_default();
                let diagnostic =
                    crate::notifications::safe_text(&String::from_utf8_lossy(&bytes), 2_000);
                return Err(Error::Helper { status, diagnostic });
            }
            thread::sleep(Duration::from_millis(25));
        }
        self.stop()?;
        Err(Error::NotReady)
    }

    fn stop(&mut self) -> Result<()> {
        let Some(mut child) = self.child.take() else {
            return Ok(());
        };
        let request = self
            .directory()
            .and_then(|directory| Ok(fs::write(directory.join(ask::QUIT_FILE), b"quit")?));
        let deadline = Instant::now() + HELPER_STOP_TIMEOUT;
        while Instant::now() < deadline {
            if child.try_wait()?.is_some() {
                return request;
            }
            thread::sleep(Duration::from_millis(25));
        }
        let _ = child.kill();
        let _ = child.wait();
        request
    }
}

fn bounded_diagnostic(mut reader: impl Read) -> Vec<u8> {
    let mut kept = Vec::new();
    let mut buffer = [0; 1024];
    while let Ok(count) = reader.read(&mut buffer) {
        if count == 0 {
            break;
        }
        let retain = count.min(4096usize.saturating_sub(kept.len()));
        kept.extend_from_slice(&buffer[..retain]);
    }
    kept
}

pub(super) fn read_checkpoint(directory: &Path) -> Result<Option<Checkpoint>> {
    let file = match fs::File::open(directory.join(ask::CHECKPOINT_FILE)) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.take(ask::MAX_CHECKPOINT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > ask::MAX_CHECKPOINT_BYTES {
        return Err(Error::InvalidState);
    }
    let checkpoint: Checkpoint = serde_json::from_slice(&bytes)?;
    if checkpoint.version != ask::VERSION {
        return Err(Error::InvalidState);
    }
    Ok(Some(checkpoint))
}

#[cfg(test)]
mod tests;

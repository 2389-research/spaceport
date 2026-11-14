// ABOUTME: Stores bounded private Ask checkpoints with one helper owner.
// ABOUTME: Replaces complete files atomically and preserves the previous file on failure.
use super::error::Error;
use sheprd_chat_protocol::ask::{CHECKPOINT_FILE, Checkpoint, MAX_CHECKPOINT_BYTES};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub(super) struct Store {
    dir: PathBuf,
    _lock: File,
}
impl Store {
    pub(super) fn open(dir: &Path) -> Result<Self, Error> {
        if !dir.is_absolute() {
            return Err(Error::InvalidState);
        }
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(dir)?;
        let metadata = fs::symlink_metadata(dir)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(Error::InvalidState);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            if metadata.uid() != rustix::process::geteuid().as_raw() {
                return Err(Error::InvalidState);
            }
            fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
        }
        let lock_path = dir.join("owner.lock");
        if fs::symlink_metadata(&lock_path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(Error::InvalidState);
        }
        let lock = private_options()
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        lock.try_lock().map_err(|_| Error::Owned)?;
        Ok(Self {
            dir: dir.canonicalize()?,
            _lock: lock,
        })
    }
    pub(super) fn save(&self, state: &Checkpoint) -> Result<(), Error> {
        let bytes = serde_json::to_vec(state)?;
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Error::TooLarge);
        }
        self.replace(CHECKPOINT_FILE, &bytes)
    }
    pub(super) fn replace(&self, name: &str, bytes: &[u8]) -> Result<(), Error> {
        let temporary = self.dir.join(format!(".{name}-{}", uuid::Uuid::new_v4()));
        let result = (|| {
            let mut file = private_options().create_new(true).open(&temporary)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, self.dir.join(name))?;
            #[cfg(unix)]
            File::open(&self.dir)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }
    pub(super) fn load(&self) -> Result<Option<Checkpoint>, Error> {
        let path = self.dir.join(CHECKPOINT_FILE);
        if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(Error::InvalidState);
        }
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let mut bytes = Vec::new();
        file.take((MAX_CHECKPOINT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Error::TooLarge);
        }
        Ok(Some(serde_json::from_slice(&bytes)?))
    }
}
fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

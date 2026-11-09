//! Opens a chat's HTML fence or tool-written HTML file in the system browser
//! after an explicit click. A copied file lives alone in a private folder, so
//! its script cannot read siblings of the original file.

mod error;

pub(crate) use error::Error;

use crate::{
    SheprdWindow,
    i18n::{Localize, fl},
    notifications::safe_text,
};
use gpui::{AppContext as _, Context, Window};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    rc::Rc,
};

/// What to show.
#[derive(Clone, Debug)]
pub(crate) enum PageSource {
    /// An HTML fence's source.
    Html(Rc<str>),
    /// An HTML file on the chat's machine.
    File(PathBuf),
}

impl SheprdWindow {
    /// Shows `source` in the message at `key` of the chat in `pane_id`, or
    /// in the system browser where pages cannot show in the window.
    pub(crate) fn show_chat_page(
        &mut self,
        pane_id: &str,
        source: PageSource,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pane) = self.chats.pane_mut(pane_id) else {
            return;
        };
        let copied = match source {
            PageSource::File(path) => {
                if let super::Route::Ssh(host) = &pane.target.route {
                    pane.error = Some(fl!("chat-page-remote", host = safe_text(host, 80)));
                    cx.notify();
                    return;
                }
                cx.background_spawn(async move {
                    let html = read_page(&path)?;
                    write_page(&html)
                })
            }
            PageSource::Html(html) => {
                let html = html.as_bytes().to_vec();
                cx.background_spawn(async move { write_page(&html) })
            }
        };
        let pane_id = pane_id.to_owned();
        cx.spawn_in(window, async move |this, cx| {
            let copied = copied.await;
            let _ = this.update_in(cx, |this, window, cx| match copied {
                Ok(path) => this.open_chat_file(&pane_id, path, window, cx),
                Err(error) => this.chat_page_failed(&pane_id, error.localize(), cx),
            });
        })
        .detach();
    }

    fn chat_page_failed(&mut self, pane_id: &str, error: String, cx: &mut Context<Self>) {
        if let Some(pane) = self.chats.pane_mut(pane_id) {
            pane.error = Some(fl!("chat-page-failed", error = error));
        }
        cx.notify();
    }

    /// The system browser shows the exact copied file after the user clicks.
    fn open_chat_file(
        &mut self,
        pane_id: &str,
        path: PathBuf,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match url::Url::from_file_path(&path) {
            Ok(url) => {
                cx.open_url(url.as_str());
                self.show_flash(crate::window::Flash::success(fl!("chat-page-opened")), cx);
            }
            Err(()) => self.chat_page_failed(pane_id, Error::NotFile.localize(), cx),
        }
    }
}

/// Bytes of an HTML file shown; a larger one is refused.
const MAX_FILE: u64 = 4 * 1024 * 1024;

/// The HTML file at `path`, when it is a bounded regular file.
fn read_page(path: &Path) -> Result<Vec<u8>, Error> {
    use std::io::Read as _;
    let mut html = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut html)?;
    if html.len() as u64 > MAX_FILE {
        return Err(Error::TooLarge);
    }
    Ok(html)
}

/// Writes a page to `index.html` in a folder of its own, named by what it
/// holds, and returns the file. Blocks, so it runs off the UI thread.
fn write_page(html: &[u8]) -> Result<PathBuf, Error> {
    let root = private_folder("pages").ok_or(Error::Folder)?;
    let mut hasher = DefaultHasher::new();
    html.hash(&mut hasher);
    let folder = root.join(format!("{:016x}", hasher.finish()));
    private_dir(&root)?;
    private_dir(&folder)?;
    let file = folder.join("index.html");
    std::fs::write(&file, html)?;
    Ok(file)
}

/// The per-user folder for chat files of one `kind`, such as pages. A local
/// page may not be served from a folder with a hidden name in its path, so
/// on macOS and Windows it is the user's own temporary folder.
#[cfg(any(target_os = "macos", windows))]
pub(super) fn private_folder(kind: &str) -> Option<PathBuf> {
    Some(std::env::temp_dir().join(format!("herdr-chat-{kind}")))
}

/// Elsewhere `/tmp` is shared with other users, so the page goes in the
/// user's runtime or cache folder.
#[cfg(not(any(target_os = "macos", windows)))]
pub(super) fn private_folder(kind: &str) -> Option<PathBuf> {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .or_else(|| std::env::var_os("XDG_CACHE_HOME"))
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;
    base.is_absolute()
        .then(|| base.join(format!("herdr-gpui-chat-{kind}")))
}

/// Creates `path`, readable by this user alone where the platform says so.
pub(super) fn private_dir(path: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(path)
}

#[cfg(test)]
mod tests;

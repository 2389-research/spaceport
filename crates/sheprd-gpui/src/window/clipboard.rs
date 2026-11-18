// ABOUTME: Bounds native clipboard acquisition outside the UI thread.
// ABOUTME: Selects normal, text-only, and image-only acquisition without copying unused formats.
//! Bounded clipboard acquisition, including image copies and hashing, off the UI thread.
// i18n: migrated

mod error;
mod pending;

pub(crate) use pending::{reserve, start, start_with};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    // keep-sorted start
    Image,
    Normal,
    Text,
    // keep-sorted end
}

pub(crate) use error::Error;

use crate::Result;
use gpui::ClipboardItem;
#[cfg(any(target_os = "macos", target_os = "linux", windows, test))]
use gpui::{Image, ImageFormat};

#[cfg(any(target_os = "macos", target_os = "linux", windows, test))]
const TEXT_LIMIT: usize = 2 * 1024 * 1024;
#[cfg(any(target_os = "macos", target_os = "linux", windows, test))]
const IMAGE_LIMIT: usize = super::image_source::MAX_IMAGE_INPUT_BYTES;

/// Blocking: invoke only on a background executor, never through an App context.
/// Normal paste reads copied files first, then text, and never fetches an image
/// when either is present. A Finder file copy also publishes the file name as
/// text and a preview image; its file URLs are returned as paths, never opened.
/// Image-only paste ignores files and text. No other URLs or metadata are read.
/// Text probes ignore files and images, even when no text is available.
pub(crate) fn read(mode: Mode) -> Result<Option<ClipboardItem>> {
    #[cfg(target_os = "macos")]
    {
        macos::read(mode)
    }
    #[cfg(target_os = "linux")]
    {
        linux::read(mode)
    }
    #[cfg(windows)]
    {
        windows::read(mode)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
    {
        let _ = mode;
        Err(Error::Unsupported.into())
    }
}

/// Native acquisition in the helper process (or a private test pasteboard).
/// Call inside an autorelease pool; the GUI must not call this on a worker.
#[cfg(target_os = "macos")]
fn read_pasteboard(
    pasteboard: &objc2_app_kit::NSPasteboard,
    mode: Mode,
) -> Result<Option<ClipboardItem>> {
    use gpui::{ClipboardEntry, ExternalPaths};
    use objc2_foundation::NSString;

    let revision = pasteboard.changeCount();
    let files = if mode == Mode::Normal {
        file_paths(pasteboard)
    } else {
        Vec::new()
    };
    let result = if !files.is_empty() {
        Ok(Some(ClipboardItem {
            entries: vec![ClipboardEntry::ExternalPaths(ExternalPaths(files.into()))],
        }))
    } else {
        read_with(mode, |format, limit| {
            let kind = NSString::from_str(match format {
                None => "public.utf8-plain-text",
                Some(ImageFormat::Png) => "public.png",
                Some(ImageFormat::Jpeg) => "public.jpeg",
                Some(ImageFormat::Gif) => "com.compuserve.gif",
                Some(ImageFormat::Webp) => "org.webmproject.webp",
                Some(ImageFormat::Bmp) => "com.microsoft.bmp",
                Some(ImageFormat::Tiff) => "public.tiff",
                Some(ImageFormat::Svg | ImageFormat::Ico | ImageFormat::Pnm) => {
                    return Err(crate::Error::ImageFormat);
                }
            });
            let Some(data) = pasteboard.dataForType(&kind) else {
                return Ok(None);
            };
            // AppKit materializes NSData before exposing its length. Bound the
            // additional Rust allocation before copying, decoding, or hashing.
            if data.len() > limit {
                return Err(Error::Size { limit }.into());
            }
            Ok(Some(data.to_vec()))
        })
    };
    if pasteboard.changeCount() != revision {
        return Err(Error::Changed.into());
    }
    result
}

/// The local paths of the pasteboard's file URLs, one per pasteboard item.
/// Finder may publish file reference URLs (`file:///.file/id=...`); they are
/// resolved to their current path, and those that no longer resolve are skipped.
/// Collection stops at one more path than a drop accepts, counted after
/// unresolved URLs are skipped, so an oversized copy still fails the drop's
/// own limit instead of pasting a list with files silently missing.
#[cfg(target_os = "macos")]
fn file_paths(pasteboard: &objc2_app_kit::NSPasteboard) -> Vec<std::path::PathBuf> {
    use objc2_foundation::{NSString, NSURL};

    let Some(items) = pasteboard.pasteboardItems() else {
        return Vec::new();
    };
    let kind = NSString::from_str("public.file-url");
    items
        .iter()
        .filter_map(|item| {
            let url = NSURL::URLWithString(&*item.stringForType(&kind)?)?;
            url.filePathURL()?.to_file_path()
        })
        .take(super::file_drop::MAX_PATHS + 1)
        .collect()
}

#[cfg(any(target_os = "macos", target_os = "linux", windows, test))]
fn read_with(
    mode: Mode,
    mut acquire: impl FnMut(Option<ImageFormat>, usize) -> Result<Option<Vec<u8>>>,
) -> Result<Option<ClipboardItem>> {
    if mode != Mode::Image
        && let Some(bytes) = acquire(None, TEXT_LIMIT)?
    {
        if bytes.len() > TEXT_LIMIT {
            return Err(Error::Size { limit: TEXT_LIMIT }.into());
        }
        let text = String::from_utf8(bytes).map_err(|error| Error::Encoding(error.utf8_error()))?;
        return Ok(Some(ClipboardItem::new_string(text)));
    }
    if mode == Mode::Text {
        return Ok(None);
    }
    // TIFF is last: AppKit apps often publish it beside a PNG that needs no
    // conversion, while Preview and some browsers publish only TIFF.
    for format in [
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Gif,
        ImageFormat::Webp,
        ImageFormat::Bmp,
        ImageFormat::Tiff,
    ] {
        if let Some(bytes) = acquire(Some(format), IMAGE_LIMIT)? {
            if bytes.len() > IMAGE_LIMIT {
                return Err(Error::Size { limit: IMAGE_LIMIT }.into());
            }
            if !bytes.is_empty() {
                // From<Image> moves the bytes; new_image(&Image) would clone them.
                return Ok(Some(Image::from_bytes(format, bytes).into()));
            }
        }
    }
    Ok(None)
}

// Exercise the Unix process policy on macOS without reading its clipboard.
#[cfg(any(target_os = "linux", all(test, unix)))]
mod linux;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
pub(crate) mod macos;

#[cfg(all(test, target_os = "macos"))]
mod pasteboard_tests;

#[cfg(test)]
mod tests;

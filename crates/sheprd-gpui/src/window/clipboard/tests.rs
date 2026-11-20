// ABOUTME: Verifies bounded native format selection and decoding.
// ABOUTME: Uses isolated data and exact helper processes.
use super::*;
mod pending;
mod text_mode;
use std::error::Error as _;

#[test]
fn text_precedes_images_even_when_empty() -> Result<()> {
    for text in ["", "text\n"] {
        let item = read_with(Mode::Normal, |format, limit| {
            assert_eq!(format, None);
            assert_eq!(limit, TEXT_LIMIT);
            Ok(Some(text.as_bytes().to_vec()))
        })?;
        // `ClipboardItem::text` reports empty text as `None`; inspect the entry.
        assert!(matches!(
            item.as_ref().map(ClipboardItem::entries),
            Some([gpui::ClipboardEntry::String(entry)]) if entry.text() == text
        ));
    }
    Ok(())
}

#[test]
fn image_only_skips_text_and_empty_images() -> Result<()> {
    let item = read_with(Mode::Image, |format, limit| {
        assert!(format.is_some());
        assert_eq!(limit, IMAGE_LIMIT);
        Ok(Some(if format == Some(ImageFormat::Jpeg) {
            vec![1, 2, 3]
        } else {
            Vec::new()
        }))
    })?;
    assert!(matches!(item.as_ref().map(|item| item.entries()),
            Some([gpui::ClipboardEntry::Image(image)]) if image.format == ImageFormat::Jpeg && image.bytes == [1, 2, 3]));
    assert!(read_with(Mode::Image, |_, _| Ok(None))?.is_none());
    Ok(())
}

#[test]
fn tiff_is_requested_only_after_every_bridge_format() -> Result<()> {
    let mut requested = Vec::new();
    let item = read_with(Mode::Image, |format, _| {
        requested.push(format);
        Ok((format == Some(ImageFormat::Tiff)).then(|| vec![1, 2, 3]))
    })?;
    assert_eq!(
        requested,
        [
            Some(ImageFormat::Png),
            Some(ImageFormat::Jpeg),
            Some(ImageFormat::Gif),
            Some(ImageFormat::Webp),
            Some(ImageFormat::Bmp),
            Some(ImageFormat::Tiff),
        ]
    );
    assert!(matches!(item.as_ref().map(|item| item.entries()),
            Some([gpui::ClipboardEntry::Image(image)]) if image.format == ImageFormat::Tiff));
    Ok(())
}

#[test]
fn size_and_encoding_errors_never_fall_through_to_images() -> Result<()> {
    for mode in [Mode::Normal, Mode::Image] {
        let limit = if mode == Mode::Image {
            IMAGE_LIMIT
        } else {
            TEXT_LIMIT
        };
        // Acquisition rejects oversized native data before copying or hashing it.
        // Exercise that contract without allocating a 128 MiB test clipboard.
        assert!(matches!(read_with(mode, |_, requested| {
                    assert_eq!(requested, limit);
                    Err(crate::Error::Clipboard(Error::Size{ limit: requested }))
                }),
                Err(crate::Error::Clipboard(Error::Size{ limit: actual })) if actual == limit));
    }
    assert!(read_with(Mode::Normal, |_, _| Ok(Some(vec![b'a'; TEXT_LIMIT])))?.is_some());
    assert!(matches!(
        read_with(Mode::Normal, |_, _| Ok(Some(vec![b'a'; TEXT_LIMIT + 1]))),
        Err(crate::Error::Clipboard(Error::Size { limit: TEXT_LIMIT }))
    ));
    let result = read_with(Mode::Normal, |format, _| {
        assert_eq!(format, None);
        Ok(Some(vec![255]))
    });
    assert!(matches!(
        result,
        Err(crate::Error::Clipboard(Error::Encoding(_)))
    ));
    if let Err(error) = result {
        assert!(
            error
                .source()
                .is_some_and(|source| source.is::<std::str::Utf8Error>())
        );
    }
    Ok(())
}

#[test]
fn image_acquisition_allows_resize_inputs_above_the_upload_limit() -> Result<()> {
    assert_eq!(IMAGE_LIMIT, 128 * 1024 * 1024);
    let bytes = vec![0; sheprd_client::protocol::MAX_CLIPBOARD_IMAGE_PAYLOAD + 1];
    let pointer = bytes.as_ptr();
    let mut bytes = Some(bytes);
    let item = read_with(Mode::Image, |format, limit| {
        assert_eq!(format, Some(ImageFormat::Png));
        assert_eq!(limit, IMAGE_LIMIT);
        Ok(bytes.take())
    })?;
    assert!(matches!(item.as_ref().map(|item| item.entries()),
            Some([gpui::ClipboardEntry::Image(image)])
                if image.bytes.as_ptr() == pointer
                    && image.bytes.len() > sheprd_client::protocol::MAX_CLIPBOARD_IMAGE_PAYLOAD));
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
#[test]
fn unsupported_platform_is_explicit() {
    assert!(matches!(
        read(Mode::Image),
        Err(crate::Error::Clipboard(Error::Unsupported))
    ));
}

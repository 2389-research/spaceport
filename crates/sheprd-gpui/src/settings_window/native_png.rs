// ABOUTME: Encodes native Settings captures without changing their dimensions or pixels.
// ABOUTME: Preserves the fixed PNG budget used by the private Windows qualifier.
use anyhow::{Result, ensure};
use image::{
    RgbaImage,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};

pub(super) fn encode(image: RgbaImage) -> Result<Vec<u8>> {
    let mut png = Vec::new();
    image.write_with_encoder(PngEncoder::new_with_quality(
        &mut png,
        CompressionType::Best,
        FilterType::Adaptive,
    ))?;
    ensure!(
        png.len() <= 196_608,
        "native Settings PNG exceeds 192 KiB: {} bytes",
        png.len()
    );
    Ok(png)
}

#[cfg(test)]
mod tests;

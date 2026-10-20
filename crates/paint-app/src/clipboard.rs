//! Clipboard integration using arboard on native platforms and fallbacks on web.

use paint_core::image::PixelBuffer;

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context;
#[cfg(not(target_arch = "wasm32"))]
use arboard::{Clipboard, ImageData};
#[cfg(not(target_arch = "wasm32"))]
use paint_core::palette::ColorRgba;
#[cfg(not(target_arch = "wasm32"))]
use std::borrow::Cow;

/// Copies a `PixelBuffer` to the system clipboard as RGBA bitmap data.
#[cfg(not(target_arch = "wasm32"))]
pub fn copy_to_clipboard(buffer: &PixelBuffer) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to open system clipboard")?;
    let img_data = ImageData {
        width: buffer.width() as usize,
        height: buffer.height() as usize,
        bytes: Cow::Borrowed(buffer.as_raw()),
    };
    clipboard
        .set_image(img_data)
        .context("Failed to write image data to clipboard")?;
    Ok(())
}

/// Retrieves an image from the system clipboard and converts it into a `PixelBuffer`.
#[cfg(not(target_arch = "wasm32"))]
pub fn paste_from_clipboard() -> anyhow::Result<PixelBuffer> {
    let mut clipboard = Clipboard::new().context("Failed to open system clipboard")?;
    let img_data = clipboard
        .get_image()
        .context("No image data on system clipboard")?;

    let width = img_data.width as u32;
    let height = img_data.height as u32;
    let mut buf = PixelBuffer::new(width, height, ColorRgba::rgb(255, 255, 255));
    buf.as_raw_mut().copy_from_slice(&img_data.bytes);
    Ok(buf)
}

#[cfg(target_arch = "wasm32")]
pub fn copy_to_clipboard(_buffer: &PixelBuffer) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn paste_from_clipboard() -> anyhow::Result<PixelBuffer> {
    anyhow::bail!("Clipboard image paste not supported on web")
}

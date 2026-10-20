//! Animated GIF export settings, data models, and encoding pipeline.
//!
//! Provides rendering, scaling, frame delay management, loop count configuration,
//! transparency handling, and animated GIF generation.

use std::fs::File;
use std::io::{BufWriter, Cursor, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, RgbaImage};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::export::png_seq::{FrameRangeChoice, LayerExportScope, TransparencyOption};
use crate::frames::AnimationDocument;
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;

#[derive(Error, Debug)]
pub enum GifExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("GIF encoding error: {0}")]
    Encoding(String),
    #[error("Invalid export options: {0}")]
    InvalidOptions(String),
    #[error("No frames matched the export selection")]
    NoFramesToExport,
    #[error("Export operation was cancelled by the user")]
    Cancelled,
}

/// Loop repetition configuration for Animated GIF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GifRepeatOption {
    /// Repeat animation indefinitely.
    #[default]
    Infinite,
    /// Play animation once (do not loop).
    Once,
    /// Repeat animation a specific number of times.
    Custom(u16),
}

/// Settings and options for Animated GIF export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GifExportOptions {
    /// Selection of frames to include.
    pub frame_range: FrameRangeChoice,
    /// Layers included in the export.
    pub layers: LayerExportScope,
    /// Transparency handling (transparent or blended onto a solid background).
    pub transparency: TransparencyOption,
    /// Override FPS for all frames, or use individual frame durations if None.
    pub fps_override: Option<u32>,
    /// Scale multiplier using nearest-neighbor (e.g. 1, 2, 4, 8).
    pub scale: u32,
    /// GIF repeat / looping behavior.
    pub repeat: GifRepeatOption,
}

impl Default for GifExportOptions {
    fn default() -> Self {
        Self {
            frame_range: FrameRangeChoice::All,
            layers: LayerExportScope::VisibleComposite,
            transparency: TransparencyOption::KeepTransparency,
            fps_override: None,
            scale: 1,
            repeat: GifRepeatOption::Infinite,
        }
    }
}

/// Result metadata returned after successful GIF export.
#[derive(Debug, Clone)]
pub struct GifExportResult {
    /// Destination path of the exported GIF file.
    pub output_path: PathBuf,
    /// Number of animation frames written to the GIF.
    pub frame_count: usize,
    /// Width in pixels of the exported GIF.
    pub width: u32,
    /// Height in pixels of the exported GIF.
    pub height: u32,
    /// Total duration of one animation cycle in milliseconds.
    pub total_duration_ms: u64,
}

/// Resolves the list of frame indices to export based on `FrameRangeChoice`.
fn resolve_frame_indices(doc: &AnimationDocument, choice: &FrameRangeChoice) -> Vec<usize> {
    let total_frames = doc.frames.len();
    if total_frames == 0 {
        return Vec::new();
    }

    match choice {
        FrameRangeChoice::All => (0..total_frames).collect(),
        FrameRangeChoice::ActiveFrame => {
            if doc.active_frame_index < total_frames {
                vec![doc.active_frame_index]
            } else {
                vec![0]
            }
        }
        FrameRangeChoice::Tag(tag_name) => {
            if let Some(tag) = doc.tags.iter().find(|t| &t.name == tag_name) {
                let start = tag.from_frame.min(total_frames - 1);
                let end = tag.to_frame.min(total_frames - 1);
                if start <= end {
                    (start..=end).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }
        FrameRangeChoice::CustomRange { start, end } => {
            let s = (*start).min(total_frames - 1);
            let e = (*end).min(total_frames - 1);
            if s <= e {
                (s..=e).collect()
            } else {
                Vec::new()
            }
        }
    }
}

/// Renders a single frame index to a `PixelBuffer` according to layer scope and transparency.
fn render_frame(
    doc: &AnimationDocument,
    frame_idx: usize,
    scope: &LayerExportScope,
    transparency: &TransparencyOption,
) -> Option<PixelBuffer> {
    if frame_idx >= doc.frames.len() {
        return None;
    }

    let mut buffer = match scope {
        LayerExportScope::VisibleComposite | LayerExportScope::OneSequencePerLayer => {
            doc.composite_frame(frame_idx)
        }
        LayerExportScope::SingleLayer(layer_id) => {
            let cel = doc.get_cel(frame_idx, *layer_id)?;
            let mut buf = PixelBuffer::new(doc.width, doc.height, ColorRgba::TRANSPARENT);
            buf.blend_buffer(&cel.buffer, cel.x, cel.y);
            buf
        }
    };

    if let TransparencyOption::Flatten(bg_color) = transparency {
        let mut flattened = PixelBuffer::new(buffer.width(), buffer.height(), *bg_color);
        flattened.blend_buffer(&buffer, 0, 0);
        buffer = flattened;
    }

    Some(buffer)
}

/// Encodes an animation document into an animated GIF writer stream.
pub fn export_gif_to_writer<W: Write + Seek>(
    doc: &AnimationDocument,
    opts: &GifExportOptions,
    writer: &mut W,
    cancel_flag: Option<&AtomicBool>,
) -> Result<GifExportResult, GifExportError> {
    if opts.scale == 0 {
        return Err(GifExportError::InvalidOptions(
            "Scale multiplier must be at least 1".into(),
        ));
    }

    let frame_indices = resolve_frame_indices(doc, &opts.frame_range);
    if frame_indices.is_empty() {
        return Err(GifExportError::NoFramesToExport);
    }

    let mut encoder = GifEncoder::new(writer);
    let repeat = match opts.repeat {
        GifRepeatOption::Infinite => Repeat::Infinite,
        GifRepeatOption::Once => Repeat::Finite(0),
        GifRepeatOption::Custom(count) => Repeat::Finite(count),
    };
    encoder
        .set_repeat(repeat)
        .map_err(|e| GifExportError::Encoding(e.to_string()))?;

    let mut exported_frames = 0;
    let mut final_width = 0;
    let mut final_height = 0;
    let mut total_duration_ms: u64 = 0;

    for &idx in &frame_indices {
        if let Some(flag) = cancel_flag {
            if flag.load(Ordering::Relaxed) {
                return Err(GifExportError::Cancelled);
            }
        }

        let Some(buf) = render_frame(doc, idx, &opts.layers, &opts.transparency) else {
            continue;
        };

        let scaled_buf = if opts.scale > 1 {
            buf.scale_nearest_neighbor(opts.scale)
        } else {
            buf
        };

        final_width = scaled_buf.width();
        final_height = scaled_buf.height();

        let duration_ms = if let Some(fps) = opts.fps_override {
            1000u64.checked_div(fps as u64).unwrap_or(100).max(10)
        } else {
            doc.frames
                .get(idx)
                .map(|f| f.duration_ms as u64)
                .unwrap_or(100)
                .max(10)
        };

        total_duration_ms += duration_ms;

        let rgba_raw = scaled_buf.as_raw().to_vec();
        let rgba_img = RgbaImage::from_raw(scaled_buf.width(), scaled_buf.height(), rgba_raw)
            .ok_or_else(|| GifExportError::Encoding("Failed to construct RGBA image".into()))?;

        let delay = Delay::from_saturating_duration(Duration::from_millis(duration_ms));
        let frame = Frame::from_parts(rgba_img, 0, 0, delay);

        encoder
            .encode_frame(frame)
            .map_err(|e| GifExportError::Encoding(e.to_string()))?;

        exported_frames += 1;
    }

    if exported_frames == 0 {
        return Err(GifExportError::NoFramesToExport);
    }

    Ok(GifExportResult {
        output_path: PathBuf::new(),
        frame_count: exported_frames,
        width: final_width,
        height: final_height,
        total_duration_ms,
    })
}

/// Encodes an animation document into in-memory GIF bytes.
pub fn export_gif_to_memory(
    doc: &AnimationDocument,
    opts: &GifExportOptions,
) -> Result<Vec<u8>, GifExportError> {
    let mut cursor = Cursor::new(Vec::new());
    export_gif_to_writer(doc, opts, &mut cursor, None)?;
    Ok(cursor.into_inner())
}

/// Encodes and saves an animation document as an animated GIF to a filesystem path.
pub fn export_gif_to_disk<P: AsRef<Path>>(
    doc: &AnimationDocument,
    opts: &GifExportOptions,
    path: P,
) -> Result<GifExportResult, GifExportError> {
    let path_ref = path.as_ref();
    if let Some(parent) = path_ref.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let file = File::create(path_ref)?;
    let mut writer = BufWriter::new(file);
    let mut result = export_gif_to_writer(doc, opts, &mut writer, None)?;
    writer.flush()?;
    result.output_path = path_ref.to_path_buf();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::RED;

    #[test]
    fn test_export_gif_animation_roundtrip() {
        let mut doc = AnimationDocument::new(32, 32, ColorRgba::rgb(255, 255, 255));
        doc.fps = 10;
        let layer_id = doc.add_layer("Main");

        // Frame 0: Red square
        if let Some(cel) = doc.get_cel_mut(0, layer_id) {
            cel.buffer.clear(RED);
        }

        // Frame 1: Blue square
        doc.add_frame(100, false);
        if let Some(cel) = doc.get_cel_mut(1, layer_id) {
            cel.buffer.clear(ColorRgba::rgb(0, 0, 255));
        }

        let opts = GifExportOptions {
            frame_range: FrameRangeChoice::All,
            scale: 2,
            fps_override: Some(10),
            ..Default::default()
        };

        let gif_bytes = export_gif_to_memory(&doc, &opts).expect("export gif memory");
        assert!(!gif_bytes.is_empty());
        assert_eq!(&gif_bytes[0..3], b"GIF");

        // Verify with image crate decoding
        let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(&gif_bytes)).expect("decode");
        use image::AnimationDecoder;
        let frames = decoder
            .into_frames()
            .collect_frames()
            .expect("collect frames");
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].buffer().width(), 64);
        assert_eq!(frames[0].buffer().height(), 64);
    }
}

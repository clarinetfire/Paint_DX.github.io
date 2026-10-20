//! PNG Sequence export settings, data models, and batch pipeline.
//!
//! Provides rendering, scaling, trimming (None, Union, Per-frame), border padding,
//! transparency handling (transparent RGBA vs flattened solid color),
//! zero-padded indexing, collision policies, and cancellation support.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frames::AnimationDocument;
use crate::image::PixelBuffer;
use crate::io::{save_to_path, ImageFileFormat, ImageIoError};
use crate::layers::LayerId;
use crate::palette::ColorRgba;

#[derive(Error, Debug)]
pub enum PngSequenceExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image codec error: {0}")]
    Image(#[from] ImageIoError),
    #[error("Invalid export options: {0}")]
    InvalidOptions(String),
    #[error("No frames matched the export selection")]
    NoFramesToExport,
    #[error("Export operation was cancelled by the user")]
    Cancelled,
}

/// Selection of animation frames to export.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FrameRangeChoice {
    /// Export all frames in the document.
    #[default]
    All,
    /// Export only the active frame.
    ActiveFrame,
    /// Export frames belonging to the specified named tag.
    Tag(String),
    /// Export an explicit range of frames `[start..=end]`.
    CustomRange { start: usize, end: usize },
}

/// Scope of layers to export.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayerExportScope {
    /// Export the composite of all visible layers.
    #[default]
    VisibleComposite,
    /// Export only the specified layer.
    SingleLayer(LayerId),
    /// Export each layer as its own numbered sequence.
    OneSequencePerLayer,
}

/// Transparency handling during PNG export.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TransparencyOption {
    /// Keep transparent pixels (RGBA PNG with alpha=0 on empty pixels).
    #[default]
    KeepTransparency,
    /// Flatten onto a solid background color (opaque PNG with alpha=255 everywhere).
    Flatten(ColorRgba),
}

/// Image trim / crop mode for animation frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TrimMode {
    /// Keep full document canvas dimensions.
    #[default]
    None,
    /// Crop every frame by the same union bounding box across all chosen frames (preserves relative alignment).
    Union,
    /// Crop each frame to its own non-transparent bounding box.
    PerFrame,
}

/// Collision handling policy if a destination file already exists on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExistingFilesPolicy {
    /// Overwrite any existing file.
    #[default]
    Overwrite,
    /// Skip writing frames that already exist.
    Skip,
    /// Automatically append a numeric suffix (e.g. `_1.png`, `_2.png`).
    AutoRename,
}

/// Comprehensive configuration options for PNG sequence export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PngSequenceExportOptions {
    /// Destination directory where PNG files will be written.
    pub output_dir: PathBuf,
    /// Filename prefix / pattern (e.g. "walk_" or "frame_").
    pub pattern: String,
    /// Starting number for index numbering (e.g. 0 or 1).
    pub start_index: u32,
    /// Number of digits for zero-padding (e.g. 4 -> 0000, 0001).
    pub padding_digits: usize,
    /// Range of frames to include.
    pub frame_range: FrameRangeChoice,
    /// Layer inclusion mode.
    pub layers: LayerExportScope,
    /// Transparency or solid flattening mode.
    pub transparency: TransparencyOption,
    /// Nearest-neighbor scaling multiplier (1x to 8x).
    pub scale: u32,
    /// Trim mode (None, Union, PerFrame).
    pub trim: TrimMode,
    /// Additional border padding in pixels around each image.
    pub padding: u32,
    /// Action to take if target file exists.
    pub existing_files: ExistingFilesPolicy,
}

impl Default for PngSequenceExportOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("export"),
            pattern: "frame_".to_string(),
            start_index: 0,
            padding_digits: 4,
            frame_range: FrameRangeChoice::All,
            layers: LayerExportScope::VisibleComposite,
            transparency: TransparencyOption::KeepTransparency,
            scale: 1,
            trim: TrimMode::None,
            padding: 0,
            existing_files: ExistingFilesPolicy::Overwrite,
        }
    }
}

/// Metadata describing an individual exported image file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedFrameInfo {
    pub frame_index: usize,
    pub file_path: PathBuf,
    pub layer_id: Option<LayerId>,
    pub layer_name: Option<String>,
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub duration_ms: u32,
}

/// Result summary of a sequence export operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PngSequenceExportResult {
    pub exported_files: Vec<ExportedFrameInfo>,
    pub skipped_files: Vec<PathBuf>,
    pub total_frames: usize,
}

/// Resolves the list of frame indices to export based on document and choice.
pub fn resolve_frame_indices(doc: &AnimationDocument, range: &FrameRangeChoice) -> Vec<usize> {
    if doc.frames.is_empty() {
        return Vec::new();
    }

    match range {
        FrameRangeChoice::All => (0..doc.frames.len()).collect(),
        FrameRangeChoice::ActiveFrame => {
            let active = doc.active_frame_index.min(doc.frames.len() - 1);
            vec![active]
        }
        FrameRangeChoice::Tag(tag_name) => {
            if let Some(tag) = doc.tags.iter().find(|t| &t.name == tag_name) {
                let start = tag.from_frame.min(doc.frames.len() - 1);
                let end = tag.to_frame.min(doc.frames.len() - 1);
                (start..=end).collect()
            } else {
                Vec::new()
            }
        }
        FrameRangeChoice::CustomRange { start, end } => {
            if *start >= doc.frames.len() {
                Vec::new()
            } else {
                let s = *start;
                let e = (*end).min(doc.frames.len() - 1);
                if s <= e {
                    (s..=e).collect()
                } else {
                    Vec::new()
                }
            }
        }
    }
}

/// Computes the union bounding box `(min_x, min_y, width, height)` of non-transparent content across multiple buffers.
pub fn compute_union_trim_rect(buffers: &[PixelBuffer]) -> Option<(u32, u32, u32, u32)> {
    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for buf in buffers {
        if let Some((bx, by, bw, bh)) = buf.bounding_box_non_transparent() {
            found = true;
            min_x = min_x.min(bx);
            min_y = min_y.min(by);
            max_x = max_x.max(bx + bw - 1);
            max_y = max_y.max(by + bh - 1);
        }
    }

    if found {
        Some((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
    } else {
        None
    }
}

/// Renders a single layer's content for a specific frame into a PixelBuffer matching document dimensions.
pub fn render_frame_for_layer(
    doc: &AnimationDocument,
    frame_idx: usize,
    layer_id: LayerId,
) -> PixelBuffer {
    let mut output = PixelBuffer::new(doc.width, doc.height, ColorRgba::TRANSPARENT);
    if let Some(frame) = doc.frames.get(frame_idx) {
        if let Some(meta) = doc.layers.iter().find(|l| l.id == layer_id) {
            if let Some(cel) = frame.get_cel(layer_id) {
                if meta.opacity == 100 {
                    output.blend_buffer(&cel.buffer, cel.x, cel.y);
                } else {
                    let mut temp = cel.buffer.clone();
                    temp.apply_opacity(meta.opacity);
                    output.blend_buffer(&temp, cel.x, cel.y);
                }
            }
        }
    }
    output
}

/// Generates a standardized filename for a sequence frame.
pub fn format_sequence_filename(
    pattern: &str,
    layer_name: Option<&str>,
    index: u32,
    padding_digits: usize,
) -> String {
    let digits = padding_digits.clamp(1, 8);
    let index_str = format!("{:0width$}", index, width = digits);
    let sanitized_prefix = if pattern.is_empty() {
        "frame_"
    } else {
        pattern
    };

    if let Some(layer) = layer_name {
        let safe_layer: String = layer
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        format!("{}{}_{}.png", sanitized_prefix, safe_layer, index_str)
    } else {
        format!("{}{}.png", sanitized_prefix, index_str)
    }
}

/// Prepares a processed image buffer and computes its offset relative to the canvas origin.
pub fn prepare_exported_buffer(
    raw_buffer: &PixelBuffer,
    trim_rect: Option<(u32, u32, u32, u32)>,
    padding: u32,
    scale: u32,
    transparency: &TransparencyOption,
) -> (PixelBuffer, i32, i32) {
    let (cropped, offset_x, offset_y) = if let Some((tx, ty, tw, th)) = trim_rect {
        (raw_buffer.crop(tx, ty, tw, th), tx as i32, ty as i32)
    } else {
        (raw_buffer.clone(), 0, 0)
    };

    let padded = if padding > 0 {
        let pad_fill = match transparency {
            TransparencyOption::KeepTransparency => ColorRgba::TRANSPARENT,
            TransparencyOption::Flatten(bg) => *bg,
        };
        cropped.pad(padding, pad_fill)
    } else {
        cropped
    };

    let scaled = if scale > 1 {
        padded.scale_nearest_neighbor(scale)
    } else {
        padded
    };

    let final_buffer = match transparency {
        TransparencyOption::KeepTransparency => scaled,
        TransparencyOption::Flatten(bg) => scaled.flatten_on_color(*bg),
    };

    (final_buffer, offset_x, offset_y)
}

/// Executes the full PNG sequence export pipeline with optional progress callback and cancellation token.
///
/// `progress_cb(current_step, total_steps) -> bool`: Return `false` to abort export.
pub fn export_png_sequence<F>(
    doc: &AnimationDocument,
    options: &PngSequenceExportOptions,
    cancel_token: Option<&AtomicBool>,
    mut progress_cb: F,
) -> Result<PngSequenceExportResult, PngSequenceExportError>
where
    F: FnMut(usize, usize) -> bool,
{
    let frame_indices = resolve_frame_indices(doc, &options.frame_range);
    if frame_indices.is_empty() {
        return Err(PngSequenceExportError::NoFramesToExport);
    }

    std::fs::create_dir_all(&options.output_dir)?;

    let mut result = PngSequenceExportResult {
        exported_files: Vec::new(),
        skipped_files: Vec::new(),
        total_frames: frame_indices.len(),
    };

    // Determine target layer streams
    let layer_streams: Vec<(Option<LayerId>, Option<String>)> = match &options.layers {
        LayerExportScope::VisibleComposite => vec![(None, None)],
        LayerExportScope::SingleLayer(id) => {
            let name = doc
                .layers
                .iter()
                .find(|l| l.id == *id)
                .map(|l| l.name.clone());
            vec![(Some(*id), name)]
        }
        LayerExportScope::OneSequencePerLayer => doc
            .layers
            .iter()
            .map(|l| (Some(l.id), Some(l.name.clone())))
            .collect(),
    };

    let total_operations = frame_indices.len() * layer_streams.len();
    let mut current_op = 0;

    for (layer_id, layer_name) in layer_streams {
        // Pre-render raw buffers for this layer stream to compute union trim if needed
        let mut raw_buffers = Vec::with_capacity(frame_indices.len());
        for &idx in &frame_indices {
            let buf = match layer_id {
                Some(lid) => render_frame_for_layer(doc, idx, lid),
                None => doc.composite_frame(idx),
            };
            raw_buffers.push(buf);
        }

        let union_rect = match options.trim {
            TrimMode::Union => {
                let rect = compute_union_trim_rect(&raw_buffers);
                // Fallback to 1x1 at (0,0) if completely empty
                rect.or(Some((0, 0, 1.max(doc.width), 1.max(doc.height))))
            }
            _ => None,
        };

        for (seq_idx, (&frame_idx, raw_buf)) in
            frame_indices.iter().zip(raw_buffers.iter()).enumerate()
        {
            // Check cancellation
            if let Some(token) = cancel_token {
                if token.load(Ordering::Relaxed) {
                    return Err(PngSequenceExportError::Cancelled);
                }
            }
            if !progress_cb(current_op, total_operations) {
                return Err(PngSequenceExportError::Cancelled);
            }

            let trim_rect = match options.trim {
                TrimMode::None => None,
                TrimMode::Union => union_rect,
                TrimMode::PerFrame => raw_buf
                    .bounding_box_non_transparent()
                    .or(Some((0, 0, 1, 1))),
            };

            let (final_buf, offset_x, offset_y) = prepare_exported_buffer(
                raw_buf,
                trim_rect,
                options.padding,
                options.scale,
                &options.transparency,
            );

            let number = options.start_index.saturating_add(seq_idx as u32);
            let filename = format_sequence_filename(
                &options.pattern,
                layer_name.as_deref(),
                number,
                options.padding_digits,
            );

            let mut target_path = options.output_dir.join(&filename);

            if target_path.exists() {
                match options.existing_files {
                    ExistingFilesPolicy::Overwrite => {}
                    ExistingFilesPolicy::Skip => {
                        result.skipped_files.push(target_path);
                        current_op += 1;
                        continue;
                    }
                    ExistingFilesPolicy::AutoRename => {
                        let stem = target_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("frame");
                        let ext = target_path
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("png");
                        let mut counter = 1;
                        loop {
                            let candidate = options
                                .output_dir
                                .join(format!("{}_{}.{}", stem, counter, ext));
                            if !candidate.exists() {
                                target_path = candidate;
                                break;
                            }
                            counter += 1;
                        }
                    }
                }
            }

            save_to_path(&final_buf, &target_path, ImageFileFormat::Png)?;

            let duration = doc
                .frames
                .get(frame_idx)
                .map(|f| f.duration_ms)
                .unwrap_or(83);

            result.exported_files.push(ExportedFrameInfo {
                frame_index: frame_idx,
                file_path: target_path,
                layer_id,
                layer_name: layer_name.clone(),
                width: final_buf.width(),
                height: final_buf.height(),
                offset_x,
                offset_y,
                duration_ms: duration,
            });

            current_op += 1;
        }
    }

    let _ = progress_cb(total_operations, total_operations);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frames::{AnimTag, Cel, Frame, LayerMetadata};
    use crate::palette::WHITE;
    use std::sync::atomic::AtomicBool;

    fn create_test_anim_doc() -> AnimationDocument {
        let mut doc = AnimationDocument::new_transparent(16, 16);
        let layer1 = LayerId(1);
        let layer2 = LayerId(2);
        doc.layers.push(LayerMetadata::new(layer2, "Overlay"));

        // Frame 0: Red square at (2, 2, 4, 4) on Layer 1
        let mut cel0 = Cel::new(16, 16);
        for y in 2..6 {
            for x in 2..6 {
                cel0.buffer.set_pixel(x, y, ColorRgba::rgb(255, 0, 0));
            }
        }
        doc.frames[0].set_cel(layer1, cel0);

        // Frame 1: Blue square at (6, 6, 4, 4) on Layer 1 + Green dot at (0, 0) on Layer 2
        let mut frame1 = Frame::new(100);
        let mut cel1_l1 = Cel::new(16, 16);
        for y in 6..10 {
            for x in 6..10 {
                cel1_l1.buffer.set_pixel(x, y, ColorRgba::rgb(0, 0, 255));
            }
        }
        frame1.set_cel(layer1, cel1_l1);
        let mut cel1_l2 = Cel::new(16, 16);
        cel1_l2.buffer.set_pixel(0, 0, ColorRgba::rgb(0, 255, 0));
        frame1.set_cel(layer2, cel1_l2);
        doc.frames.push(frame1);

        // Frame 2: Yellow dot at (10, 10)
        let mut frame2 = Frame::new(150);
        let mut cel2 = Cel::new(16, 16);
        cel2.buffer.set_pixel(10, 10, ColorRgba::rgb(255, 255, 0));
        frame2.set_cel(layer1, cel2);
        doc.frames.push(frame2);

        doc.tags.push(AnimTag::new("first_two", 0, 1));
        doc
    }

    #[test]
    fn test_format_sequence_filename() {
        assert_eq!(
            format_sequence_filename("anim_", None, 0, 4),
            "anim_0000.png"
        );
        assert_eq!(
            format_sequence_filename("anim_", Some("Layer 1"), 5, 3),
            "anim_Layer_1_005.png"
        );
        assert_eq!(format_sequence_filename("", None, 12, 2), "frame_12.png");
    }

    #[test]
    fn test_resolve_frame_indices() {
        let doc = create_test_anim_doc();
        assert_eq!(
            resolve_frame_indices(&doc, &FrameRangeChoice::All),
            vec![0, 1, 2]
        );
        assert_eq!(
            resolve_frame_indices(&doc, &FrameRangeChoice::Tag("first_two".to_string())),
            vec![0, 1]
        );
        assert_eq!(
            resolve_frame_indices(&doc, &FrameRangeChoice::CustomRange { start: 1, end: 2 }),
            vec![1, 2]
        );
    }

    #[test]
    fn test_export_png_sequence_transparent_and_flattened() {
        let doc = create_test_anim_doc();
        let temp_dir = std::env::temp_dir().join("paint_dx_test_png_seq");
        let _ = std::fs::remove_dir_all(&temp_dir);

        // 1. Transparent export
        let mut opts = PngSequenceExportOptions {
            output_dir: temp_dir.join("transparent"),
            pattern: "trans_".to_string(),
            start_index: 1,
            padding_digits: 3,
            frame_range: FrameRangeChoice::All,
            layers: LayerExportScope::VisibleComposite,
            transparency: TransparencyOption::KeepTransparency,
            scale: 2,
            trim: TrimMode::None,
            padding: 0,
            existing_files: ExistingFilesPolicy::Overwrite,
        };

        let result = export_png_sequence(&doc, &opts, None, |_, _| true).unwrap();
        assert_eq!(result.exported_files.len(), 3);
        assert_eq!(result.exported_files[0].width, 32);
        assert_eq!(result.exported_files[0].height, 32);
        assert!(result.exported_files[0]
            .file_path
            .to_str()
            .unwrap()
            .ends_with("trans_001.png"));

        // Verify transparent pixels have alpha 0
        let img0 = crate::io::load_from_path(&result.exported_files[0].file_path).unwrap();
        assert_eq!(img0.get_pixel(0, 0), Some(ColorRgba::TRANSPARENT));
        // Pixel at (2*2, 2*2) = (4, 4) should be Red
        assert_eq!(img0.get_pixel(4, 4), Some(ColorRgba::rgb(255, 0, 0)));

        // 2. Flattened export on White
        opts.output_dir = temp_dir.join("flattened");
        opts.pattern = "flat_".to_string();
        opts.transparency = TransparencyOption::Flatten(WHITE);

        let flat_res = export_png_sequence(&doc, &opts, None, |_, _| true).unwrap();
        assert_eq!(flat_res.exported_files.len(), 3);

        let flat_img0 = crate::io::load_from_path(&flat_res.exported_files[0].file_path).unwrap();
        // Empty pixel at (0, 0) should be White with alpha 255
        assert_eq!(flat_img0.get_pixel(0, 0), Some(WHITE));
        assert_eq!(flat_img0.get_pixel(4, 4), Some(ColorRgba::rgb(255, 0, 0)));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_export_png_sequence_union_trim() {
        let doc = create_test_anim_doc();
        let temp_dir = std::env::temp_dir().join("paint_dx_test_png_seq_trim");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let opts = PngSequenceExportOptions {
            output_dir: temp_dir.clone(),
            pattern: "trim_".to_string(),
            start_index: 0,
            padding_digits: 2,
            frame_range: FrameRangeChoice::All,
            layers: LayerExportScope::VisibleComposite,
            transparency: TransparencyOption::KeepTransparency,
            scale: 1,
            trim: TrimMode::Union,
            padding: 1,
            existing_files: ExistingFilesPolicy::Overwrite,
        };

        let result = export_png_sequence(&doc, &opts, None, |_, _| true).unwrap();
        assert_eq!(result.exported_files.len(), 3);

        // Union bounding box across frames 0, 1, 2:
        // Frame 0: (2, 2) to (5, 5)
        // Frame 1: (0, 0) and (6, 6) to (9, 9)
        // Frame 2: (10, 10)
        // Union covers (0, 0) to (10, 10) -> size 11x11, plus padding 1 -> 13x13
        for f in &result.exported_files {
            assert_eq!(f.width, 13);
            assert_eq!(f.height, 13);
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_export_cancellation() {
        let doc = create_test_anim_doc();
        let temp_dir = std::env::temp_dir().join("paint_dx_test_cancel");
        let cancel_token = AtomicBool::new(true);

        let opts = PngSequenceExportOptions {
            output_dir: temp_dir,
            ..Default::default()
        };

        let res = export_png_sequence(&doc, &opts, Some(&cancel_token), |_, _| true);
        assert!(matches!(res, Err(PngSequenceExportError::Cancelled)));
    }
}

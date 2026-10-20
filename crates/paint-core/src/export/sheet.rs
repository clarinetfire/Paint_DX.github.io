//! Sprite Sheet export settings, layout models, bin packing, extrusion, and metadata generation.
//!
//! Supports Horizontal Strip, Vertical Strip, Grid, and MaxRects Packed layouts,
//! Power-of-Two (POT) rounding, edge extrusion dilation, trim modes,
//! transparent RGBA or solid background flattening, and versioned JSON/CSV metadata.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

use crate::export::png_seq::{
    compute_union_trim_rect, prepare_exported_buffer, render_frame_for_layer,
    resolve_frame_indices, FrameRangeChoice, LayerExportScope, TransparencyOption, TrimMode,
};
use crate::frames::{AnimTag, AnimationDocument};
use crate::image::PixelBuffer;
use crate::io::{save_to_path, ImageFileFormat, ImageIoError};
use crate::palette::ColorRgba;

#[derive(Error, Debug)]
pub enum SpriteSheetExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image codec error: {0}")]
    Image(#[from] ImageIoError),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid export options: {0}")]
    InvalidOptions(String),
    #[error("No frames matched the export selection")]
    NoFramesToExport,
    #[error("Sprite sheet exceeds maximum allowed size ({0}x{1} > {2}x{3})")]
    MaxSizeExceeded(u32, u32, u32, u32),
}

/// Layout mode for positioning frames on the sprite sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SheetLayout {
    /// Frames aligned in a single horizontal row from left to right.
    HorizontalStrip,
    /// Frames stacked in a single vertical column from top to bottom.
    VerticalStrip,
    /// Frames organized in a grid with a fixed number of columns.
    #[default]
    Grid,
    /// Packed tightly using a 2D bin packing algorithm (MaxRects).
    Packed,
}

/// Sizing policy for the generated sprite sheet canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SheetSizeMode {
    /// Fit tightly to bounding content plus padding/border.
    #[default]
    Auto,
    /// Width and height rounded up to the nearest power of two (e.g. 64, 128, 256, 512, 1024).
    PowerOfTwo,
    /// Constrain sheet within maximum width and height, erroring if overflow occurs.
    MaxSize { max_w: u32, max_h: u32 },
}

/// Metadata format to produce alongside the sprite sheet PNG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MetadataFormat {
    /// Versioned JSON metadata file adhering to the Paint DX specification.
    #[default]
    Json,
    /// Comma-separated values (CSV) metadata file.
    Csv,
    /// No metadata file (export image only).
    None,
}

/// Frame ordering within the sprite sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FrameOrdering {
    /// Export strictly by sequential frame index (0, 1, 2, ...).
    #[default]
    ByFrameIndex,
    /// Group frames by animation tag.
    ByTag,
}

/// Options configuring the generation of a sprite sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteSheetOptions {
    /// Output layout arrangement.
    pub layout: SheetLayout,
    /// Number of columns when using `SheetLayout::Grid`. Defaults to 4.
    pub columns: u32,
    /// Spacing between adjacent cells in pixels.
    pub spacing: u32,
    /// Outer border / margin in pixels around the perimeter of the sheet.
    pub border: u32,
    /// Edge pixel extrusion in pixels (e.g. 1px dilation to prevent texture filtering bleed).
    pub extrusion: u32,
    /// Canvas sizing constraints (Auto, PowerOfTwo, MaxSize).
    pub size_mode: SheetSizeMode,
    /// Frame range selector.
    pub frame_range: FrameRangeChoice,
    /// Layer inclusion mode.
    pub layers: LayerExportScope,
    /// Transparency handling (RGBA transparent vs solid background).
    pub transparency: TransparencyOption,
    /// Nearest-neighbor scaling multiplier (1x to 8x).
    pub scale: u32,
    /// Trim mode (None, Union, PerFrame).
    pub trim: TrimMode,
    /// Additional border padding in pixels around each frame before extrusion.
    pub padding: u32,
    /// Metadata format (JSON, CSV, None).
    pub metadata_format: MetadataFormat,
    /// Ordering of frames.
    pub ordering: FrameOrdering,
}

impl Default for SpriteSheetOptions {
    fn default() -> Self {
        Self {
            layout: SheetLayout::Grid,
            columns: 4,
            spacing: 0,
            border: 0,
            extrusion: 0,
            size_mode: SheetSizeMode::Auto,
            frame_range: FrameRangeChoice::All,
            layers: LayerExportScope::VisibleComposite,
            transparency: TransparencyOption::KeepTransparency,
            scale: 1,
            trim: TrimMode::None,
            padding: 0,
            metadata_format: MetadataFormat::Json,
            ordering: FrameOrdering::ByFrameIndex,
        }
    }
}

/// Metadata describing a single frame placed on the sprite sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedFrameInfo {
    /// Original animation frame index in the document.
    pub frame_index: usize,
    /// Optional tag name if the frame belongs to a tag.
    pub tag_name: Option<String>,
    /// X coordinate on the sheet where the frame content begins (inside any extrusion).
    pub x: u32,
    /// Y coordinate on the sheet where the frame content begins (inside any extrusion).
    pub y: u32,
    /// Rendered frame content width in pixels on the sheet.
    pub w: u32,
    /// Rendered frame content height in pixels on the sheet.
    pub h: u32,
    /// Animation duration in milliseconds.
    pub duration_ms: u32,
    /// Trim offset X relative to the original uncropped canvas.
    pub trim_offset_x: i32,
    /// Trim offset Y relative to the original uncropped canvas.
    pub trim_offset_y: i32,
    /// Original source document width (scaled).
    pub source_w: u32,
    /// Original source document height (scaled).
    pub source_h: u32,
}

/// Complete in-memory result of sprite sheet generation.
#[derive(Debug, Clone)]
pub struct SpriteSheetResult {
    /// The composited pixel buffer containing all frames.
    pub sheet_buffer: PixelBuffer,
    /// Placed frame information including coordinates and durations.
    pub frames: Vec<PlacedFrameInfo>,
    /// Animation tags included in the export.
    pub tags: Vec<AnimTag>,
    /// Total width of the sprite sheet.
    pub sheet_width: u32,
    /// Total height of the sprite sheet.
    pub sheet_height: u32,
    /// Optional generated JSON metadata string.
    pub json_metadata: Option<String>,
    /// Optional generated CSV metadata string.
    pub csv_metadata: Option<String>,
}

// ---------------------------------------------------------------------------
// JSON Specification Models (Plan §2.5 Schema)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSheetDoc {
    pub version: u32,
    pub image: String,
    pub size: JsonSheetSize,
    pub frames: Vec<JsonSheetFrame>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<JsonSheetTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSheetSize {
    pub w: u32,
    pub h: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSheetPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSheetFrame {
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub duration_ms: u32,
    pub trim_offset: JsonSheetPoint,
    pub source_size: JsonSheetSize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSheetTag {
    pub name: String,
    pub from: usize,
    pub to: usize,
}

// ---------------------------------------------------------------------------
// Extrusion (Edge Pixel Dilation)
// ---------------------------------------------------------------------------

/// Dilates the outermost border pixels of a buffer by `extrusion` pixels outward.
///
/// This eliminates texture bleeding / UV seam artifacts when game engines apply linear filtering or mipmaps.
pub fn extrude_cell(cell: &PixelBuffer, extrusion: u32) -> PixelBuffer {
    if extrusion == 0 || cell.width() == 0 || cell.height() == 0 {
        return cell.clone();
    }

    let w = cell.width();
    let h = cell.height();
    let ext = extrusion;
    let new_w = w + 2 * ext;
    let new_h = h + 2 * ext;

    let mut output = PixelBuffer::new(new_w, new_h, ColorRgba::TRANSPARENT);

    // 1. Copy the main content into the center
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = cell.get_pixel(x, y) {
                output.set_pixel(x + ext, y + ext, p);
            }
        }
    }

    // 2. Extrude top and bottom edges
    for x in 0..w {
        let top_color = cell.get_pixel(x, 0).unwrap_or(ColorRgba::TRANSPARENT);
        for dy in 0..ext {
            output.set_pixel(x + ext, dy, top_color);
        }
        let bot_color = cell.get_pixel(x, h - 1).unwrap_or(ColorRgba::TRANSPARENT);
        for dy in 0..ext {
            output.set_pixel(x + ext, ext + h + dy, bot_color);
        }
    }

    // 3. Extrude left and right edges
    for y in 0..h {
        let left_color = cell.get_pixel(0, y).unwrap_or(ColorRgba::TRANSPARENT);
        for dx in 0..ext {
            output.set_pixel(dx, y + ext, left_color);
        }
        let right_color = cell.get_pixel(w - 1, y).unwrap_or(ColorRgba::TRANSPARENT);
        for dx in 0..ext {
            output.set_pixel(ext + w + dx, y + ext, right_color);
        }
    }

    // 4. Extrude four corners
    let tl = cell.get_pixel(0, 0).unwrap_or(ColorRgba::TRANSPARENT);
    let tr = cell.get_pixel(w - 1, 0).unwrap_or(ColorRgba::TRANSPARENT);
    let bl = cell.get_pixel(0, h - 1).unwrap_or(ColorRgba::TRANSPARENT);
    let br = cell
        .get_pixel(w - 1, h - 1)
        .unwrap_or(ColorRgba::TRANSPARENT);

    for dy in 0..ext {
        for dx in 0..ext {
            output.set_pixel(dx, dy, tl);
            output.set_pixel(ext + w + dx, dy, tr);
            output.set_pixel(dx, ext + h + dy, bl);
            output.set_pixel(ext + w + dx, ext + h + dy, br);
        }
    }

    output
}

// ---------------------------------------------------------------------------
// 2D Bin Packing (MaxRects for Packed Layout)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Rect {
    fn contains(&self, other: &Rect) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.w >= other.x + other.w
            && self.y + self.h >= other.y + other.h
    }

    fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

struct MaxRectsPacker {
    free_rects: Vec<Rect>,
}

impl MaxRectsPacker {
    fn new(width: u32, height: u32) -> Self {
        Self {
            free_rects: vec![Rect {
                x: 0,
                y: 0,
                w: width,
                h: height,
            }],
        }
    }

    fn insert(&mut self, w: u32, h: u32) -> Option<Rect> {
        let mut best_rect: Option<Rect> = None;
        let mut best_short_fit = u32::MAX;
        let mut best_long_fit = u32::MAX;
        let mut best_index = 0;

        for (i, r) in self.free_rects.iter().enumerate() {
            if r.w >= w && r.h >= h {
                let leftover_x = r.w - w;
                let leftover_y = r.h - h;
                let short_side_fit = leftover_x.min(leftover_y);
                let long_side_fit = leftover_x.max(leftover_y);

                if short_side_fit < best_short_fit
                    || (short_side_fit == best_short_fit && long_side_fit < best_long_fit)
                {
                    best_short_fit = short_side_fit;
                    best_long_fit = long_side_fit;
                    best_rect = Some(Rect {
                        x: r.x,
                        y: r.y,
                        w,
                        h,
                    });
                    best_index = i;
                }
            }
        }

        let placed = best_rect?;
        let _ = best_index;

        // Split free rectangles intersecting the placed rectangle
        let mut new_free = Vec::new();
        for r in &self.free_rects {
            if r.intersects(&placed) {
                // Top split
                if placed.y > r.y && placed.y < r.y + r.h {
                    new_free.push(Rect {
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: placed.y - r.y,
                    });
                }
                // Bottom split
                if placed.y + placed.h < r.y + r.h {
                    new_free.push(Rect {
                        x: r.x,
                        y: placed.y + placed.h,
                        w: r.w,
                        h: (r.y + r.h) - (placed.y + placed.h),
                    });
                }
                // Left split
                if placed.x > r.x && placed.x < r.x + r.w {
                    new_free.push(Rect {
                        x: r.x,
                        y: r.y,
                        w: placed.x - r.x,
                        h: r.h,
                    });
                }
                // Right split
                if placed.x + placed.w < r.x + r.w {
                    new_free.push(Rect {
                        x: placed.x + placed.w,
                        y: r.y,
                        w: (r.x + r.w) - (placed.x + placed.w),
                        h: r.h,
                    });
                }
            } else {
                new_free.push(*r);
            }
        }

        // Prune redundant rectangles
        self.free_rects.clear();
        for i in 0..new_free.len() {
            let mut contained = false;
            for j in 0..new_free.len() {
                if i != j && new_free[j].contains(&new_free[i]) {
                    contained = true;
                    break;
                }
            }
            if !contained && new_free[i].w > 0 && new_free[i].h > 0 {
                self.free_rects.push(new_free[i]);
            }
        }

        Some(placed)
    }
}

/// Packs rectangular items into the smallest bounding sheet.
fn pack_rectangles(sizes: &[(u32, u32)], spacing: u32, border: u32) -> (Vec<(u32, u32)>, u32, u32) {
    if sizes.is_empty() {
        return (Vec::new(), 0, 0);
    }

    // Sort items descending by height then width for optimal packing
    let mut indices: Vec<usize> = (0..sizes.len()).collect();
    indices.sort_by(|&a, &b| {
        let (wa, ha) = sizes[a];
        let (wb, hb) = sizes[b];
        (hb, wb).cmp(&(ha, wa)).reverse()
    });

    let total_area: u64 = sizes
        .iter()
        .map(|&(w, h)| (w + spacing) as u64 * (h + spacing) as u64)
        .sum();
    let mut est_w = ((total_area as f64).sqrt().ceil() as u32)
        .max(sizes.iter().map(|s| s.0).max().unwrap_or(0));
    est_w = est_w.next_power_of_two().max(16);

    let mut sheet_w = est_w;
    let mut sheet_h = est_w;

    let mut placed_positions = vec![(0, 0); sizes.len()];

    // Expand search area until all fit
    loop {
        let mut packer = MaxRectsPacker::new(sheet_w, sheet_h);
        let mut success = true;

        for &idx in &indices {
            let (w, h) = sizes[idx];
            let cell_w = w + spacing;
            let cell_h = h + spacing;

            if let Some(r) = packer.insert(cell_w, cell_h) {
                placed_positions[idx] = (border + r.x, border + r.y);
            } else {
                success = false;
                break;
            }
        }

        if success {
            break;
        }

        if sheet_w <= sheet_h {
            sheet_w = (sheet_w * 2).max(32);
        } else {
            sheet_h = (sheet_h * 2).max(32);
        }
    }

    // Compute bounding box
    let mut max_x = 0;
    let mut max_y = 0;
    for (i, &(w, h)) in sizes.iter().enumerate() {
        let (px, py) = placed_positions[i];
        max_x = max_x.max(px + w);
        max_y = max_y.max(py + h);
    }

    let final_w = max_x + border;
    let final_h = max_y + border;

    (placed_positions, final_w, final_h)
}

// ---------------------------------------------------------------------------
// Layout Positioning Engines
// ---------------------------------------------------------------------------

struct FrameCell {
    frame_index: usize,
    tag_name: Option<String>,
    duration_ms: u32,
    buffer: PixelBuffer,
    trim_offset_x: i32,
    trim_offset_y: i32,
    source_w: u32,
    source_h: u32,
}

/// Computes placement coordinates for each cell based on the configured layout.
fn compute_layout_placements(
    cells: &[FrameCell],
    options: &SpriteSheetOptions,
) -> (Vec<(u32, u32)>, u32, u32) {
    if cells.is_empty() {
        return (Vec::new(), 0, 0);
    }

    let border = options.border;
    let spacing = options.spacing;
    let ext = options.extrusion;

    match options.layout {
        SheetLayout::HorizontalStrip => {
            let mut cur_x = border;
            let mut max_h = 0;
            let mut placements = Vec::with_capacity(cells.len());

            for (i, cell) in cells.iter().enumerate() {
                let cell_w = cell.buffer.width() + 2 * ext;
                let cell_h = cell.buffer.height() + 2 * ext;

                placements.push((cur_x, border));
                cur_x += cell_w;
                if i + 1 < cells.len() {
                    cur_x += spacing;
                }
                max_h = max_h.max(cell_h);
            }

            let sheet_w = cur_x + border;
            let sheet_h = max_h + 2 * border;
            (placements, sheet_w, sheet_h)
        }

        SheetLayout::VerticalStrip => {
            let mut cur_y = border;
            let mut max_w = 0;
            let mut placements = Vec::with_capacity(cells.len());

            for (i, cell) in cells.iter().enumerate() {
                let cell_w = cell.buffer.width() + 2 * ext;
                let cell_h = cell.buffer.height() + 2 * ext;

                placements.push((border, cur_y));
                cur_y += cell_h;
                if i + 1 < cells.len() {
                    cur_y += spacing;
                }
                max_w = max_w.max(cell_w);
            }

            let sheet_w = max_w + 2 * border;
            let sheet_h = cur_y + border;
            (placements, sheet_w, sheet_h)
        }

        SheetLayout::Grid => {
            let cols = options.columns.max(1) as usize;
            let mut col_widths = vec![0u32; cols];
            let num_rows = cells.len().div_ceil(cols);
            let mut row_heights = vec![0u32; num_rows];

            // 1. Determine column widths and row heights
            for (idx, cell) in cells.iter().enumerate() {
                let c = idx % cols;
                let r = idx / cols;
                let cell_w = cell.buffer.width() + 2 * ext;
                let cell_h = cell.buffer.height() + 2 * ext;

                col_widths[c] = col_widths[c].max(cell_w);
                row_heights[r] = row_heights[r].max(cell_h);
            }

            // 2. Compute cumulative column X offsets
            let mut col_x = vec![border; cols];
            for c in 1..cols {
                col_x[c] = col_x[c - 1] + col_widths[c - 1] + spacing;
            }

            // 3. Compute cumulative row Y offsets
            let mut row_y = vec![border; num_rows];
            for r in 1..num_rows {
                row_y[r] = row_y[r - 1] + row_heights[r - 1] + spacing;
            }

            let mut placements = Vec::with_capacity(cells.len());
            for idx in 0..cells.len() {
                let c = idx % cols;
                let r = idx / cols;
                placements.push((col_x[c], row_y[r]));
            }

            let total_w = if cols > 0 {
                col_x.last().copied().unwrap_or(border)
                    + col_widths.last().copied().unwrap_or(0)
                    + border
            } else {
                border * 2
            };

            let total_h = if num_rows > 0 {
                row_y.last().copied().unwrap_or(border)
                    + row_heights.last().copied().unwrap_or(0)
                    + border
            } else {
                border * 2
            };

            (placements, total_w, total_h)
        }

        SheetLayout::Packed => {
            let sizes: Vec<(u32, u32)> = cells
                .iter()
                .map(|c| (c.buffer.width() + 2 * ext, c.buffer.height() + 2 * ext))
                .collect();
            pack_rectangles(&sizes, spacing, border)
        }
    }
}

// ---------------------------------------------------------------------------
// Main Sprite Sheet Generation Pipeline
// ---------------------------------------------------------------------------

/// Generates a sprite sheet in-memory from an animation document and configuration options.
pub fn generate_sprite_sheet(
    doc: &AnimationDocument,
    options: &SpriteSheetOptions,
    image_filename: &str,
) -> Result<SpriteSheetResult, SpriteSheetExportError> {
    let frame_indices = resolve_frame_indices(doc, &options.frame_range);
    if frame_indices.is_empty() {
        return Err(SpriteSheetExportError::NoFramesToExport);
    }

    // 1. Frame ordering: ByFrameIndex or ByTag
    let ordered_indices = match options.ordering {
        FrameOrdering::ByFrameIndex => frame_indices,
        FrameOrdering::ByTag => {
            let mut sorted = Vec::new();
            let mut visited = std::collections::HashSet::new();

            for tag in &doc.tags {
                for &idx in &frame_indices {
                    if idx >= tag.from_frame && idx <= tag.to_frame && visited.insert(idx) {
                        sorted.push(idx);
                    }
                }
            }
            // Append any remaining untagged frames
            for &idx in &frame_indices {
                if visited.insert(idx) {
                    sorted.push(idx);
                }
            }
            sorted
        }
    };

    // 2. Pre-render raw buffers for union trim if needed
    let mut raw_buffers = Vec::with_capacity(ordered_indices.len());
    for &idx in &ordered_indices {
        let buf = match &options.layers {
            LayerExportScope::SingleLayer(lid) => render_frame_for_layer(doc, idx, *lid),
            _ => doc.composite_frame(idx),
        };
        raw_buffers.push(buf);
    }

    let union_rect = match options.trim {
        TrimMode::Union => {
            let rect = compute_union_trim_rect(&raw_buffers);
            rect.or(Some((0, 0, 1.max(doc.width), 1.max(doc.height))))
        }
        _ => None,
    };

    // 3. Process individual frame cells (trim, scale, padding, transparency)
    let mut cells = Vec::with_capacity(ordered_indices.len());
    let source_w = doc.width * options.scale.max(1);
    let source_h = doc.height * options.scale.max(1);

    for (&frame_idx, raw_buf) in ordered_indices.iter().zip(raw_buffers.iter()) {
        let trim_rect = match options.trim {
            TrimMode::None => None,
            TrimMode::Union => union_rect,
            TrimMode::PerFrame => raw_buf
                .bounding_box_non_transparent()
                .or(Some((0, 0, 1, 1))),
        };

        let (processed_buf, offset_x, offset_y) = prepare_exported_buffer(
            raw_buf,
            trim_rect,
            options.padding,
            options.scale,
            &options.transparency,
        );

        let duration_ms = doc
            .frames
            .get(frame_idx)
            .map(|f| f.duration_ms)
            .unwrap_or(83);

        let tag_name = doc
            .tags
            .iter()
            .find(|t| frame_idx >= t.from_frame && frame_idx <= t.to_frame)
            .map(|t| t.name.clone());

        cells.push(FrameCell {
            frame_index: frame_idx,
            tag_name,
            duration_ms,
            buffer: processed_buf,
            trim_offset_x: offset_x * options.scale.max(1) as i32,
            trim_offset_y: offset_y * options.scale.max(1) as i32,
            source_w,
            source_h,
        });
    }

    // 4. Compute layout placements
    let (placements, mut final_sheet_w, mut final_sheet_h) =
        compute_layout_placements(&cells, options);

    // 5. Apply sheet sizing constraints
    match options.size_mode {
        SheetSizeMode::Auto => {}
        SheetSizeMode::PowerOfTwo => {
            final_sheet_w = final_sheet_w.next_power_of_two().max(16);
            final_sheet_h = final_sheet_h.next_power_of_two().max(16);
        }
        SheetSizeMode::MaxSize { max_w, max_h } => {
            if final_sheet_w > max_w || final_sheet_h > max_h {
                return Err(SpriteSheetExportError::MaxSizeExceeded(
                    final_sheet_w,
                    final_sheet_h,
                    max_w,
                    max_h,
                ));
            }
        }
    }

    // 6. Allocate sheet buffer and render extruded cells onto canvas
    let bg_color = match options.transparency {
        TransparencyOption::KeepTransparency => ColorRgba::TRANSPARENT,
        TransparencyOption::Flatten(c) => c,
    };
    let mut sheet_buffer = PixelBuffer::new(final_sheet_w, final_sheet_h, bg_color);

    let mut placed_frame_infos = Vec::with_capacity(cells.len());
    let ext = options.extrusion;

    for (cell, (px, py)) in cells.into_iter().zip(placements) {
        let content_w = cell.buffer.width();
        let content_h = cell.buffer.height();

        // Extrude if enabled
        let extruded_buf = if ext > 0 {
            extrude_cell(&cell.buffer, ext)
        } else {
            cell.buffer
        };

        sheet_buffer.blend_buffer(&extruded_buf, px as i32, py as i32);

        // Content top-left is positioned at (px + ext, py + ext)
        let content_x = px + ext;
        let content_y = py + ext;

        placed_frame_infos.push(PlacedFrameInfo {
            frame_index: cell.frame_index,
            tag_name: cell.tag_name,
            x: content_x,
            y: content_y,
            w: content_w,
            h: content_h,
            duration_ms: cell.duration_ms,
            trim_offset_x: cell.trim_offset_x,
            trim_offset_y: cell.trim_offset_y,
            source_w: cell.source_w,
            source_h: cell.source_h,
        });
    }

    // 7. Flatten entire sheet if solid background requested
    let final_sheet = match options.transparency {
        TransparencyOption::KeepTransparency => sheet_buffer,
        TransparencyOption::Flatten(bg) => sheet_buffer.flatten_on_color(bg),
    };

    // 8. Generate metadata (JSON / CSV)
    let json_metadata = if matches!(options.metadata_format, MetadataFormat::Json) {
        let json_doc = JsonSheetDoc {
            version: 1,
            image: image_filename.to_string(),
            size: JsonSheetSize {
                w: final_sheet_w,
                h: final_sheet_h,
            },
            frames: placed_frame_infos
                .iter()
                .map(|f| JsonSheetFrame {
                    index: f.frame_index,
                    x: f.x,
                    y: f.y,
                    w: f.w,
                    h: f.h,
                    duration_ms: f.duration_ms,
                    trim_offset: JsonSheetPoint {
                        x: f.trim_offset_x,
                        y: f.trim_offset_y,
                    },
                    source_size: JsonSheetSize {
                        w: f.source_w,
                        h: f.source_h,
                    },
                })
                .collect(),
            tags: doc
                .tags
                .iter()
                .map(|t| JsonSheetTag {
                    name: t.name.clone(),
                    from: t.from_frame,
                    to: t.to_frame,
                })
                .collect(),
        };
        Some(serde_json::to_string_pretty(&json_doc)?)
    } else {
        None
    };

    let csv_metadata = if matches!(options.metadata_format, MetadataFormat::Csv) {
        let mut csv =
            String::from("index,x,y,w,h,duration_ms,offset_x,offset_y,source_w,source_h,tag\n");
        for f in &placed_frame_infos {
            let tag = f.tag_name.as_deref().unwrap_or("");
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                f.frame_index,
                f.x,
                f.y,
                f.w,
                f.h,
                f.duration_ms,
                f.trim_offset_x,
                f.trim_offset_y,
                f.source_w,
                f.source_h,
                tag
            ));
        }
        Some(csv)
    } else {
        None
    };

    Ok(SpriteSheetResult {
        sheet_buffer: final_sheet,
        frames: placed_frame_infos,
        tags: doc.tags.clone(),
        sheet_width: final_sheet_w,
        sheet_height: final_sheet_h,
        json_metadata,
        csv_metadata,
    })
}

/// Exports a sprite sheet image and optional metadata file to disk.
pub fn export_sprite_sheet_to_disk(
    doc: &AnimationDocument,
    options: &SpriteSheetOptions,
    destination_png_path: &Path,
) -> Result<SpriteSheetResult, SpriteSheetExportError> {
    if let Some(parent) = destination_png_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let image_filename = destination_png_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("sheet.png");

    let result = generate_sprite_sheet(doc, options, image_filename)?;

    // 1. Save PNG
    save_to_path(
        &result.sheet_buffer,
        destination_png_path,
        ImageFileFormat::Png,
    )?;

    // 2. Save metadata
    match options.metadata_format {
        MetadataFormat::Json => {
            if let Some(ref json_str) = result.json_metadata {
                let json_path = destination_png_path.with_extension("json");
                std::fs::write(&json_path, json_str)?;
            }
        }
        MetadataFormat::Csv => {
            if let Some(ref csv_str) = result.csv_metadata {
                let csv_path = destination_png_path.with_extension("csv");
                std::fs::write(&csv_path, csv_str)?;
            }
        }
        MetadataFormat::None => {}
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frames::{Cel, Frame, LayerMetadata};
    use crate::layers::LayerId;
    use crate::palette::{BLUE, GREEN, RED, WHITE};

    fn create_test_sheet_doc() -> AnimationDocument {
        let mut doc = AnimationDocument::new_transparent(8, 8);
        let layer1 = LayerId(1);
        let layer2 = LayerId(2);
        doc.layers.push(LayerMetadata::new(layer2, "Effects"));

        // Frame 0: Red 4x4 block at (1, 1)
        let mut cel0 = Cel::new(8, 8);
        for y in 1..5 {
            for x in 1..5 {
                cel0.buffer.set_pixel(x, y, RED);
            }
        }
        doc.frames[0].set_cel(layer1, cel0);

        // Frame 1: Blue 4x4 block at (3, 3)
        let mut frame1 = Frame::new(120);
        let mut cel1 = Cel::new(8, 8);
        for y in 3..7 {
            for x in 3..7 {
                cel1.buffer.set_pixel(x, y, BLUE);
            }
        }
        frame1.set_cel(layer1, cel1);
        doc.frames.push(frame1);

        // Frame 2: Green 2x2 block at (0, 0)
        let mut frame2 = Frame::new(150);
        let mut cel2 = Cel::new(8, 8);
        for y in 0..2 {
            for x in 0..2 {
                cel2.buffer.set_pixel(x, y, GREEN);
            }
        }
        frame2.set_cel(layer1, cel2);
        doc.frames.push(frame2);

        doc.tags.push(AnimTag::new("intro", 0, 1));
        doc
    }

    #[test]
    fn test_extrude_cell() {
        let mut buf = PixelBuffer::new(2, 2, ColorRgba::TRANSPARENT);
        buf.set_pixel(0, 0, RED);
        buf.set_pixel(1, 0, GREEN);
        buf.set_pixel(0, 1, BLUE);
        buf.set_pixel(1, 1, WHITE);

        let extruded = extrude_cell(&buf, 1);
        assert_eq!(extruded.width(), 4);
        assert_eq!(extruded.height(), 4);

        // Check center content
        assert_eq!(extruded.get_pixel(1, 1), Some(RED));
        assert_eq!(extruded.get_pixel(2, 1), Some(GREEN));
        assert_eq!(extruded.get_pixel(1, 2), Some(BLUE));
        assert_eq!(extruded.get_pixel(2, 2), Some(WHITE));

        // Check extruded edges & corners
        assert_eq!(extruded.get_pixel(0, 0), Some(RED)); // Top-Left corner
        assert_eq!(extruded.get_pixel(1, 0), Some(RED)); // Top edge of (0,0)
        assert_eq!(extruded.get_pixel(2, 0), Some(GREEN)); // Top edge of (1,0)
        assert_eq!(extruded.get_pixel(3, 0), Some(GREEN)); // Top-Right corner
        assert_eq!(extruded.get_pixel(0, 1), Some(RED)); // Left edge of (0,0)
        assert_eq!(extruded.get_pixel(3, 2), Some(WHITE)); // Right edge of (1,1)
    }

    #[test]
    fn test_horizontal_strip_layout() {
        let doc = create_test_sheet_doc();
        let opts = SpriteSheetOptions {
            layout: SheetLayout::HorizontalStrip,
            columns: 4,
            spacing: 2,
            border: 1,
            extrusion: 0,
            size_mode: SheetSizeMode::Auto,
            scale: 1,
            trim: TrimMode::None,
            ..Default::default()
        };

        let res = generate_sprite_sheet(&doc, &opts, "test_h.png").unwrap();
        // 3 frames of 8px width, with 2px spacing and 1px border on each side:
        // Width: 1 + 8 + 2 + 8 + 2 + 8 + 1 = 30
        // Height: 8 + 2*1 = 10
        assert_eq!(res.sheet_width, 30);
        assert_eq!(res.sheet_height, 10);
        assert_eq!(res.frames.len(), 3);
        assert_eq!(res.frames[0].x, 1);
        assert_eq!(res.frames[1].x, 11);
        assert_eq!(res.frames[2].x, 21);
    }

    #[test]
    fn test_vertical_strip_layout() {
        let doc = create_test_sheet_doc();
        let opts = SpriteSheetOptions {
            layout: SheetLayout::VerticalStrip,
            spacing: 3,
            border: 2,
            extrusion: 0,
            size_mode: SheetSizeMode::Auto,
            scale: 1,
            trim: TrimMode::None,
            ..Default::default()
        };

        let res = generate_sprite_sheet(&doc, &opts, "test_v.png").unwrap();
        // Width: 8 + 2*2 = 12
        // Height: 2 + 8 + 3 + 8 + 3 + 8 + 2 = 34
        assert_eq!(res.sheet_width, 12);
        assert_eq!(res.sheet_height, 34);
        assert_eq!(res.frames[0].y, 2);
        assert_eq!(res.frames[1].y, 13);
        assert_eq!(res.frames[2].y, 24);
    }

    #[test]
    fn test_grid_layout_and_pot() {
        let doc = create_test_sheet_doc();
        let opts = SpriteSheetOptions {
            layout: SheetLayout::Grid,
            columns: 2,
            spacing: 0,
            border: 0,
            extrusion: 0,
            size_mode: SheetSizeMode::PowerOfTwo,
            scale: 1,
            trim: TrimMode::None,
            ..Default::default()
        };

        let res = generate_sprite_sheet(&doc, &opts, "test_grid.png").unwrap();
        // Grid 2 cols x 2 rows of 8x8: 16x16 -> next POT is 16x16
        assert_eq!(res.sheet_width, 16);
        assert_eq!(res.sheet_height, 16);
        assert_eq!(res.frames[0].x, 0);
        assert_eq!(res.frames[0].y, 0);
        assert_eq!(res.frames[1].x, 8);
        assert_eq!(res.frames[1].y, 0);
        assert_eq!(res.frames[2].x, 0);
        assert_eq!(res.frames[2].y, 8);
    }

    #[test]
    fn test_json_and_csv_metadata_generation() {
        let doc = create_test_sheet_doc();
        let opts = SpriteSheetOptions {
            layout: SheetLayout::Grid,
            columns: 2,
            metadata_format: MetadataFormat::Json,
            ..Default::default()
        };

        let res = generate_sprite_sheet(&doc, &opts, "hero.png").unwrap();
        assert!(res.json_metadata.is_some());
        let json_str = res.json_metadata.as_ref().unwrap();

        let parsed: JsonSheetDoc = serde_json::from_str(json_str).expect("Valid JSON");
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.image, "hero.png");
        assert_eq!(parsed.frames.len(), 3);
        assert_eq!(parsed.tags.len(), 1);
        assert_eq!(parsed.tags[0].name, "intro");
        assert_eq!(parsed.tags[0].from, 0);
        assert_eq!(parsed.tags[0].to, 1);
    }
}

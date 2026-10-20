//! Pattern Fill inspired by Kid Pix.
//!
//! Pure Rust implementation (#![forbid(unsafe_code)]) with deterministic algorithms,
//! full RGBA support, and zero UI dependencies.

use crate::geometry::{Point, Rect};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use crate::tools::fill::color_matches;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FillPattern {
    #[default]
    Checkerboard,
    PolkaDots,
    HorizontalStripes,
    VerticalStripes,
    DiagonalStripes,
    Bricks,
    Houndstooth,
    Scales,
}

impl FillPattern {
    pub const ALL: [FillPattern; 8] = [
        FillPattern::Checkerboard,
        FillPattern::PolkaDots,
        FillPattern::HorizontalStripes,
        FillPattern::VerticalStripes,
        FillPattern::DiagonalStripes,
        FillPattern::Bricks,
        FillPattern::Houndstooth,
        FillPattern::Scales,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            FillPattern::Checkerboard => "Checkerboard / Damier",
            FillPattern::PolkaDots => "Polka Dots / Pois",
            FillPattern::HorizontalStripes => "Horizontal Stripes / Lignes H",
            FillPattern::VerticalStripes => "Vertical Stripes / Lignes V",
            FillPattern::DiagonalStripes => "Diagonal Stripes / Diagonales",
            FillPattern::Bricks => "Bricks / Briques",
            FillPattern::Houndstooth => "Houndstooth / Pied-de-poule",
            FillPattern::Scales => "Scales / Écailles",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FillPattern::Checkerboard => "🏁",
            FillPattern::PolkaDots => "🔘",
            FillPattern::HorizontalStripes => "═",
            FillPattern::VerticalStripes => "║",
            FillPattern::DiagonalStripes => "▨",
            FillPattern::Bricks => "🧱",
            FillPattern::Houndstooth => "▦",
            FillPattern::Scales => "🌊",
        }
    }

    /// Evaluates if coordinate `(x, y)` corresponds to foreground or background in the pattern.
    #[inline]
    pub fn sample(&self, x: i32, y: i32, fg: ColorRgba, bg: ColorRgba) -> ColorRgba {
        let is_fg = match self {
            FillPattern::Checkerboard => {
                let cell = 4;
                ((x.div_euclid(cell) + y.div_euclid(cell)) % 2) == 0
            }
            FillPattern::PolkaDots => {
                let mx = x.rem_euclid(6);
                let my = y.rem_euclid(6);
                (2..=3).contains(&mx) && (2..=3).contains(&my)
            }
            FillPattern::HorizontalStripes => (y.rem_euclid(4)) < 2,
            FillPattern::VerticalStripes => (x.rem_euclid(4)) < 2,
            FillPattern::DiagonalStripes => (x.wrapping_add(y).rem_euclid(6)) < 3,
            FillPattern::Bricks => {
                let row = y.div_euclid(4);
                let offset = if row % 2 == 0 { 0 } else { 4 };
                let col = (x + offset).rem_euclid(8);
                (y.rem_euclid(4) != 0) && (col != 0)
            }
            FillPattern::Houndstooth => {
                let bx = x.rem_euclid(8);
                let by = y.rem_euclid(8);
                (bx < 4 && by < 4) || (bx >= 4 && by >= 4 && (bx + by) % 2 == 0)
            }
            FillPattern::Scales => {
                let row = y.div_euclid(6);
                let offset = if row % 2 == 0 { 0 } else { 4 };
                let cx = (x + offset).rem_euclid(8);
                let cy = y.rem_euclid(6);
                (cx - 4) * (cx - 4) + cy * cy <= 16
            }
        };

        if is_fg {
            fg
        } else {
            bg
        }
    }

    /// Evaluates if coordinate `(x, y)` corresponds to foreground or background in the pattern with options.
    #[inline]
    pub fn sample_with_options(
        &self,
        x: i32,
        y: i32,
        options: &PatternFillOptions,
        fg: ColorRgba,
        bg: ColorRgba,
    ) -> ColorRgba {
        let scale = options.scale.max(1) as i32;
        let nx = (x - options.origin.x).div_euclid(scale);
        let ny = (y - options.origin.y).div_euclid(scale);
        self.sample(nx, ny, fg, bg)
    }
}

/// Precision controls for procedural pattern fills.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternFillOptions {
    /// Pattern magnification factor (1 = 1x native micro-pattern, 2 = 2x, 4 = 4x, etc.).
    pub scale: u32,
    /// Origin coordinate (X, Y) to align pattern phase/seams with pixel art edges.
    pub origin: Point,
    /// Optional bounding box clip. Pixels outside are neither sampled nor modified.
    pub clip_rect: Option<Rect>,
}

impl Default for PatternFillOptions {
    fn default() -> Self {
        Self {
            scale: 1,
            origin: Point::new(0, 0),
            clip_rect: None,
        }
    }
}

/// Flood fills an area with a retro pattern with fine-grained options (scale, origin, clip rect).
pub fn pattern_flood_fill_with_options(
    buffer: &mut PixelBuffer,
    start: Point,
    pattern: FillPattern,
    tolerance: u8,
    fg: ColorRgba,
    bg: ColorRgba,
    options: &PatternFillOptions,
) {
    let width = buffer.width() as i32;
    let height = buffer.height() as i32;

    if start.x < 0 || start.x >= width || start.y < 0 || start.y >= height {
        return;
    }

    if let Some(clip) = options.clip_rect {
        if !clip.contains(start) {
            return;
        }
    }

    let target_color = match buffer.get_pixel_i32(start.x, start.y) {
        Some(c) => c,
        None => return,
    };

    let mut visited = vec![false; (width * height) as usize];
    let mut queue = VecDeque::new();
    queue.push_back((start.x, start.y));

    let min_x = options.clip_rect.map(|r| r.min.x.max(0)).unwrap_or(0);
    let max_x = options
        .clip_rect
        .map(|r| (r.max.x).min(width - 1))
        .unwrap_or(width - 1);
    let min_y = options.clip_rect.map(|r| r.min.y.max(0)).unwrap_or(0);
    let max_y = options
        .clip_rect
        .map(|r| (r.max.y).min(height - 1))
        .unwrap_or(height - 1);

    while let Some((cx, cy)) = queue.pop_front() {
        if cx < min_x || cx > max_x || cy < min_y || cy > max_y {
            continue;
        }

        let idx = (cy * width + cx) as usize;
        if visited[idx] {
            continue;
        }

        if let Some(c) = buffer.get_pixel_i32(cx, cy) {
            if !color_matches(c, target_color, tolerance) {
                continue;
            }
        } else {
            continue;
        }

        // Find left and right bounds of the span
        let mut left = cx;
        while left > min_x {
            let next_idx = (cy * width + (left - 1)) as usize;
            if visited[next_idx] {
                break;
            }
            if let Some(c) = buffer.get_pixel_i32(left - 1, cy) {
                if color_matches(c, target_color, tolerance) {
                    left -= 1;
                    continue;
                }
            }
            break;
        }

        let mut right = cx;
        while right < max_x {
            let next_idx = (cy * width + (right + 1)) as usize;
            if visited[next_idx] {
                break;
            }
            if let Some(c) = buffer.get_pixel_i32(right + 1, cy) {
                if color_matches(c, target_color, tolerance) {
                    right += 1;
                    continue;
                }
            }
            break;
        }

        // Fill the span with the pattern color at (x, cy)
        for x in left..=right {
            let p_idx = (cy * width + x) as usize;
            visited[p_idx] = true;
            let col = pattern.sample_with_options(x, cy, options, fg, bg);
            buffer.set_pixel_i32(x, cy, col);
        }

        // Scan row above
        if cy > min_y {
            let mut x = left;
            while x <= right {
                let up_idx = ((cy - 1) * width + x) as usize;
                if !visited[up_idx] {
                    if let Some(c) = buffer.get_pixel_i32(x, cy - 1) {
                        if color_matches(c, target_color, tolerance) {
                            queue.push_back((x, cy - 1));
                            while x <= right {
                                if let Some(c2) = buffer.get_pixel_i32(x, cy - 1) {
                                    if !color_matches(c2, target_color, tolerance) {
                                        break;
                                    }
                                }
                                x += 1;
                            }
                        }
                    }
                }
                x += 1;
            }
        }

        // Scan row below
        if cy < max_y {
            let mut x = left;
            while x <= right {
                let down_idx = ((cy + 1) * width + x) as usize;
                if !visited[down_idx] {
                    if let Some(c) = buffer.get_pixel_i32(x, cy + 1) {
                        if color_matches(c, target_color, tolerance) {
                            queue.push_back((x, cy + 1));
                            while x <= right {
                                if let Some(c2) = buffer.get_pixel_i32(x, cy + 1) {
                                    if !color_matches(c2, target_color, tolerance) {
                                        break;
                                    }
                                }
                                x += 1;
                            }
                        }
                    }
                }
                x += 1;
            }
        }
    }
}

/// Flood fills an area with a retro pattern starting at `start`.
pub fn pattern_flood_fill(
    buffer: &mut PixelBuffer,
    start: Point,
    pattern: FillPattern,
    tolerance: u8,
    fg: ColorRgba,
    bg: ColorRgba,
) {
    pattern_flood_fill_with_options(
        buffer,
        start,
        pattern,
        tolerance,
        fg,
        bg,
        &PatternFillOptions::default(),
    );
}

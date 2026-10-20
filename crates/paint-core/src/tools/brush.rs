//! Brush tool with 12 classic Windows XP tips.

use crate::geometry::{bresenham_line, Point};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BrushShape {
    #[default]
    RoundLarge, // ~7px diameter circle
    RoundMedium,        // ~4px diameter circle
    RoundSmall,         // ~1px diameter point
    SquareLarge,        // 8x8 square
    SquareMedium,       // 5x5 square
    SquareSmall,        // 2x2 square
    ForwardSlashLarge,  // 9px forward diagonal
    ForwardSlashMedium, // 5px forward diagonal
    ForwardSlashSmall,  // 3px forward diagonal
    BackslashLarge,     // 9px backward diagonal
    BackslashMedium,    // 5px backward diagonal
    BackslashSmall,     // 3px backward diagonal
}

impl BrushShape {
    pub const ALL: [BrushShape; 12] = [
        BrushShape::RoundLarge,
        BrushShape::RoundMedium,
        BrushShape::RoundSmall,
        BrushShape::SquareLarge,
        BrushShape::SquareMedium,
        BrushShape::SquareSmall,
        BrushShape::ForwardSlashLarge,
        BrushShape::ForwardSlashMedium,
        BrushShape::ForwardSlashSmall,
        BrushShape::BackslashLarge,
        BrushShape::BackslashMedium,
        BrushShape::BackslashSmall,
    ];

    /// Returns relative pixel offsets `(dx, dy)` from center for this brush tip.
    pub fn offsets(&self) -> &'static [(i32, i32)] {
        match self {
            BrushShape::RoundLarge => &[
                (-1, -3),
                (0, -3),
                (1, -3),
                (-2, -2),
                (-1, -2),
                (0, -2),
                (1, -2),
                (2, -2),
                (-3, -1),
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (3, -1),
                (-3, 0),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (-3, 1),
                (-2, 1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (-2, 2),
                (-1, 2),
                (0, 2),
                (1, 2),
                (2, 2),
                (-1, 3),
                (0, 3),
                (1, 3),
            ],
            BrushShape::RoundMedium => &[
                (-1, -2),
                (0, -2),
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
            ],
            BrushShape::RoundSmall => &[(0, 0)],
            BrushShape::SquareLarge => &[
                (-3, -3),
                (-2, -3),
                (-1, -3),
                (0, -3),
                (1, -3),
                (2, -3),
                (3, -3),
                (4, -3),
                (-3, -2),
                (-2, -2),
                (-1, -2),
                (0, -2),
                (1, -2),
                (2, -2),
                (3, -2),
                (4, -2),
                (-3, -1),
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (3, -1),
                (4, -1),
                (-3, 0),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (-3, 1),
                (-2, 1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (4, 1),
                (-3, 2),
                (-2, 2),
                (-1, 2),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
                (4, 2),
                (-3, 3),
                (-2, 3),
                (-1, 3),
                (0, 3),
                (1, 3),
                (2, 3),
                (3, 3),
                (4, 3),
                (-3, 4),
                (-2, 4),
                (-1, 4),
                (0, 4),
                (1, 4),
                (2, 4),
                (3, 4),
                (4, 4),
            ],
            BrushShape::SquareMedium => &[
                (-2, -2),
                (-1, -2),
                (0, -2),
                (1, -2),
                (2, -2),
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (-2, 1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (2, 1),
                (-2, 2),
                (-1, 2),
                (0, 2),
                (1, 2),
                (2, 2),
            ],
            BrushShape::SquareSmall => &[(0, 0), (1, 0), (0, 1), (1, 1)],
            BrushShape::ForwardSlashLarge => &[
                (4, -4),
                (3, -3),
                (2, -2),
                (1, -1),
                (0, 0),
                (-1, 1),
                (-2, 2),
                (-3, 3),
                (-4, 4),
            ],
            BrushShape::ForwardSlashMedium => &[(2, -2), (1, -1), (0, 0), (-1, 1), (-2, 2)],
            BrushShape::ForwardSlashSmall => &[(1, -1), (0, 0), (-1, 1)],
            BrushShape::BackslashLarge => &[
                (-4, -4),
                (-3, -3),
                (-2, -2),
                (-1, -1),
                (0, 0),
                (1, 1),
                (2, 2),
                (3, 3),
                (4, 4),
            ],
            BrushShape::BackslashMedium => &[(-2, -2), (-1, -1), (0, 0), (1, 1), (2, 2)],
            BrushShape::BackslashSmall => &[(-1, -1), (0, 0), (1, 1)],
        }
    }
}

/// Applies a single brush stamp at `center`.
pub fn apply_brush_stamp(
    buffer: &mut PixelBuffer,
    center: Point,
    shape: BrushShape,
    color: ColorRgba,
) {
    for &(dx, dy) in shape.offsets() {
        buffer.set_pixel_i32(center.x + dx, center.y + dy, color);
    }
}

/// Draws an interpolated brush stroke between `p0` and `p1`.
pub fn draw_brush_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    shape: BrushShape,
    color: ColorRgba,
) {
    for pt in bresenham_line(p0, p1) {
        apply_brush_stamp(buffer, pt, shape, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, WHITE};

    #[test]
    fn test_all_12_brush_shapes() {
        let mut buffer = PixelBuffer::new(50, 50, WHITE);
        for (i, &shape) in BrushShape::ALL.iter().enumerate() {
            let x = 5 + (i as i32 % 6) * 7;
            let y = 5 + (i as i32 / 6) * 15;
            apply_brush_stamp(&mut buffer, Point::new(x, y), shape, BLACK);
            assert_eq!(buffer.get_pixel_i32(x, y), Some(BLACK));
            assert!(!shape.offsets().is_empty());
        }
    }

    #[test]
    fn test_brush_stroke_interpolation() {
        let mut buffer = PixelBuffer::new(30, 30, WHITE);
        draw_brush_stroke(
            &mut buffer,
            Point::new(5, 5),
            Point::new(20, 5),
            BrushShape::SquareSmall,
            BLACK,
        );
        for x in 5..=20 {
            assert_eq!(buffer.get_pixel_i32(x, 5), Some(BLACK));
            assert_eq!(buffer.get_pixel_i32(x, 6), Some(BLACK));
        }
    }
}

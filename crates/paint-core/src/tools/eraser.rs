//! Eraser and Color Eraser tool implementations.

use crate::geometry::{bresenham_line, Point};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EraserSize {
    Small = 4, // 4x4
    #[default]
    Medium = 6, // 6x6
    Large = 8, // 8x8
    XLarge = 10, // 10x10
}

impl EraserSize {
    pub const ALL: [EraserSize; 4] = [
        EraserSize::Small,
        EraserSize::Medium,
        EraserSize::Large,
        EraserSize::XLarge,
    ];

    pub fn size_in_pixels(&self) -> i32 {
        *self as i32
    }
}

/// Applies standard eraser stamp (square of size NxN filled with `bg_color`).
pub fn apply_eraser_stamp(
    buffer: &mut PixelBuffer,
    center: Point,
    size: EraserSize,
    bg_color: ColorRgba,
) {
    let s = size.size_in_pixels();
    let half = s / 2;
    let min_x = center.x - half;
    let min_y = center.y - half;

    for dy in 0..s {
        for dx in 0..s {
            buffer.set_pixel_i32(min_x + dx, min_y + dy, bg_color);
        }
    }
}

/// Applies color eraser stamp (square of size NxN replacing only `target_fg` with `bg_color`).
pub fn apply_color_eraser_stamp(
    buffer: &mut PixelBuffer,
    center: Point,
    size: EraserSize,
    target_fg: ColorRgba,
    bg_color: ColorRgba,
) {
    let s = size.size_in_pixels();
    let half = s / 2;
    let min_x = center.x - half;
    let min_y = center.y - half;

    for dy in 0..s {
        for dx in 0..s {
            let px = min_x + dx;
            let py = min_y + dy;
            if let Some(c) = buffer.get_pixel_i32(px, py) {
                if c == target_fg {
                    buffer.set_pixel_i32(px, py, bg_color);
                }
            }
        }
    }
}

/// Draws an interpolated normal eraser stroke between `p0` and `p1`.
pub fn draw_eraser_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: EraserSize,
    bg_color: ColorRgba,
) {
    for pt in bresenham_line(p0, p1) {
        apply_eraser_stamp(buffer, pt, size, bg_color);
    }
}

/// Draws an interpolated color eraser stroke between `p0` and `p1`.
pub fn draw_color_eraser_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: EraserSize,
    target_fg: ColorRgba,
    bg_color: ColorRgba,
) {
    for pt in bresenham_line(p0, p1) {
        apply_color_eraser_stamp(buffer, pt, size, target_fg, bg_color);
    }
}

/// Draws an interpolated transparent eraser stroke between `p0` and `p1` (sets alpha to 0).
pub fn draw_transparent_eraser_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: EraserSize,
) {
    draw_eraser_stroke(buffer, p0, p1, size, ColorRgba::rgba(0, 0, 0, 0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, RED, WHITE};

    #[test]
    fn test_eraser_normal() {
        let mut buffer = PixelBuffer::new(20, 20, BLACK);
        draw_eraser_stroke(
            &mut buffer,
            Point::new(5, 5),
            Point::new(15, 5),
            EraserSize::Small,
            WHITE,
        );
        assert_eq!(buffer.get_pixel_i32(5, 5), Some(WHITE));
        assert_eq!(buffer.get_pixel_i32(15, 5), Some(WHITE));
    }

    #[test]
    fn test_eraser_transparent() {
        let mut buffer = PixelBuffer::new(20, 20, BLACK);
        draw_transparent_eraser_stroke(
            &mut buffer,
            Point::new(5, 5),
            Point::new(15, 5),
            EraserSize::Small,
        );
        assert_eq!(
            buffer.get_pixel_i32(5, 5),
            Some(ColorRgba::rgba(0, 0, 0, 0))
        );
        assert_eq!(
            buffer.get_pixel_i32(15, 5),
            Some(ColorRgba::rgba(0, 0, 0, 0))
        );
    }

    #[test]
    fn test_color_eraser() {
        let mut buffer = PixelBuffer::new(20, 20, BLACK);
        buffer.set_pixel_i32(5, 5, RED);
        buffer.set_pixel_i32(6, 5, BLACK);

        // Replace only RED with WHITE
        apply_color_eraser_stamp(&mut buffer, Point::new(5, 5), EraserSize::Small, RED, WHITE);

        assert_eq!(buffer.get_pixel_i32(5, 5), Some(WHITE));
        assert_eq!(buffer.get_pixel_i32(6, 5), Some(BLACK));
    }
}

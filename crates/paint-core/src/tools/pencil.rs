//! Pencil tool: 1-pixel freehand continuous drawing without anti-aliasing.

use crate::geometry::{bresenham_line, Point};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;

/// Draws a continuous 1-pixel line segment from `p0` to `p1` on `buffer`.
pub fn draw_pencil_stroke(buffer: &mut PixelBuffer, p0: Point, p1: Point, color: ColorRgba) {
    draw_pencil_stroke_sized(buffer, p0, p1, 1, color);
}

/// Draws a single 1-pixel dot at `p`.
pub fn draw_pencil_point(buffer: &mut PixelBuffer, p: Point, color: ColorRgba) {
    draw_pencil_point_sized(buffer, p, 1, color);
}

/// Draws a single square pixel-art point with dimension `size` at `p`.
pub fn draw_pencil_point_sized(buffer: &mut PixelBuffer, p: Point, size: u32, color: ColorRgba) {
    if size <= 1 {
        buffer.set_pixel_i32(p.x, p.y, color);
        return;
    }
    let half = (size / 2) as i32;
    let start_x = p.x - half;
    let start_y = p.y - half;
    for dy in 0..size as i32 {
        for dx in 0..size as i32 {
            buffer.set_pixel_i32(start_x + dx, start_y + dy, color);
        }
    }
}

/// Draws a continuous line segment from `p0` to `p1` on `buffer` with thickness `size`.
pub fn draw_pencil_stroke_sized(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: u32,
    color: ColorRgba,
) {
    if size <= 1 {
        for pt in bresenham_line(p0, p1) {
            buffer.set_pixel_i32(pt.x, pt.y, color);
        }
    } else {
        for pt in bresenham_line(p0, p1) {
            draw_pencil_point_sized(buffer, pt, size, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::BLACK;

    #[test]
    fn test_pencil_stroke() {
        let mut buffer = PixelBuffer::new(10, 10, crate::palette::WHITE);
        draw_pencil_stroke(&mut buffer, Point::new(1, 1), Point::new(3, 1), BLACK);

        assert_eq!(buffer.get_pixel_i32(1, 1), Some(BLACK));
        assert_eq!(buffer.get_pixel_i32(2, 1), Some(BLACK));
        assert_eq!(buffer.get_pixel_i32(3, 1), Some(BLACK));
        assert_eq!(buffer.get_pixel_i32(4, 1), Some(crate::palette::WHITE));
    }
}

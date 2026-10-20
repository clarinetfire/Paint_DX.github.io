//! Line tool: straight lines with selectable widths (1..=5 px) and 45-degree angle constraints.

use crate::geometry::{bresenham_line, snap_angle_45, Point};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;

/// Draws a straight line with the given thickness (1 to 5 pixels) from `p0` to `p1`.
pub fn draw_line(buffer: &mut PixelBuffer, p0: Point, p1: Point, width: u32, color: ColorRgba) {
    let w = width.clamp(1, 5) as i32;
    let points = bresenham_line(p0, p1);

    if w == 1 {
        for pt in points {
            buffer.set_pixel_i32(pt.x, pt.y, color);
        }
        return;
    }

    let min_offset = -((w - 1) / 2);
    let max_offset = min_offset + w;

    for pt in points {
        for dy in min_offset..max_offset {
            for dx in min_offset..max_offset {
                buffer.set_pixel_i32(pt.x + dx, pt.y + dy, color);
            }
        }
    }
}

/// Helper to compute end point when Shift is pressed (45-degree snapping).
pub fn constrained_line_end(start: Point, current: Point, is_shift_down: bool) -> Point {
    if is_shift_down {
        snap_angle_45(start, current)
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, WHITE};

    #[test]
    fn test_draw_line_1px() {
        let mut buffer = PixelBuffer::new(10, 10, WHITE);
        draw_line(&mut buffer, Point::new(0, 0), Point::new(4, 0), 1, BLACK);

        for x in 0..=4 {
            assert_eq!(buffer.get_pixel_i32(x, 0), Some(BLACK));
        }
        assert_eq!(buffer.get_pixel_i32(5, 0), Some(WHITE));
    }

    #[test]
    fn test_draw_line_3px() {
        let mut buffer = PixelBuffer::new(10, 10, WHITE);
        draw_line(&mut buffer, Point::new(5, 5), Point::new(5, 5), 3, BLACK);

        // 3x3 stamp centered at (5, 5) => x: 4..=6, y: 4..=6
        for y in 4..=6 {
            for x in 4..=6 {
                assert_eq!(buffer.get_pixel_i32(x, y), Some(BLACK));
            }
        }
        assert_eq!(buffer.get_pixel_i32(3, 5), Some(WHITE));
        assert_eq!(buffer.get_pixel_i32(7, 5), Some(WHITE));
    }
}

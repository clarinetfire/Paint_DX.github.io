//! Flood fill (Fill with Color / Paint Bucket) implementation.

use crate::geometry::Point;
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use std::collections::VecDeque;

/// Checks whether two colors match within a given channel tolerance.
#[inline]
pub fn color_matches(c1: ColorRgba, c2: ColorRgba, tolerance: u8) -> bool {
    if tolerance == 0 {
        c1 == c2
    } else {
        let dr = (c1.r as i32 - c2.r as i32).abs();
        let dg = (c1.g as i32 - c2.g as i32).abs();
        let db = (c1.b as i32 - c2.b as i32).abs();
        let da = (c1.a as i32 - c2.a as i32).abs();
        dr.max(dg).max(db).max(da) <= tolerance as i32
    }
}

/// Performs 4-connected flood fill starting at `start` with `fill_color` (exact color match).
pub fn flood_fill(buffer: &mut PixelBuffer, start: Point, fill_color: ColorRgba) {
    flood_fill_tolerance(buffer, start, fill_color, 0);
}

/// Performs 4-connected flood fill starting at `start` with `fill_color` and custom tolerance (0..=255).
pub fn flood_fill_tolerance(
    buffer: &mut PixelBuffer,
    start: Point,
    fill_color: ColorRgba,
    tolerance: u8,
) {
    if start.x < 0
        || start.y < 0
        || start.x >= buffer.width() as i32
        || start.y >= buffer.height() as i32
    {
        return;
    }

    let target_color = match buffer.get_pixel_i32(start.x, start.y) {
        Some(c) => c,
        None => return,
    };

    if color_matches(target_color, fill_color, tolerance) && tolerance == 0 {
        return;
    }

    let width = buffer.width() as i32;
    let height = buffer.height() as i32;

    let mut visited = vec![false; (width as usize) * (height as usize)];
    let mut queue = VecDeque::new();
    queue.push_back((start.x, start.y));

    while let Some((cx, cy)) = queue.pop_front() {
        if cx < 0 || cy < 0 || cx >= width || cy >= height {
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

        // Find left and right span
        let mut left = cx;
        while left > 0 {
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
        while right < width - 1 {
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

        // Fill horizontal scanline span
        for x in left..=right {
            let pidx = (cy * width + x) as usize;
            visited[pidx] = true;
            buffer.set_pixel_i32(x, cy, fill_color);
        }

        // Check scanlines above and below
        for &ny in &[cy - 1, cy + 1] {
            if ny >= 0 && ny < height {
                let mut x = left;
                while x <= right {
                    let nidx = (ny * width + x) as usize;
                    if !visited[nidx] {
                        if let Some(c) = buffer.get_pixel_i32(x, ny) {
                            if color_matches(c, target_color, tolerance) {
                                queue.push_back((x, ny));
                                while x <= right {
                                    let sub_idx = (ny * width + x) as usize;
                                    if visited[sub_idx] {
                                        break;
                                    }
                                    if let Some(sc) = buffer.get_pixel_i32(x, ny) {
                                        if !color_matches(sc, target_color, tolerance) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, RED, WHITE};

    #[test]
    fn test_flood_fill_simple_rect() {
        let mut buffer = PixelBuffer::new(10, 10, WHITE);

        // Draw black border around 4x4 inner box (from 2,2 to 5,5)
        for x in 2..=5 {
            buffer.set_pixel_i32(x, 2, BLACK);
            buffer.set_pixel_i32(x, 5, BLACK);
        }
        for y in 2..=5 {
            buffer.set_pixel_i32(2, y, BLACK);
            buffer.set_pixel_i32(5, y, BLACK);
        }

        // Fill inside at (3, 3) with RED
        flood_fill(&mut buffer, Point::new(3, 3), RED);

        assert_eq!(buffer.get_pixel_i32(3, 3), Some(RED));
        assert_eq!(buffer.get_pixel_i32(4, 4), Some(RED));

        // Outside is still WHITE
        assert_eq!(buffer.get_pixel_i32(0, 0), Some(WHITE));
        // Border is still BLACK
        assert_eq!(buffer.get_pixel_i32(2, 2), Some(BLACK));
    }

    #[test]
    fn test_flood_fill_with_tolerance() {
        let mut buffer = PixelBuffer::new(10, 10, WHITE);
        // Set slightly off-white pixels
        buffer.set_pixel_i32(1, 1, ColorRgba::rgb(250, 250, 250));
        buffer.set_pixel_i32(1, 2, ColorRgba::rgb(240, 240, 240));

        // Fill with tolerance 20
        flood_fill_tolerance(&mut buffer, Point::new(0, 0), RED, 20);

        // (1, 1) and (1, 2) should be filled because delta <= 20
        assert_eq!(buffer.get_pixel_i32(0, 0), Some(RED));
        assert_eq!(buffer.get_pixel_i32(1, 1), Some(RED));
        assert_eq!(buffer.get_pixel_i32(1, 2), Some(RED));
    }
}

//! Selection tools (Rectangular and Free-Form Lasso) with move, copy (Ctrl+drag), and stamp/trail (Shift+drag).

use crate::geometry::{bresenham_line, rasterize_polygon_scanline, Point, Rect};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SelectionMode {
    #[default]
    Opaque,
    Transparent, // Pixels matching bg_color are treated as transparent
}

#[derive(Debug, Clone)]
pub struct FloatingSelection {
    pub original_rect: Rect,
    pub current_origin: Point,
    pub buffer: PixelBuffer,
    pub mask: Option<Vec<bool>>, // true for selected pixels if freeform lasso
}

impl FloatingSelection {
    pub fn width(&self) -> u32 {
        self.buffer.width()
    }

    pub fn height(&self) -> u32 {
        self.buffer.height()
    }

    pub fn current_rect(&self) -> Rect {
        Rect {
            min: self.current_origin,
            max: Point::new(
                self.current_origin.x + self.width() as i32 - 1,
                self.current_origin.y + self.height() as i32 - 1,
            ),
        }
    }
}

/// Lifts rectangular area from `canvas` into a `FloatingSelection`, clearing the canvas area with `bg_color`.
pub fn lift_rectangular_selection(
    canvas: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    bg_color: ColorRgba,
) -> Option<FloatingSelection> {
    let rect = Rect::from_points(p0, p1);
    let w = (rect.max.x - rect.min.x + 1).max(1) as u32;
    let h = (rect.max.y - rect.min.y + 1).max(1) as u32;

    let mut sel_buffer = PixelBuffer::new(w, h, bg_color);

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let src_x = rect.min.x + x;
            let src_y = rect.min.y + y;
            if let Some(c) = canvas.get_pixel_i32(src_x, src_y) {
                sel_buffer.set_pixel_i32(x, y, c);
            }
            canvas.set_pixel_i32(src_x, src_y, bg_color);
        }
    }

    Some(FloatingSelection {
        original_rect: rect,
        current_origin: rect.min,
        buffer: sel_buffer,
        mask: None,
    })
}

/// Lifts a free-form lasso polygon from `canvas` into a `FloatingSelection`.
pub fn lift_freeform_selection(
    canvas: &mut PixelBuffer,
    points: &[Point],
    bg_color: ColorRgba,
) -> Option<FloatingSelection> {
    if points.len() < 3 {
        return None;
    }

    let min_x = points.iter().map(|p| p.x).min().unwrap_or(0);
    let max_x = points.iter().map(|p| p.x).max().unwrap_or(0);
    let min_y = points.iter().map(|p| p.y).min().unwrap_or(0);
    let max_y = points.iter().map(|p| p.y).max().unwrap_or(0);

    let rect = Rect {
        min: Point::new(min_x, min_y),
        max: Point::new(max_x, max_y),
    };

    let w = (max_x - min_x + 1).max(1) as u32;
    let h = (max_y - min_y + 1).max(1) as u32;

    let mut sel_buffer = PixelBuffer::new(w, h, bg_color);
    let mut mask = vec![false; (w * h) as usize];

    // Compute polygon fill spans relative to min_x, min_y
    let spans = rasterize_polygon_scanline(points);
    for (y, x0, x1) in spans {
        let rel_y = y - min_y;
        if rel_y >= 0 && rel_y < h as i32 {
            for x in x0..=x1 {
                let rel_x = x - min_x;
                if rel_x >= 0 && rel_x < w as i32 {
                    let idx = (rel_y as u32 * w + rel_x as u32) as usize;
                    mask[idx] = true;
                }
            }
        }
    }

    // Also include boundary
    for i in 0..points.len() {
        let p1 = points[i];
        let p2 = points[(i + 1) % points.len()];
        for pt in bresenham_line(p1, p2) {
            let rel_x = pt.x - min_x;
            let rel_y = pt.y - min_y;
            if rel_x >= 0 && rel_x < w as i32 && rel_y >= 0 && rel_y < h as i32 {
                let idx = (rel_y as u32 * w + rel_x as u32) as usize;
                mask[idx] = true;
            }
        }
    }

    // Copy masked pixels and clear canvas
    for rel_y in 0..h as i32 {
        for rel_x in 0..w as i32 {
            let idx = (rel_y as u32 * w + rel_x as u32) as usize;
            if mask[idx] {
                let src_x = min_x + rel_x;
                let src_y = min_y + rel_y;
                if let Some(c) = canvas.get_pixel_i32(src_x, src_y) {
                    sel_buffer.set_pixel_i32(rel_x, rel_y, c);
                }
                canvas.set_pixel_i32(src_x, src_y, bg_color);
            }
        }
    }

    Some(FloatingSelection {
        original_rect: rect,
        current_origin: rect.min,
        buffer: sel_buffer,
        mask: Some(mask),
    })
}

/// Stamps/blits the `FloatingSelection` onto `canvas` at `dest_origin`.
pub fn stamp_floating_selection(
    canvas: &mut PixelBuffer,
    selection: &FloatingSelection,
    dest_origin: Point,
    mode: SelectionMode,
    transparent_color: ColorRgba,
) {
    let w = selection.width() as i32;
    let h = selection.height() as i32;

    for dy in 0..h {
        for dx in 0..w {
            if let Some(ref mask) = selection.mask {
                let idx = (dy as u32 * selection.width() + dx as u32) as usize;
                if !mask.get(idx).copied().unwrap_or(false) {
                    continue;
                }
            }

            if let Some(color) = selection.buffer.get_pixel_i32(dx, dy) {
                if mode == SelectionMode::Transparent && color == transparent_color {
                    continue;
                }
                canvas.set_pixel_i32(dest_origin.x + dx, dest_origin.y + dy, color);
            }
        }
    }
}

/// Commits the `FloatingSelection` onto `canvas` and consumes it.
pub fn commit_floating_selection(
    canvas: &mut PixelBuffer,
    selection: FloatingSelection,
    mode: SelectionMode,
    transparent_color: ColorRgba,
) {
    stamp_floating_selection(
        canvas,
        &selection,
        selection.current_origin,
        mode,
        transparent_color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{RED, WHITE};

    #[test]
    fn test_rectangular_selection_lift_and_stamp() {
        let mut canvas = PixelBuffer::new(20, 20, WHITE);
        canvas.set_pixel_i32(5, 5, RED);

        let sel =
            lift_rectangular_selection(&mut canvas, Point::new(4, 4), Point::new(6, 6), WHITE)
                .expect("selection lift failed");

        // Canvas original spot should be cleared to WHITE
        assert_eq!(canvas.get_pixel_i32(5, 5), Some(WHITE));

        // Stamp selection at (10, 10)
        stamp_floating_selection(
            &mut canvas,
            &sel,
            Point::new(10, 10),
            SelectionMode::Opaque,
            WHITE,
        );
        assert_eq!(canvas.get_pixel_i32(11, 11), Some(RED));
    }
}

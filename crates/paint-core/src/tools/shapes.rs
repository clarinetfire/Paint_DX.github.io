//! Shape tools (Rectangle, Ellipse, Rounded Rectangle, Polygon, Curve) with 3 fill styles.

use crate::geometry::{
    rasterize_bezier_cubic, rasterize_bezier_quadratic, rasterize_ellipse,
    rasterize_polygon_scanline, rasterize_rounded_rectangle, Point,
};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use crate::tools::line::draw_line;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FillStyle {
    #[default]
    OutlineOnly, // Border drawn with primary color
    FilledWithOutline, // Border drawn with primary color, filled with secondary color
    FillOnly,          // Solid fill with primary color (no border)
}

/// Helper to stamp a thickness disk/square at a point for thick outlines.
fn stamp_point(buffer: &mut PixelBuffer, pt: Point, width: u32, color: ColorRgba) {
    if width <= 1 {
        buffer.set_pixel_i32(pt.x, pt.y, color);
        return;
    }

    let half = (width / 2) as i32;
    for dy in -half..=half {
        for dx in -half..=half {
            buffer.set_pixel_i32(pt.x + dx, pt.y + dy, color);
        }
    }
}

/// Draws a rectangle with fill style and line width.
pub fn draw_rectangle(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    style: FillStyle,
    line_width: u32,
    border_color: ColorRgba,
    fill_color: ColorRgba,
) {
    let min_x = p0.x.min(p1.x);
    let max_x = p0.x.max(p1.x);
    let min_y = p0.y.min(p1.y);
    let max_y = p0.y.max(p1.y);

    // 1. Fill if needed
    match style {
        FillStyle::FillOnly => {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    buffer.set_pixel_i32(x, y, border_color);
                }
            }
            return;
        }
        FillStyle::FilledWithOutline => {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    buffer.set_pixel_i32(x, y, fill_color);
                }
            }
        }
        FillStyle::OutlineOnly => {}
    }

    // 2. Draw border
    draw_line(
        buffer,
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        line_width,
        border_color,
    );
    draw_line(
        buffer,
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        line_width,
        border_color,
    );
    draw_line(
        buffer,
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
        line_width,
        border_color,
    );
    draw_line(
        buffer,
        Point::new(min_x, max_y),
        Point::new(min_x, min_y),
        line_width,
        border_color,
    );
}

/// Draws an ellipse with fill style and line width.
pub fn draw_ellipse(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    style: FillStyle,
    line_width: u32,
    border_color: ColorRgba,
    fill_color: ColorRgba,
) {
    let (outline, spans) = rasterize_ellipse(p0, p1);

    // Fill
    match style {
        FillStyle::FillOnly => {
            for (y, x0, x1) in spans {
                for x in x0..=x1 {
                    buffer.set_pixel_i32(x, y, border_color);
                }
            }
            return;
        }
        FillStyle::FilledWithOutline => {
            for (y, x0, x1) in spans {
                for x in x0..=x1 {
                    buffer.set_pixel_i32(x, y, fill_color);
                }
            }
        }
        FillStyle::OutlineOnly => {}
    }

    // Outline
    for pt in outline {
        stamp_point(buffer, pt, line_width, border_color);
    }
}

/// Draws a rounded rectangle with fill style, corner radius, and line width.
#[allow(clippy::too_many_arguments)]
pub fn draw_rounded_rectangle(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    radius: i32,
    style: FillStyle,
    line_width: u32,
    border_color: ColorRgba,
    fill_color: ColorRgba,
) {
    let (outline, spans) = rasterize_rounded_rectangle(p0, p1, radius);

    // Fill
    match style {
        FillStyle::FillOnly => {
            for (y, x0, x1) in spans {
                for x in x0..=x1 {
                    buffer.set_pixel_i32(x, y, border_color);
                }
            }
            return;
        }
        FillStyle::FilledWithOutline => {
            for (y, x0, x1) in spans {
                for x in x0..=x1 {
                    buffer.set_pixel_i32(x, y, fill_color);
                }
            }
        }
        FillStyle::OutlineOnly => {}
    }

    // Outline
    for pt in outline {
        stamp_point(buffer, pt, line_width, border_color);
    }
}

/// Draws a closed polygon with fill style and line width.
pub fn draw_polygon(
    buffer: &mut PixelBuffer,
    vertices: &[Point],
    style: FillStyle,
    line_width: u32,
    border_color: ColorRgba,
    fill_color: ColorRgba,
) {
    if vertices.len() < 2 {
        return;
    }

    if vertices.len() >= 3 {
        let spans = rasterize_polygon_scanline(vertices);
        match style {
            FillStyle::FillOnly => {
                for (y, x0, x1) in spans {
                    for x in x0..=x1 {
                        buffer.set_pixel_i32(x, y, border_color);
                    }
                }
                return;
            }
            FillStyle::FilledWithOutline => {
                for (y, x0, x1) in spans {
                    for x in x0..=x1 {
                        buffer.set_pixel_i32(x, y, fill_color);
                    }
                }
            }
            FillStyle::OutlineOnly => {}
        }
    }

    // Draw polygon edges
    for i in 0..vertices.len() {
        let p1 = vertices[i];
        let p2 = vertices[(i + 1) % vertices.len()];
        draw_line(buffer, p1, p2, line_width, border_color);
    }
}

/// Draws a Bezier curve (1 control point quadratic or 2 control points cubic).
pub fn draw_curve_quadratic(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    p2: Point,
    line_width: u32,
    color: ColorRgba,
) {
    let pts = rasterize_bezier_quadratic(p0, p1, p2);
    for pt in pts {
        stamp_point(buffer, pt, line_width, color);
    }
}

pub fn draw_curve_cubic(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    line_width: u32,
    color: ColorRgba,
) {
    let pts = rasterize_bezier_cubic(p0, p1, p2, p3);
    for pt in pts {
        stamp_point(buffer, pt, line_width, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, RED, WHITE};

    #[test]
    fn test_draw_rectangle_filled_with_outline() {
        let mut buffer = PixelBuffer::new(20, 20, WHITE);
        draw_rectangle(
            &mut buffer,
            Point::new(2, 2),
            Point::new(10, 10),
            FillStyle::FilledWithOutline,
            1,
            BLACK,
            RED,
        );

        // Center is RED
        assert_eq!(buffer.get_pixel_i32(5, 5), Some(RED));
        // Border is BLACK
        assert_eq!(buffer.get_pixel_i32(2, 2), Some(BLACK));
        assert_eq!(buffer.get_pixel_i32(10, 10), Some(BLACK));
    }

    #[test]
    fn test_draw_ellipse_fill_only() {
        let mut buffer = PixelBuffer::new(20, 20, WHITE);
        draw_ellipse(
            &mut buffer,
            Point::new(2, 2),
            Point::new(10, 10),
            FillStyle::FillOnly,
            1,
            RED,
            BLACK,
        );
        assert_eq!(buffer.get_pixel_i32(6, 6), Some(RED));
    }
}

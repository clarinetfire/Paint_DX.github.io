//! Texture and Glyph Wacky Brushes.
//!
//! Sub-agent 3: Rainbow, Caterpillar, Pancake Stack, Fuzzer / Fur, Pattern Roll, Alphabet.

use super::{blend_filled_disk, draw_filled_disk, next_lcg_rand, WackyBrushShape, WackyBrushState};
use crate::geometry::{bresenham_line, Point};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use crate::tools::text::get_ascii_glyph;

/// 7 rich spectral rainbow bands: Red, Orange, Yellow, Green, Cyan, Blue, Violet.
pub const RAINBOW_SPECTRUM: [ColorRgba; 7] = [
    ColorRgba::rgb(255, 30, 30),  // Red
    ColorRgba::rgb(255, 130, 20), // Orange
    ColorRgba::rgb(255, 230, 20), // Yellow
    ColorRgba::rgb(30, 200, 40),  // Green
    ColorRgba::rgb(20, 200, 240), // Cyan
    ColorRgba::rgb(40, 80, 255),  // Blue
    ColorRgba::rgb(150, 40, 220), // Violet
];

/// Retro 8x8 herringbone weave pattern used for PatternRoll.
pub const PATTERN_HERRINGBONE_8X8: [u8; 8] = [
    0b11000011, 0b01100110, 0b00111100, 0b00011000, 0b00111100, 0b01100110, 0b11000011, 0b10000001,
];

/// Applies a texture or glyph wacky brush point.
pub fn apply_texture_brush_point(
    buffer: &mut PixelBuffer,
    center: Point,
    shape: WackyBrushShape,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    match shape {
        WackyBrushShape::Rainbow => {
            render_rainbow_point(buffer, center, size, state);
        }
        WackyBrushShape::Caterpillar => {
            render_caterpillar_segment(buffer, center, size, true);
        }
        WackyBrushShape::PancakeStack => {
            render_pancake_slice(buffer, center, size, true);
        }
        WackyBrushShape::FuzzerFur => {
            render_fur_point(buffer, center, size, fg, state, None);
        }
        WackyBrushShape::PatternRoll => {
            render_pattern_roll_stamp(buffer, center, size, fg, bg);
        }
        WackyBrushShape::Alphabet => {
            render_alphabet_glyph(buffer, center, size, fg, bg, state);
        }
        _ => {}
    }
}

/// Draws an interpolated texture or glyph wacky brush stroke.
#[allow(clippy::too_many_arguments)]
pub fn draw_texture_brush_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    shape: WackyBrushShape,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    match shape {
        WackyBrushShape::Rainbow => {
            draw_rainbow_stroke(buffer, p0, p1, size, state);
        }
        WackyBrushShape::Caterpillar => {
            draw_caterpillar_stroke(buffer, p0, p1, size, state);
        }
        WackyBrushShape::PancakeStack => {
            draw_pancake_stroke(buffer, p0, p1, size, state);
        }
        WackyBrushShape::FuzzerFur => {
            draw_fur_stroke(buffer, p0, p1, size, fg, state);
        }
        WackyBrushShape::PatternRoll => {
            let pts = bresenham_line(p0, p1);
            for pt in pts {
                render_pattern_roll_stamp(buffer, pt, size, fg, bg);
            }
        }
        WackyBrushShape::Alphabet => {
            draw_alphabet_stroke(buffer, p0, p1, size, fg, bg, state);
        }
        _ => {
            let pts = bresenham_line(p0, p1);
            for pt in pts {
                apply_texture_brush_point(buffer, pt, shape, size, fg, bg, state);
            }
        }
    }
}

// ============================================================================
// 1. Rainbow Ribbon
// ============================================================================

fn render_rainbow_point(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    _state: &mut WackyBrushState,
) {
    let band_w = (size as f64 / 3.0).max(1.0);
    for (i, &col) in RAINBOW_SPECTRUM.iter().enumerate() {
        let offset = ((i as f64 - 3.0) * band_w).round() as i32;
        let pt = Point::new(center.x + offset, center.y + offset / 2);
        let r = (band_w.round() as i32 / 2).max(0);
        draw_filled_disk(buffer, pt, r, col);
    }
}

fn draw_rainbow_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: u32,
    _state: &mut WackyBrushState,
) {
    let dx = (p1.x - p0.x) as f64;
    let dy = (p1.y - p0.y) as f64;
    let dist = (dx * dx + dy * dy).sqrt();

    let (nx, ny) = if dist < 0.001 {
        (0.0, 1.0)
    } else {
        (-dy / dist, dx / dist)
    };

    let band_w = (size as f64 / 3.0).max(1.0);
    let band_rad = (band_w.round() as i32 / 2).max(0);

    for (i, &col) in RAINBOW_SPECTRUM.iter().enumerate() {
        let offset = (i as f64 - 3.0) * band_w;
        let b0 = Point::new(
            (p0.x as f64 + nx * offset).round() as i32,
            (p0.y as f64 + ny * offset).round() as i32,
        );
        let b1 = Point::new(
            (p1.x as f64 + nx * offset).round() as i32,
            (p1.y as f64 + ny * offset).round() as i32,
        );

        let pts = bresenham_line(b0, b1);
        for pt in pts {
            if band_rad <= 0 {
                buffer.set_pixel_i32(pt.x, pt.y, col);
            } else {
                draw_filled_disk(buffer, pt, band_rad, col);
            }
        }
    }
}

// ============================================================================
// 2. Caterpillar
// ============================================================================

fn render_caterpillar_segment(buffer: &mut PixelBuffer, center: Point, size: u32, is_head: bool) {
    let radius = (size as i32 + 2).clamp(3, 16);
    let r2 = radius * radius;

    let col_edge = ColorRgba::rgb(20, 75, 25);
    let col_body = ColorRgba::rgb(105, 205, 45);
    let col_high = ColorRgba::rgb(190, 245, 80);
    let col_spot = ColorRgba::rgb(255, 220, 40);

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let d2 = dx * dx + dy * dy;
            if d2 <= r2 {
                let px = center.x + dx;
                let py = center.y + dy;

                if d2 >= (radius - 1) * (radius - 1) {
                    buffer.set_pixel_i32(px, py, col_edge);
                } else if dx < -1 && dy < -1 && (dx * dx + dy * dy) <= (radius / 2) * (radius / 2) {
                    buffer.set_pixel_i32(px, py, col_high);
                } else if dx == 0 && dy == 0 && radius >= 5 {
                    buffer.set_pixel_i32(px, py, col_spot);
                } else {
                    buffer.set_pixel_i32(px, py, col_body);
                }
            }
        }
    }

    if is_head {
        // Cute eyes (white + black pupil)
        let eye_offset = (radius / 2).max(2);
        let eye_y = center.y - eye_offset;
        let left_eye = Point::new(center.x - eye_offset, eye_y);
        let right_eye = Point::new(center.x + eye_offset, eye_y);

        buffer.set_pixel_i32(left_eye.x, left_eye.y, ColorRgba::rgb(255, 255, 255));
        buffer.set_pixel_i32(left_eye.x + 1, left_eye.y, ColorRgba::rgb(0, 0, 0));
        buffer.set_pixel_i32(right_eye.x, right_eye.y, ColorRgba::rgb(255, 255, 255));
        buffer.set_pixel_i32(right_eye.x + 1, right_eye.y, ColorRgba::rgb(0, 0, 0));

        // Cute antennae
        let ant_len = (radius / 2).max(2);
        for i in 1..=ant_len {
            buffer.set_pixel_i32(center.x - eye_offset - i / 2, eye_y - i, col_edge);
            buffer.set_pixel_i32(center.x + eye_offset + i / 2, eye_y - i, col_edge);
        }
        buffer.set_pixel_i32(
            center.x - eye_offset - ant_len / 2,
            eye_y - ant_len,
            ColorRgba::rgb(255, 120, 40),
        );
        buffer.set_pixel_i32(
            center.x + eye_offset + ant_len / 2,
            eye_y - ant_len,
            ColorRgba::rgb(255, 120, 40),
        );
    }
}

fn draw_caterpillar_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: u32,
    state: &mut WackyBrushState,
) {
    let spacing = (size as i32 * 2).clamp(6, 22);
    let pts = bresenham_line(p0, p1);

    for pt in pts {
        let should_stamp = match state.last_point {
            Some(last) => {
                let dx = pt.x - last.x;
                let dy = pt.y - last.y;
                (dx * dx + dy * dy) >= spacing * spacing
            }
            None => true,
        };

        if should_stamp {
            render_caterpillar_segment(buffer, pt, size, false);
            state.last_point = Some(pt);
            state.step = state.step.wrapping_add(1);
        }
    }

    // Render head at current destination p1
    render_caterpillar_segment(buffer, p1, size, true);
    state.last_point = Some(p1);
}

// ============================================================================
// 3. Pancake Stack
// ============================================================================

fn render_pancake_slice(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    with_butter_and_syrup: bool,
) {
    let rx = (size as i32 * 2 + 4).clamp(6, 28);
    let ry = (rx / 2).max(3);

    let col_crust = ColorRgba::rgb(140, 75, 20);
    let col_pancake = ColorRgba::rgb(235, 175, 80);
    let col_highlight = ColorRgba::rgb(255, 215, 130);
    let col_syrup = ColorRgba::rgb(115, 45, 10);
    let col_butter = ColorRgba::rgb(255, 235, 75);
    let col_butter_border = ColorRgba::rgb(205, 155, 30);

    let rx2 = (rx * rx) as i64;
    let ry2 = (ry * ry) as i64;

    for dy in -ry..=ry {
        for dx in -rx..=rx {
            let lhs = (dx as i64 * dx as i64) * ry2 + (dy as i64 * dy as i64) * rx2;
            if lhs <= rx2 * ry2 {
                let px = center.x + dx;
                let py = center.y + dy;

                let is_rim = lhs >= (rx2 * ry2 * 8) / 10;
                if is_rim {
                    buffer.set_pixel_i32(px, py, col_crust);
                } else if dy < 0 && dx < rx / 2 {
                    buffer.set_pixel_i32(px, py, col_highlight);
                } else {
                    buffer.set_pixel_i32(px, py, col_pancake);
                }
            }
        }
    }

    if with_butter_and_syrup {
        // Syrup drips down edge
        let drip_x1 = center.x - rx / 3;
        let drip_x2 = center.x + rx / 4;
        for d in 0..=(ry / 2 + 2) {
            buffer.set_pixel_i32(drip_x1, center.y + ry / 2 + d, col_syrup);
            buffer.set_pixel_i32(drip_x2, center.y + ry / 2 + d + 1, col_syrup);
        }

        // Butter pat on top
        let butter_w = (rx / 4).clamp(2, 6);
        let butter_h = (ry / 3).clamp(2, 4);
        for by in -butter_h..=butter_h {
            for bx in -butter_w..=butter_w {
                let px = center.x + bx;
                let py = center.y - (ry / 2) + by;
                if bx == -butter_w || bx == butter_w || by == -butter_h || by == butter_h {
                    buffer.set_pixel_i32(px, py, col_butter_border);
                } else {
                    buffer.set_pixel_i32(px, py, col_butter);
                }
            }
        }
    }
}

fn draw_pancake_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: u32,
    state: &mut WackyBrushState,
) {
    let spacing = (size as i32 * 2 + 2).clamp(8, 24);
    let pts = bresenham_line(p0, p1);

    for (idx, pt) in pts.into_iter().enumerate() {
        let should_stamp = match state.last_point {
            Some(last) => {
                let dx = pt.x - last.x;
                let dy = pt.y - last.y;
                (dx * dx + dy * dy) >= spacing * spacing
            }
            None => true,
        };

        if should_stamp {
            render_pancake_slice(buffer, pt, size, false);
            state.last_point = Some(pt);
            state.step = state.step.wrapping_add(1);
        }

        let _ = idx;
    }

    // Top pancake with fresh melting butter & syrup
    render_pancake_slice(buffer, p1, size, true);
    state.last_point = Some(p1);
}

// ============================================================================
// 4. Fuzzer / Fur
// ============================================================================

fn render_fur_point(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
    travel_normal: Option<(f64, f64)>,
) {
    let core_r = (size as i32 / 3).max(0);
    draw_filled_disk(buffer, center, core_r, fg);

    let hair_count = (size * 2).clamp(4, 16);
    let max_len = (size as i32 * 2 + 2).clamp(3, 20);

    for _ in 0..hair_count {
        let rand_val = next_lcg_rand(&mut state.step);
        let len = ((rand_val % (max_len as u32 + 1)) as i32).max(1);

        let (dx, dy) = if let Some((nx, ny)) = travel_normal {
            // Perpendicular with randomized length and sign
            let sign = if (rand_val & 1) == 0 { 1.0 } else { -1.0 };
            let jitter = ((rand_val >> 2) % 5) as f64 - 2.0;
            let fx = (nx * sign * len as f64) + jitter * 0.2;
            let fy = (ny * sign * len as f64) + jitter * 0.2;
            (fx.round() as i32, fy.round() as i32)
        } else {
            // Radial bristles when stationary
            let angle = (rand_val % 360) as f64 * std::f64::consts::PI / 180.0;
            let fx = angle.cos() * len as f64;
            let fy = angle.sin() * len as f64;
            (fx.round() as i32, fy.round() as i32)
        };

        let hair_pts = bresenham_line(center, Point::new(center.x + dx, center.y + dy));
        for (i, p) in hair_pts.iter().enumerate() {
            let alpha = if i == 0 {
                fg.a
            } else {
                ((fg.a as u32 * (hair_pts.len() - i) as u32) / hair_pts.len() as u32) as u8
            };
            blend_filled_disk(
                buffer,
                *p,
                0,
                ColorRgba::rgba(fg.r, fg.g, fg.b, alpha.max(70)),
            );
        }
    }
}

fn draw_fur_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
) {
    let dx = (p1.x - p0.x) as f64;
    let dy = (p1.y - p0.y) as f64;
    let dist = (dx * dx + dy * dy).sqrt();

    let normal = if dist > 0.001 {
        Some((-dy / dist, dx / dist))
    } else {
        None
    };

    let pts = bresenham_line(p0, p1);
    for pt in pts {
        render_fur_point(buffer, pt, size, fg, state, normal);
    }
}

// ============================================================================
// 5. Pattern Roll
// ============================================================================

fn render_pattern_roll_stamp(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
) {
    let radius = (size as i32).max(2);
    let r2 = radius * radius;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let d2 = dx * dx + dy * dy;
            if d2 <= r2 {
                let px = center.x + dx;
                let py = center.y + dy;

                let tx = px.rem_euclid(8) as usize;
                let ty = py.rem_euclid(8) as usize;

                let bit_set = (PATTERN_HERRINGBONE_8X8[ty] & (0x80 >> tx)) != 0;
                let col = if bit_set { fg } else { bg };

                // Draw stamped pixel
                buffer.set_pixel_i32(px, py, col);
            }
        }
    }
}

// ============================================================================
// 6. Alphabet
// ============================================================================

fn render_alphabet_glyph(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    _bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    let ch = (b'A' + (state.alphabet_index % 26)) as char;
    let glyph = get_ascii_glyph(ch);
    let scale = (size / 3).clamp(1, 4) as i32;

    let origin_x = center.x - (4 * scale);
    let origin_y = center.y - (4 * scale);

    let shadow_col = ColorRgba::rgba(0, 0, 0, 160);

    for (row_idx, &row_byte) in glyph.iter().enumerate() {
        for bit in 0..8_i32 {
            if (row_byte & (0x80 >> bit)) != 0 {
                // Drop shadow
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = origin_x + bit * scale + sx + 1;
                        let py = origin_y + (row_idx as i32) * scale + sy + 1;
                        buffer.blend_pixel_i32(px, py, shadow_col);
                    }
                }
                // Glyph pixel
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = origin_x + bit * scale + sx;
                        let py = origin_y + (row_idx as i32) * scale + sy;
                        buffer.set_pixel_i32(px, py, fg);
                    }
                }
            }
        }
    }

    state.alphabet_index = (state.alphabet_index + 1) % 26;
}

fn draw_alphabet_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    let spacing = (size as i32 * 2 + 12).clamp(12, 32);
    let pts = bresenham_line(p0, p1);

    for pt in pts {
        let should_stamp = match state.last_point {
            Some(last) => {
                let dx = pt.x - last.x;
                let dy = pt.y - last.y;
                (dx * dx + dy * dy) >= spacing * spacing
            }
            None => true,
        };

        if should_stamp {
            render_alphabet_glyph(buffer, pt, size, fg, bg, state);
            state.last_point = Some(pt);
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, RED, WHITE};

    #[test]
    fn test_rainbow_brush() {
        let mut buffer = PixelBuffer::new(50, 50, WHITE);
        let mut state = WackyBrushState::new();

        draw_texture_brush_stroke(
            &mut buffer,
            Point::new(10, 10),
            Point::new(40, 40),
            WackyBrushShape::Rainbow,
            6,
            RED,
            WHITE,
            &mut state,
        );

        let mut colored_count = 0;
        for y in 0..50 {
            for x in 0..50 {
                if buffer.get_pixel(x, y).unwrap() != WHITE {
                    colored_count += 1;
                }
            }
        }
        assert!(colored_count > 50, "Rainbow should render colorful ribbon");
    }

    #[test]
    fn test_caterpillar_brush() {
        let mut buffer = PixelBuffer::new(60, 60, WHITE);
        let mut state = WackyBrushState::new();

        draw_texture_brush_stroke(
            &mut buffer,
            Point::new(10, 30),
            Point::new(50, 30),
            WackyBrushShape::Caterpillar,
            4,
            RED,
            WHITE,
            &mut state,
        );

        assert!(state.last_point.is_some());
    }

    #[test]
    fn test_pancake_stack_brush() {
        let mut buffer = PixelBuffer::new(60, 60, WHITE);
        let mut state = WackyBrushState::new();

        draw_texture_brush_stroke(
            &mut buffer,
            Point::new(30, 10),
            Point::new(30, 50),
            WackyBrushShape::PancakeStack,
            5,
            RED,
            WHITE,
            &mut state,
        );

        assert!(state.last_point.is_some());
    }

    #[test]
    fn test_fuzzer_fur_brush() {
        let mut buffer = PixelBuffer::new(40, 40, WHITE);
        let mut state = WackyBrushState::new();

        draw_texture_brush_stroke(
            &mut buffer,
            Point::new(20, 10),
            Point::new(20, 30),
            WackyBrushShape::FuzzerFur,
            4,
            BLACK,
            WHITE,
            &mut state,
        );

        let mut dark_count = 0;
        for y in 0..40 {
            for x in 0..40 {
                if buffer.get_pixel(x, y).unwrap() != WHITE {
                    dark_count += 1;
                }
            }
        }
        assert!(dark_count > 30, "Fur should produce bristles around stroke");
    }

    #[test]
    fn test_pattern_roll_brush() {
        let mut buffer = PixelBuffer::new(40, 40, WHITE);
        let mut state = WackyBrushState::new();

        draw_texture_brush_stroke(
            &mut buffer,
            Point::new(10, 20),
            Point::new(30, 20),
            WackyBrushShape::PatternRoll,
            5,
            BLACK,
            WHITE,
            &mut state,
        );

        let mut black_count = 0;
        for y in 0..40 {
            for x in 0..40 {
                if buffer.get_pixel(x, y).unwrap() == BLACK {
                    black_count += 1;
                }
            }
        }
        assert!(black_count > 10, "Pattern roll should stamp pattern pixels");
    }

    #[test]
    fn test_alphabet_brush_advances() {
        let mut buffer = PixelBuffer::new(100, 40, WHITE);
        let mut state = WackyBrushState::new();
        assert_eq!(state.alphabet_index, 0);

        apply_texture_brush_point(
            &mut buffer,
            Point::new(20, 20),
            WackyBrushShape::Alphabet,
            4,
            BLACK,
            WHITE,
            &mut state,
        );

        assert_eq!(state.alphabet_index, 1);

        apply_texture_brush_point(
            &mut buffer,
            Point::new(40, 20),
            WackyBrushShape::Alphabet,
            4,
            BLACK,
            WHITE,
            &mut state,
        );

        assert_eq!(state.alphabet_index, 2);
    }
}

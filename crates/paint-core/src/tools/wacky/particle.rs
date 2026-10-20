//! Particle and Dynamic Ink Wacky Brushes.
//!
//! Features:
//! 1. Leaky Pen: Expanding ink blot with organic round bleed and vertical gravity drip with teardrop bulb.
//! 2. Bubbles: Outlined circles of randomized radii with translucent interior, rim outline, and specular highlight.
//! 3. Splatter: High-velocity spray paint with dense core and power-law ballistic dispersion.
//! 4. Northern Lights: Shimmering spectral aurora rays (emerald green, neon cyan, electric magenta) with alpha attenuation.
//! 5. Confetti: Clustered geometric particles (diamonds, triangles, rectangles, squares) in bright fiesta colors.
//! 6. Connect-The-Dots: Spaced anchor beads linked by crisp straight Bresenham lines.

use super::{draw_filled_disk, next_lcg_rand, WackyBrushShape, WackyBrushState};
use crate::geometry::{bresenham_line, Point};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;

// ============================================================================
// CONSTANTS & PALETTES
// ============================================================================

pub const AURORA_EMERALD: ColorRgba = ColorRgba::rgb(0, 230, 118);
pub const AURORA_CYAN: ColorRgba = ColorRgba::rgb(0, 229, 255);
pub const AURORA_MAGENTA: ColorRgba = ColorRgba::rgb(255, 0, 128);

pub const FIESTA_GOLD: ColorRgba = ColorRgba::rgb(255, 215, 0);
pub const FIESTA_MAGENTA: ColorRgba = ColorRgba::rgb(255, 20, 147);
pub const FIESTA_CYAN: ColorRgba = ColorRgba::rgb(0, 238, 255);
pub const FIESTA_LIME: ColorRgba = ColorRgba::rgb(50, 205, 50);
pub const FIESTA_ORANGE: ColorRgba = ColorRgba::rgb(255, 127, 0);

pub const FIESTA_COLORS: [ColorRgba; 5] = [
    FIESTA_GOLD,
    FIESTA_MAGENTA,
    FIESTA_CYAN,
    FIESTA_LIME,
    FIESTA_ORANGE,
];

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let val = (a as f32 + (b as f32 - a as f32) * t).round();
    val.clamp(0.0, 255.0) as u8
}

fn lerp_color(c0: ColorRgba, c1: ColorRgba, t: f32, alpha: u8) -> ColorRgba {
    ColorRgba::rgba(
        lerp_u8(c0.r, c1.r, t),
        lerp_u8(c0.g, c1.g, t),
        lerp_u8(c0.b, c1.b, t),
        alpha,
    )
}

fn draw_bead_marker(
    buffer: &mut PixelBuffer,
    center: Point,
    radius: i32,
    fg: ColorRgba,
    bg: ColorRgba,
) {
    let fill_color = if bg == fg {
        ColorRgba::rgb(255, 255, 255)
    } else {
        bg
    };
    let r2 = radius * radius;
    let inner_r = (radius - 1).max(0);
    let inner_r2 = inner_r * inner_r;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= r2 {
                if dist_sq >= inner_r2 {
                    buffer.set_pixel_i32(center.x + dx, center.y + dy, fg);
                } else {
                    buffer.set_pixel_i32(center.x + dx, center.y + dy, fill_color);
                }
            }
        }
    }
    // Crisp center dot
    buffer.set_pixel_i32(center.x, center.y, fg);
}

// ============================================================================
// PARTICLE BRUSH IMPLEMENTATIONS
// ============================================================================

/// 1. Leaky Pen: Expanding ink blot with organic round bleed and gravity drip stem.
fn apply_leaky_pen(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
) {
    if let Some(last) = state.last_point {
        if last == center {
            state.hold_ticks = state.hold_ticks.saturating_add(1);
        } else {
            state.hold_ticks = 0;
        }
    }

    let base_r = (size as i32 * 2).max(2);
    let expansion = ((state.hold_ticks as i32) / 2).min(size as i32 * 4);
    let blot_r = base_r + expansion;

    // Core ink blot
    draw_filled_disk(buffer, center, blot_r, fg);

    // Organic round bleed around the perimeter
    let bleed_count = 10 + (blot_r as usize);
    for i in 0..bleed_count {
        let angle = (i as f32) * (std::f32::consts::TAU / bleed_count as f32);
        let rand_val = next_lcg_rand(&mut state.step);
        let protrusion = (rand_val % 3) as i32 + 1;
        let rad = blot_r + protrusion;
        let px = center.x + (angle.cos() * rad as f32).round() as i32;
        let py = center.y + (angle.sin() * rad as f32).round() as i32;
        let dot_r = ((rand_val >> 4) % 2) as i32;
        draw_filled_disk(buffer, Point::new(px, py), dot_r, fg);
    }

    // Downward gravity drip stem with teardrop bulb when stationary/held
    if state.hold_ticks > 0 {
        let stem_start_y = center.y + blot_r;
        let stem_len = (state.hold_ticks as i32 * 3).min(45 + size as i32 * 5);
        let stem_end_y = stem_start_y + stem_len;

        // Vertical stem
        for y in stem_start_y..=stem_end_y {
            buffer.set_pixel_i32(center.x, y, fg);
            if size >= 3 {
                buffer.set_pixel_i32(center.x + 1, y, fg);
            }
        }

        // Teardrop bulb at the bottom
        let bulb_r = (2 + (state.hold_ticks as i32 / 3)).min(blot_r + 2);
        let bulb_cy = stem_end_y + bulb_r;
        draw_filled_disk(buffer, Point::new(center.x, bulb_cy), bulb_r, fg);

        // Taper connecting stem into bulb
        for dy in 0..bulb_r {
            let half_w = (dy * bulb_r) / bulb_r.max(1);
            let y = stem_end_y + dy;
            for dx in -half_w..=half_w {
                buffer.set_pixel_i32(center.x + dx, y, fg);
            }
        }
    }
}

// Bubbles: Outlined circles of randomized radii with translucent interior,
// solid rim outline, and crisp white specular reflection highlight at top-left (-r/2, -r/2).
fn apply_bubbles(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
) {
    let min_r: i32 = 3;
    let max_r: i32 = ((size * 2) as i32).max(min_r);
    let r_range = (max_r - min_r + 1).max(1) as u32;
    let r = min_r + (next_lcg_rand(&mut state.step) % r_range) as i32;

    // Drift along path
    let drift_range = (r / 2).max(1);
    let drift_x =
        ((next_lcg_rand(&mut state.step) % (drift_range as u32 * 2 + 1)) as i32) - drift_range;
    let drift_y =
        ((next_lcg_rand(&mut state.step) % (drift_range as u32 * 2 + 1)) as i32) - drift_range;
    let bc = Point::new(center.x + drift_x, center.y + drift_y);

    let rim_color = ColorRgba::rgba(fg.r, fg.g, fg.b, 200);
    let interior_color = ColorRgba::rgba(fg.r, fg.g, fg.b, 40);
    let r2 = r * r;
    let inner_r = (r - 1).max(0);
    let inner_r2 = inner_r * inner_r;

    for dy in -r..=r {
        for dx in -r..=r {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= r2 {
                if dist_sq >= inner_r2 {
                    buffer.blend_pixel_i32(bc.x + dx, bc.y + dy, rim_color);
                } else {
                    buffer.blend_pixel_i32(bc.x + dx, bc.y + dy, interior_color);
                }
            }
        }
    }

    // Crisp white specular reflection highlight at top-left (-r/2, -r/2)
    let hx = bc.x - r / 2;
    let hy = bc.y - r / 2;
    let white = ColorRgba::rgb(255, 255, 255);
    buffer.set_pixel_i32(hx, hy, white);
    if r >= 5 {
        buffer.set_pixel_i32(hx + 1, hy, white);
        buffer.set_pixel_i32(hx, hy + 1, white);
    }
}

// Splatter: Dense core splatter with power-law ballistic dispersion.
fn apply_splatter(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
) {
    let max_radius = ((size as f32) * 8.0).max(12.0);
    let count = 25 + (size as usize) * 15;

    for _ in 0..count {
        let rand_angle = next_lcg_rand(&mut state.step);
        let rand_dist = next_lcg_rand(&mut state.step);

        let theta = (rand_angle % 3600) as f32 * (std::f32::consts::TAU / 3600.0);
        let u = ((rand_dist % 10000) as f32) / 10000.0;
        // Power-law dispersion: heavily concentrated near 0 (dense core), ejected drops further out
        let dist = max_radius * u.powf(2.4);

        let px = center.x + (theta.cos() * dist).round() as i32;
        let py = center.y + (theta.sin() * dist).round() as i32;

        if dist < max_radius * 0.25 {
            // Dense core splatter: solid dots or small disks
            let core_r = if (rand_dist >> 16).is_multiple_of(3) {
                1
            } else {
                0
            };
            draw_filled_disk(buffer, Point::new(px, py), core_r, fg);
        } else if dist < max_radius * 0.65 {
            // Ejected droplets
            buffer.set_pixel_i32(px, py, fg);
        } else {
            // Fine aerosol specks with slight alpha attenuation
            let alpha = 140 + ((rand_angle >> 16) % 116) as u8;
            buffer.blend_pixel_i32(px, py, ColorRgba::rgba(fg.r, fg.g, fg.b, alpha));
        }
    }
}

// Northern Lights: Vertical shimmering spectral rays in vivid polar aurora colors
// (emerald green, neon cyan, electric magenta) with randomized heights and alpha attenuation.
fn apply_northern_lights(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    state: &mut WackyBrushState,
) {
    let half_w = (size as i32 * 2).max(3);
    let base_h = (size as i32 * 14).max(20);

    for dx in -half_w..=half_w {
        let rx = center.x + dx;
        let col_factor = 1.0 - (dx.abs() as f32 / (half_w as f32 + 1.0));

        let rand_up = next_lcg_rand(&mut state.step);
        let rand_down = next_lcg_rand(&mut state.step);

        let h_up = (base_h / 2) + ((rand_up % (base_h as u32 + 1)) as i32);
        let h_down = (base_h / 4) + ((rand_down % ((base_h / 2) as u32 + 1)) as i32);

        // Upward ray: neon cyan near center transitioning to electric magenta at top
        for y in (center.y - h_up)..=center.y {
            let dist_up = (center.y - y) as f32 / h_up.max(1) as f32;
            let alpha = ((1.0 - dist_up).powf(0.8) * 220.0 * col_factor) as u8;
            let color = lerp_color(AURORA_CYAN, AURORA_MAGENTA, dist_up, alpha);
            buffer.blend_pixel_i32(rx, y, color);
        }

        // Downward ray: neon cyan near center transitioning to emerald green at bottom
        for y in (center.y + 1)..=(center.y + h_down) {
            let dist_down = (y - center.y) as f32 / h_down.max(1) as f32;
            let alpha = ((1.0 - dist_down).powf(0.8) * 200.0 * col_factor) as u8;
            let color = lerp_color(AURORA_CYAN, AURORA_EMERALD, dist_down, alpha);
            buffer.blend_pixel_i32(rx, y, color);
        }
    }
}

// Confetti: Clustered geometric particles (diamonds, triangles, rectangles, squares)
// in festive fiesta colors (gold, magenta, cyan, lime, orange).
fn apply_confetti(buffer: &mut PixelBuffer, center: Point, size: u32, state: &mut WackyBrushState) {
    let count = 5 + (size as usize) * 2;
    let scatter_r = (size as f32 * 5.0).max(12.0);

    for _ in 0..count {
        let rand_dist = next_lcg_rand(&mut state.step);
        let rand_angle = next_lcg_rand(&mut state.step);
        let rand_shape = next_lcg_rand(&mut state.step);
        let rand_color = next_lcg_rand(&mut state.step);

        let r = ((rand_dist % (scatter_r as u32 * 10)) as f32) / 10.0;
        let theta = (rand_angle % 3600) as f32 * (std::f32::consts::TAU / 3600.0);
        let px = center.x + (theta.cos() * r).round() as i32;
        let py = center.y + (theta.sin() * r).round() as i32;

        let color = FIESTA_COLORS[(rand_color as usize) % FIESTA_COLORS.len()];
        let s = (1 + ((rand_shape >> 8) % 3)) as i32;

        match rand_shape % 4 {
            0 => {
                // 45° tilted diamond: |dx| + |dy| <= s
                for dy in -s..=s {
                    for dx in -s..=s {
                        if dx.abs() + dy.abs() <= s {
                            buffer.set_pixel_i32(px + dx, py + dy, color);
                        }
                    }
                }
            }
            1 => {
                // Tiny right triangle
                for dy in 0..=s {
                    for dx in 0..=(s - dy) {
                        buffer.set_pixel_i32(px + dx, py + dy, color);
                    }
                }
            }
            2 => {
                // Rectangle (horizontal or vertical)
                let (w, h) = if (rand_shape >> 16).is_multiple_of(2) {
                    (s * 2, s.max(1))
                } else {
                    (s.max(1), s * 2)
                };
                for dy in 0..h {
                    for dx in 0..w {
                        buffer.set_pixel_i32(px + dx, py + dy, color);
                    }
                }
            }
            _ => {
                // Square: s x s
                for dy in 0..s {
                    for dx in 0..s {
                        buffer.set_pixel_i32(px + dx, py + dy, color);
                    }
                }
            }
        }
    }
}

// Connect-The-Dots: Spaced anchor beads linked by crisp straight Bresenham lines.
fn apply_connect_the_dots(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    let bead_r = (size as i32 + 1).clamp(2, 5);

    if let Some(prev) = state.connect_dots_last {
        if prev != center {
            let line_pts = bresenham_line(prev, center);
            for pt in line_pts {
                buffer.set_pixel_i32(pt.x, pt.y, fg);
            }
            draw_bead_marker(buffer, prev, bead_r, fg, bg);
        }
    }
    draw_bead_marker(buffer, center, bead_r, fg, bg);
    state.connect_dots_last = Some(center);
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Applies a particle or dynamic ink wacky brush point at `center`.
#[allow(clippy::too_many_arguments)]
pub fn apply_particle_brush_point(
    buffer: &mut PixelBuffer,
    center: Point,
    shape: WackyBrushShape,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    match shape {
        WackyBrushShape::LeakyPen => {
            apply_leaky_pen(buffer, center, size, fg, state);
        }
        WackyBrushShape::Bubbles => {
            apply_bubbles(buffer, center, size, fg, state);
        }
        WackyBrushShape::Splatter => {
            apply_splatter(buffer, center, size, fg, state);
        }
        WackyBrushShape::NorthernLights => {
            apply_northern_lights(buffer, center, size, state);
        }
        WackyBrushShape::Confetti => {
            apply_confetti(buffer, center, size, state);
        }
        WackyBrushShape::ConnectTheDots => {
            apply_connect_the_dots(buffer, center, size, fg, bg, state);
        }
        _ => {}
    }
    state.last_point = Some(center);
}

/// Draws an interpolated particle or dynamic ink wacky brush stroke between `p0` and `p1`.
#[allow(clippy::too_many_arguments)]
pub fn draw_particle_brush_stroke(
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
        WackyBrushShape::ConnectTheDots => {
            let bead_r = (size as i32 + 1).clamp(2, 5);
            let interval = (size as f32 * 8.0).max(18.0);
            let pts = bresenham_line(p0, p1);

            if state.connect_dots_last.is_none() {
                draw_bead_marker(buffer, p0, bead_r, fg, bg);
                state.connect_dots_last = Some(p0);
            }

            let mut last_anchor = state.connect_dots_last.unwrap_or(p0);

            for &pt in &pts {
                let dx = (pt.x - last_anchor.x) as f32;
                let dy = (pt.y - last_anchor.y) as f32;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist >= interval {
                    let line_pts = bresenham_line(last_anchor, pt);
                    for lp in line_pts {
                        buffer.set_pixel_i32(lp.x, lp.y, fg);
                    }
                    draw_bead_marker(buffer, last_anchor, bead_r, fg, bg);
                    draw_bead_marker(buffer, pt, bead_r, fg, bg);
                    last_anchor = pt;
                    state.connect_dots_last = Some(pt);
                }
            }

            if last_anchor != p1 {
                let line_pts = bresenham_line(last_anchor, p1);
                for lp in line_pts {
                    buffer.set_pixel_i32(lp.x, lp.y, fg);
                }
                draw_bead_marker(buffer, last_anchor, bead_r, fg, bg);
                draw_bead_marker(buffer, p1, bead_r, fg, bg);
                state.connect_dots_last = Some(p1);
            }
        }
        WackyBrushShape::Bubbles => {
            let pts = bresenham_line(p0, p1);
            let spacing = ((size * 2).max(6) as usize).max(1);
            if pts.is_empty() {
                apply_particle_brush_point(buffer, p0, shape, size, fg, bg, state);
            } else {
                for (i, &pt) in pts.iter().enumerate() {
                    if i % spacing == 0 || i == pts.len() - 1 {
                        apply_particle_brush_point(buffer, pt, shape, size, fg, bg, state);
                    }
                }
            }
        }
        WackyBrushShape::Splatter => {
            let pts = bresenham_line(p0, p1);
            let spacing = ((size * 3).max(6) as usize).max(1);
            if pts.is_empty() {
                apply_particle_brush_point(buffer, p0, shape, size, fg, bg, state);
            } else {
                for (i, &pt) in pts.iter().enumerate() {
                    if i % spacing == 0 || i == pts.len() - 1 {
                        apply_particle_brush_point(buffer, pt, shape, size, fg, bg, state);
                    }
                }
            }
        }
        WackyBrushShape::NorthernLights => {
            let pts = bresenham_line(p0, p1);
            let spacing = ((size * 2).max(4) as usize).max(1);
            if pts.is_empty() {
                apply_particle_brush_point(buffer, p0, shape, size, fg, bg, state);
            } else {
                for (i, &pt) in pts.iter().enumerate() {
                    if i % spacing == 0 || i == pts.len() - 1 {
                        apply_particle_brush_point(buffer, pt, shape, size, fg, bg, state);
                    }
                }
            }
        }
        WackyBrushShape::Confetti => {
            let pts = bresenham_line(p0, p1);
            let spacing = ((size * 3).max(8) as usize).max(1);
            if pts.is_empty() {
                apply_particle_brush_point(buffer, p0, shape, size, fg, bg, state);
            } else {
                for (i, &pt) in pts.iter().enumerate() {
                    if i % spacing == 0 || i == pts.len() - 1 {
                        apply_particle_brush_point(buffer, pt, shape, size, fg, bg, state);
                    }
                }
            }
        }
        WackyBrushShape::LeakyPen => {
            let pts = bresenham_line(p0, p1);
            let spacing = (size as usize).max(2);
            if pts.is_empty() {
                apply_particle_brush_point(buffer, p0, shape, size, fg, bg, state);
            } else {
                for (i, &pt) in pts.iter().enumerate() {
                    if i % spacing == 0 || i == pts.len() - 1 {
                        apply_particle_brush_point(buffer, pt, shape, size, fg, bg, state);
                    }
                }
            }
        }
        _ => {
            let pts = bresenham_line(p0, p1);
            for pt in pts {
                apply_particle_brush_point(buffer, pt, shape, size, fg, bg, state);
            }
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, BLUE, RED, WHITE};

    #[test]
    fn test_leaky_pen_ink_blot_and_gravity_drip() {
        let mut buffer = PixelBuffer::new(100, 100, WHITE);
        let mut state = WackyBrushState::default();
        let center = Point::new(50, 30);

        // First stamp: ink blot without drip
        apply_particle_brush_point(
            &mut buffer,
            center,
            WackyBrushShape::LeakyPen,
            3,
            BLACK,
            WHITE,
            &mut state,
        );

        assert_eq!(buffer.get_pixel_i32(center.x, center.y), Some(BLACK));
        assert_eq!(state.hold_ticks, 0);

        // Repeated stamps at same center simulate holding stationary
        for _ in 0..10 {
            apply_particle_brush_point(
                &mut buffer,
                center,
                WackyBrushShape::LeakyPen,
                3,
                BLACK,
                WHITE,
                &mut state,
            );
        }

        assert!(state.hold_ticks >= 10);
        // Vertical stem should exist below center
        let mut has_drip_pixel = false;
        for y in (center.y + 10)..=(center.y + 40) {
            if buffer.get_pixel_i32(center.x, y) == Some(BLACK) {
                has_drip_pixel = true;
                break;
            }
        }
        assert!(has_drip_pixel, "Expected gravity drip below ink blot");
    }

    #[test]
    fn test_bubbles_translucent_rim_and_highlight() {
        let mut buffer = PixelBuffer::new(100, 100, WHITE);
        let mut state = WackyBrushState::default();
        let center = Point::new(50, 50);

        apply_particle_brush_point(
            &mut buffer,
            center,
            WackyBrushShape::Bubbles,
            4,
            BLUE,
            WHITE,
            &mut state,
        );

        // Bubbles should leave non-white pixels
        let mut modified = 0;
        let mut has_white_highlight = false;

        for y in 30..70 {
            for x in 30..70 {
                if let Some(c) = buffer.get_pixel_i32(x, y) {
                    if c != WHITE {
                        modified += 1;
                    }
                    if c == WHITE {
                        // Check if adjacent to modified pixels (specular highlight)
                        let neighbors = [
                            buffer.get_pixel_i32(x + 1, y),
                            buffer.get_pixel_i32(x, y + 1),
                        ];
                        if neighbors
                            .iter()
                            .any(|&n| n.is_some() && n.unwrap() != WHITE)
                        {
                            has_white_highlight = true;
                        }
                    }
                }
            }
        }

        assert!(modified > 10, "Bubbles should draw rim and interior");
        assert!(
            has_white_highlight,
            "Expected white specular reflection highlight"
        );
    }

    #[test]
    fn test_splatter_ballistic_dispersion() {
        let mut buffer = PixelBuffer::new(100, 100, WHITE);
        let mut state = WackyBrushState::default();
        let center = Point::new(50, 50);

        apply_particle_brush_point(
            &mut buffer,
            center,
            WackyBrushShape::Splatter,
            3,
            RED,
            WHITE,
            &mut state,
        );

        // Core should have high density of droplets, while outer area has aerosol
        let mut inner_count = 0;
        let mut outer_count = 0;

        for y in 0..100 {
            for x in 0..100 {
                if buffer.get_pixel_i32(x, y) != Some(WHITE) {
                    let d = (((x - center.x).pow(2) + (y - center.y).pow(2)) as f32).sqrt();
                    if d <= 10.0 {
                        inner_count += 1;
                    } else if d <= 30.0 {
                        outer_count += 1;
                    }
                }
            }
        }

        assert!(inner_count > 0, "Core splatter must have droplets");
        assert!(
            outer_count > 0,
            "Ejected aerosol droplets must exist in outer radius"
        );
    }

    #[test]
    fn test_northern_lights_aurora_rays() {
        let mut buffer = PixelBuffer::new(100, 100, BLACK);
        let mut state = WackyBrushState::default();
        let center = Point::new(50, 50);

        apply_particle_brush_point(
            &mut buffer,
            center,
            WackyBrushShape::NorthernLights,
            3,
            BLACK,
            BLACK,
            &mut state,
        );

        let mut upward_pixels = 0;
        let mut downward_pixels = 0;

        for y in 10..50 {
            for x in 40..60 {
                if buffer.get_pixel_i32(x, y) != Some(BLACK) {
                    upward_pixels += 1;
                }
            }
        }

        for y in 51..90 {
            for x in 40..60 {
                if buffer.get_pixel_i32(x, y) != Some(BLACK) {
                    downward_pixels += 1;
                }
            }
        }

        assert!(upward_pixels > 0, "Upward aurora ray spikes must exist");
        assert!(downward_pixels > 0, "Downward aurora ray spikes must exist");
    }

    #[test]
    fn test_confetti_clustered_geometric_particles() {
        let mut buffer = PixelBuffer::new(100, 100, WHITE);
        let mut state = WackyBrushState::default();
        let center = Point::new(50, 50);

        apply_particle_brush_point(
            &mut buffer,
            center,
            WackyBrushShape::Confetti,
            3,
            BLACK,
            WHITE,
            &mut state,
        );

        let mut found_fiesta_colors = 0;
        for &fc in &FIESTA_COLORS {
            let mut found = false;
            for y in 30..70 {
                for x in 30..70 {
                    if buffer.get_pixel_i32(x, y) == Some(fc) {
                        found = true;
                        break;
                    }
                }
                if found {
                    break;
                }
            }
            if found {
                found_fiesta_colors += 1;
            }
        }

        assert!(
            found_fiesta_colors >= 2,
            "Confetti should contain multiple festive fiesta colors"
        );
    }

    #[test]
    fn test_connect_the_dots_spaced_anchors() {
        let mut buffer = PixelBuffer::new(150, 60, WHITE);
        let mut state = WackyBrushState::default();
        let p0 = Point::new(20, 30);
        let p1 = Point::new(130, 30);

        draw_particle_brush_stroke(
            &mut buffer,
            p0,
            p1,
            WackyBrushShape::ConnectTheDots,
            3,
            BLACK,
            WHITE,
            &mut state,
        );

        assert_eq!(state.connect_dots_last, Some(p1));

        // Start bead, line, and end bead should exist
        assert_eq!(buffer.get_pixel_i32(p0.x, p0.y), Some(BLACK));
        assert_eq!(buffer.get_pixel_i32(p1.x, p1.y), Some(BLACK));
        assert_eq!(buffer.get_pixel_i32((p0.x + p1.x) / 2, p0.y), Some(BLACK));
    }

    #[test]
    fn test_all_particle_brushes_stroke_non_empty() {
        let shapes = [
            WackyBrushShape::LeakyPen,
            WackyBrushShape::Bubbles,
            WackyBrushShape::Splatter,
            WackyBrushShape::NorthernLights,
            WackyBrushShape::Confetti,
            WackyBrushShape::ConnectTheDots,
        ];

        for shape in shapes {
            let mut buffer = PixelBuffer::new(80, 80, WHITE);
            let mut state = WackyBrushState::default();
            draw_particle_brush_stroke(
                &mut buffer,
                Point::new(20, 20),
                Point::new(60, 60),
                shape,
                3,
                RED,
                WHITE,
                &mut state,
            );

            let non_white_count = (0..80)
                .flat_map(|y| (0..80).map(move |x| (x, y)))
                .filter(|&(x, y)| buffer.get_pixel_i32(x, y) != Some(WHITE))
                .count();

            assert!(
                non_white_count > 0,
                "Stroke for {:?} should produce pixels on buffer",
                shape
            );
        }
    }
}

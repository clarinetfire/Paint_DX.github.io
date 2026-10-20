//! Procedural and Geometric Wacky Brushes.
//!
//! Crisp pixel algorithms for:
//! 1. Pine Tree (WackyBrushShape::PineTree)
//! 2. Concentric Circles (WackyBrushShape::ConcentricCircles)
//! 3. Starburst (WackyBrushShape::Starburst)
//! 4. Swirl (WackyBrushShape::Swirl)
//! 5. Looper / Ribbon (WackyBrushShape::LooperRibbon)
//! 6. Zig-Zag (WackyBrushShape::ZigZag)

use super::{WackyBrushShape, WackyBrushState};
use crate::geometry::{bresenham_line, Point};
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;

/// Color of pine needle foliage at terminal twigs of Pine Tree brush.
pub const PINE_NEEDLE_GREEN: ColorRgba = ColorRgba::rgb(34, 145, 55);

// ============================================================================
// Bresenham Midpoint Circle Outline
// ============================================================================

/// Draws a pixel-crisp 1-pixel circle outline using the midpoint circle algorithm
/// to guarantee zero gaps and zero anti-aliasing.
pub fn draw_midpoint_circle_outline(
    buffer: &mut PixelBuffer,
    center: Point,
    radius: i32,
    color: ColorRgba,
) {
    if radius <= 0 {
        buffer.set_pixel_i32(center.x, center.y, color);
        return;
    }

    let mut x = 0;
    let mut y = radius;
    let mut d = 1 - radius;

    let mut plot8 = |x: i32, y: i32| {
        buffer.set_pixel_i32(center.x + x, center.y + y, color);
        buffer.set_pixel_i32(center.x - x, center.y + y, color);
        buffer.set_pixel_i32(center.x + x, center.y - y, color);
        buffer.set_pixel_i32(center.x - x, center.y - y, color);
        buffer.set_pixel_i32(center.x + y, center.y + x, color);
        buffer.set_pixel_i32(center.x - y, center.y + x, color);
        buffer.set_pixel_i32(center.x + y, center.y - x, color);
        buffer.set_pixel_i32(center.x - y, center.y - x, color);
    };

    plot8(x, y);

    while x < y {
        x += 1;
        if d < 0 {
            d += 2 * x + 1;
        } else {
            y -= 1;
            d += 2 * (x - y) + 1;
        }
        plot8(x, y);
    }
}

// ============================================================================
// 1. Pine Tree
// ============================================================================

fn draw_pine_branch(
    buffer: &mut PixelBuffer,
    start: Point,
    angle: f32,
    length: f32,
    depth: u32,
    fg: ColorRgba,
) {
    if length < 2.5 || depth == 0 {
        // Terminal twigs decorated with 2-3 green pine needle pixels
        buffer.set_pixel_i32(start.x, start.y, PINE_NEEDLE_GREEN);
        buffer.set_pixel_i32(start.x - 1, start.y - 1, PINE_NEEDLE_GREEN);
        buffer.set_pixel_i32(start.x + 1, start.y - 1, PINE_NEEDLE_GREEN);
        return;
    }

    let end_x = (start.x as f32 + length * angle.cos()).round() as i32;
    let end_y = (start.y as f32 + length * angle.sin()).round() as i32;
    let end_pt = Point::new(end_x, end_y);

    for pt in bresenham_line(start, end_pt) {
        buffer.set_pixel_i32(pt.x, pt.y, fg);
    }

    // Sub-twigs with decaying length (length * 0.65)
    let sub_len = length * 0.65;
    let sub_angle_delta = 38.0_f32.to_radians(); // 35°-45° branching angle

    draw_pine_branch(
        buffer,
        end_pt,
        angle - sub_angle_delta,
        sub_len,
        depth - 1,
        fg,
    );
    draw_pine_branch(
        buffer,
        end_pt,
        angle + sub_angle_delta,
        sub_len,
        depth - 1,
        fg,
    );
}

fn apply_pine_tree(buffer: &mut PixelBuffer, center: Point, size: u32, fg: ColorRgba) {
    let trunk_len = ((size as f32) * 4.5 + 8.0).max(10.0);
    let trunk_top_y = (center.y as f32 - trunk_len).round() as i32;

    // Draw central trunk base
    for y in trunk_top_y..=center.y {
        buffer.set_pixel_i32(center.x, y, fg);
    }
    if size >= 3 {
        let base_thicken_start = (center.y as f32 - trunk_len * 0.35).round() as i32;
        for y in base_thicken_start..=center.y {
            buffer.set_pixel_i32(center.x + 1, y, fg);
        }
    }

    // Alternating branches at 35°-45° angles along the central trunk
    let branch_angle = 40.0_f32.to_radians();
    let branch_levels = [
        (0.25_f32, -1.0_f32, trunk_len * 0.55),
        (0.40_f32, 1.0_f32, trunk_len * 0.50),
        (0.55_f32, -1.0_f32, trunk_len * 0.45),
        (0.70_f32, 1.0_f32, trunk_len * 0.40),
        (0.85_f32, -1.0_f32, trunk_len * 0.32),
        (0.92_f32, 1.0_f32, trunk_len * 0.25),
    ];

    for (height_ratio, side, branch_len) in branch_levels {
        let branch_start_y = (center.y as f32 - trunk_len * height_ratio).round() as i32;
        let branch_start = Point::new(center.x, branch_start_y);
        let angle = -std::f32::consts::FRAC_PI_2 + side * branch_angle;
        draw_pine_branch(buffer, branch_start, angle, branch_len, 2, fg);
    }

    // Apical leader at trunk tip
    let tip = Point::new(center.x, trunk_top_y);
    draw_pine_branch(
        buffer,
        tip,
        -std::f32::consts::FRAC_PI_2,
        trunk_len * 0.3,
        1,
        fg,
    );
}

// ============================================================================
// 2. Concentric Circles
// ============================================================================

fn apply_concentric_circles(buffer: &mut PixelBuffer, center: Point, size: u32, fg: ColorRgba) {
    let step_r = (size as i32).clamp(2, 5);
    let num_rings = (3 + (size % 3)).min(5) as i32;

    buffer.set_pixel_i32(center.x, center.y, fg);
    if step_r > 2 {
        buffer.set_pixel_i32(center.x + 1, center.y, fg);
        buffer.set_pixel_i32(center.x - 1, center.y, fg);
        buffer.set_pixel_i32(center.x, center.y + 1, fg);
        buffer.set_pixel_i32(center.x, center.y - 1, fg);
    }

    for ring in 1..=num_rings {
        let radius = ring * step_r;
        draw_midpoint_circle_outline(buffer, center, radius, fg);
    }
}

// ============================================================================
// 3. Starburst
// ============================================================================

fn apply_starburst(buffer: &mut PixelBuffer, center: Point, size: u32, fg: ColorRgba) {
    let num_spikes = if size <= 2 {
        8
    } else if size <= 4 {
        12
    } else {
        16
    };

    let r_max = ((size as f32) * 3.5 + 7.0).max(8.0);

    // Center core
    buffer.set_pixel_i32(center.x, center.y, fg);
    if size >= 3 {
        buffer.set_pixel_i32(center.x + 1, center.y, fg);
        buffer.set_pixel_i32(center.x - 1, center.y, fg);
        buffer.set_pixel_i32(center.x, center.y + 1, fg);
        buffer.set_pixel_i32(center.x, center.y - 1, fg);
    }

    for i in 0..num_spikes {
        let angle = (i as f32) * (2.0 * std::f32::consts::PI / num_spikes as f32);
        let length_factor = if num_spikes == 16 {
            match i % 4 {
                0 => 1.0,
                2 => 0.72,
                _ => 0.48,
            }
        } else if num_spikes == 12 {
            match i % 3 {
                0 => 1.0,
                1 => 0.55,
                _ => 0.75,
            }
        } else if i % 2 == 0 {
            1.0
        } else {
            0.58
        };

        let spike_len = r_max * length_factor;
        let tip = Point::new(
            (center.x as f32 + spike_len * angle.cos()).round() as i32,
            (center.y as f32 + spike_len * angle.sin()).round() as i32,
        );

        for pt in bresenham_line(center, tip) {
            buffer.set_pixel_i32(pt.x, pt.y, fg);
        }
    }
}

// ============================================================================
// 4. Swirl
// ============================================================================

fn apply_swirl(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    state: &WackyBrushState,
) {
    let num_arms = 3;
    let base_phase = (state.step as f32) * 0.4;
    let max_r = ((size as f32) * 3.5 + 6.0).max(8.0);
    let max_theta = 1.25 * 2.0 * std::f32::consts::PI;
    let steps = (max_r * 3.5).max(20.0) as usize;

    buffer.set_pixel_i32(center.x, center.y, fg);

    for k in 0..num_arms {
        let arm_offset = base_phase + (k as f32) * (2.0 * std::f32::consts::PI / num_arms as f32);
        let mut prev = center;
        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            let theta = t * max_theta;
            let r = max_r * t;
            let current_angle = arm_offset + theta;
            let pt = Point::new(
                (center.x as f32 + r * current_angle.cos()).round() as i32,
                (center.y as f32 + r * current_angle.sin()).round() as i32,
            );
            for line_pt in bresenham_line(prev, pt) {
                buffer.set_pixel_i32(line_pt.x, line_pt.y, fg);
            }
            prev = pt;
        }
    }
}

// ============================================================================
// 5. Looper / Ribbon
// ============================================================================

fn draw_ribbon_step(
    buffer: &mut PixelBuffer,
    prev_pt: Point,
    curr_pt: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
) {
    state.ribbon_trail.push_back(curr_pt);
    if state.ribbon_trail.len() > 32 {
        state.ribbon_trail.pop_front();
    }

    let dx = (curr_pt.x - prev_pt.x) as f32;
    let dy = (curr_pt.y - prev_pt.y) as f32;
    let dist = (dx * dx + dy * dy).sqrt();

    let (nx, ny) = if dist > 0.001 {
        (-dy / dist, dx / dist)
    } else {
        (0.0, 1.0)
    };

    let num_strands: i32 = (size.clamp(2, 5)) as i32;
    let half_strands = num_strands / 2;
    let strand_spacing = ((size as f32) * 1.5).max(2.5);

    for s in -half_strands..=half_strands {
        let s_f = s as f32;
        let phase_curr = (state.step as f32) * 0.25 + s_f * 0.7;
        let phase_prev = ((state.step.wrapping_sub(1)) as f32) * 0.25 + s_f * 0.7;

        let twist_curr = phase_curr.sin();
        let twist_prev = phase_prev.sin();

        let expansion_curr = 1.0 + 0.3 * (phase_curr * 0.5).sin();
        let expansion_prev = 1.0 + 0.3 * (phase_prev * 0.5).sin();

        let off_curr = s_f * strand_spacing * expansion_curr * twist_curr;
        let off_prev = s_f * strand_spacing * expansion_prev * twist_prev;

        let p_prev_strand = Point::new(
            (prev_pt.x as f32 + nx * off_prev).round() as i32,
            (prev_pt.y as f32 + ny * off_prev).round() as i32,
        );
        let p_curr_strand = Point::new(
            (curr_pt.x as f32 + nx * off_curr).round() as i32,
            (curr_pt.y as f32 + ny * off_curr).round() as i32,
        );

        for pt in bresenham_line(p_prev_strand, p_curr_strand) {
            buffer.set_pixel_i32(pt.x, pt.y, fg);
        }
    }
}

fn apply_ribbon_point(
    buffer: &mut PixelBuffer,
    center: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
) {
    if let Some(prev) = state.ribbon_trail.back().copied() {
        if prev != center {
            draw_ribbon_step(buffer, prev, center, size, fg, state);
            return;
        }
    }

    state.ribbon_trail.push_back(center);
    if state.ribbon_trail.len() > 32 {
        state.ribbon_trail.pop_front();
    }

    // Single click rosette loop
    let loop_r = ((size as f32) * 2.5 + 4.0).max(5.0);
    let num_petals = 4;
    for i in 0..num_petals {
        let angle = (i as f32) * (2.0 * std::f32::consts::PI / num_petals as f32)
            + (state.step as f32) * 0.35;
        let tip = Point::new(
            (center.x as f32 + loop_r * angle.cos()).round() as i32,
            (center.y as f32 + loop_r * angle.sin()).round() as i32,
        );
        for pt in bresenham_line(center, tip) {
            buffer.set_pixel_i32(pt.x, pt.y, fg);
        }
    }
    buffer.set_pixel_i32(center.x, center.y, fg);
}

fn draw_ribbon_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
) {
    if p0 == p1 {
        apply_ribbon_point(buffer, p0, size, fg, state);
        return;
    }

    let pts = bresenham_line(p0, p1);
    let step_size = (size as usize / 2).clamp(1, 3);
    let mut prev_pt = p0;

    for pt in pts.into_iter().step_by(step_size) {
        if pt == prev_pt && !state.ribbon_trail.is_empty() {
            continue;
        }
        state.step = state.step.wrapping_add(1);
        draw_ribbon_step(buffer, prev_pt, pt, size, fg, state);
        prev_pt = pt;
    }

    if prev_pt != p1 {
        state.step = state.step.wrapping_add(1);
        draw_ribbon_step(buffer, prev_pt, p1, size, fg, state);
    }
}

// ============================================================================
// 6. Zig-Zag
// ============================================================================

fn apply_zigzag_point(buffer: &mut PixelBuffer, center: Point, size: u32, fg: ColorRgba) {
    let r = ((size as i32) * 3).max(6);
    let half_r = (r / 2).max(2);
    let vertices = [
        Point::new(center.x - half_r, center.y - r),
        Point::new(center.x + half_r, center.y - r / 4),
        Point::new(center.x - half_r, center.y + r / 4),
        Point::new(center.x + half_r, center.y + r),
    ];
    for w in vertices.windows(2) {
        for pt in bresenham_line(w[0], w[1]) {
            buffer.set_pixel_i32(pt.x, pt.y, fg);
        }
    }
}

fn draw_zigzag_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    size: u32,
    fg: ColorRgba,
    state: &mut WackyBrushState,
) {
    if p0 == p1 {
        apply_zigzag_point(buffer, p0, size, fg);
        return;
    }

    let dx = (p1.x - p0.x) as f32;
    let dy = (p1.y - p0.y) as f32;
    let dist = (dx * dx + dy * dy).sqrt();

    let ux = dx / dist;
    let uy = dy / dist;
    let nx = -uy;
    let ny = ux;

    let amplitude = ((size as f32) * 2.5 + 4.0).max(5.0);
    let half_wavelength = ((size as f32) * 2.0 + 4.0).max(5.0);
    let num_teeth = ((dist / half_wavelength).round() as usize).max(2);

    let mut vertices = Vec::with_capacity(num_teeth + 1);
    vertices.push(p0);

    for i in 1..num_teeth {
        let t = i as f32 / num_teeth as f32;
        let base_x = p0.x as f32 + t * dx;
        let base_y = p0.y as f32 + t * dy;
        let sign = if (state.step as usize + i) % 2 == 1 {
            1.0
        } else {
            -1.0
        };
        let vx = (base_x + sign * amplitude * nx).round() as i32;
        let vy = (base_y + sign * amplitude * ny).round() as i32;
        vertices.push(Point::new(vx, vy));
    }
    vertices.push(p1);

    state.step = state.step.wrapping_add(num_teeth as u64);

    for w in vertices.windows(2) {
        for pt in bresenham_line(w[0], w[1]) {
            buffer.set_pixel_i32(pt.x, pt.y, fg);
        }
    }
}

// ============================================================================
// Public Dispatch Functions
// ============================================================================

/// Applies a single procedural or geometric wacky brush stamp/point.
#[allow(clippy::too_many_arguments)]
pub fn apply_geometric_brush_point(
    buffer: &mut PixelBuffer,
    center: Point,
    shape: WackyBrushShape,
    size: u32,
    fg: ColorRgba,
    _bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    match shape {
        WackyBrushShape::PineTree => {
            apply_pine_tree(buffer, center, size, fg);
        }
        WackyBrushShape::ConcentricCircles => {
            apply_concentric_circles(buffer, center, size, fg);
        }
        WackyBrushShape::Starburst => {
            apply_starburst(buffer, center, size, fg);
        }
        WackyBrushShape::Swirl => {
            apply_swirl(buffer, center, size, fg, state);
        }
        WackyBrushShape::LooperRibbon => {
            apply_ribbon_point(buffer, center, size, fg, state);
        }
        WackyBrushShape::ZigZag => {
            if let Some(last) = state.last_point {
                let dist_sq = (last.x - center.x).pow(2) + (last.y - center.y).pow(2);
                if last != center && dist_sq < 100 * 100 {
                    draw_zigzag_stroke(buffer, last, center, size, fg, state);
                    return;
                }
            }
            apply_zigzag_point(buffer, center, size, fg);
        }
        _ => {}
    }
}

/// Draws an interpolated geometric wacky brush stroke.
#[allow(clippy::too_many_arguments)]
pub fn draw_geometric_brush_stroke(
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
        WackyBrushShape::ZigZag => {
            draw_zigzag_stroke(buffer, p0, p1, size, fg, state);
        }
        WackyBrushShape::LooperRibbon => {
            draw_ribbon_stroke(buffer, p0, p1, size, fg, state);
        }
        WackyBrushShape::PineTree
        | WackyBrushShape::ConcentricCircles
        | WackyBrushShape::Starburst
        | WackyBrushShape::Swirl => {
            if p0 == p1 {
                apply_geometric_brush_point(buffer, p0, shape, size, fg, bg, state);
                return;
            }

            let dist = (((p1.x - p0.x).pow(2) + (p1.y - p0.y).pow(2)) as f32).sqrt();
            let spacing = match shape {
                WackyBrushShape::PineTree => ((size as f32) * 5.0 + 8.0).max(14.0),
                WackyBrushShape::ConcentricCircles => ((size as f32) * 6.0 + 10.0).max(18.0),
                WackyBrushShape::Starburst => ((size as f32) * 5.0 + 10.0).max(16.0),
                WackyBrushShape::Swirl => ((size as f32) * 5.0 + 8.0).max(14.0),
                _ => 16.0,
            };

            let count = ((dist / spacing).round() as usize).max(1);
            for i in 0..=count {
                let t = i as f32 / count as f32;
                let x = (p0.x as f32 + t * (p1.x - p0.x) as f32).round() as i32;
                let y = (p0.y as f32 + t * (p1.y - p0.y) as f32).round() as i32;
                state.step = state.step.wrapping_add(1);
                apply_geometric_brush_point(buffer, Point::new(x, y), shape, size, fg, bg, state);
            }
        }
        _ => {}
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, RED, WHITE};

    fn count_colored_pixels(buffer: &PixelBuffer, target_color: ColorRgba) -> usize {
        let mut count = 0;
        for y in 0..buffer.height() {
            for x in 0..buffer.width() {
                if buffer.get_pixel(x, y) == Some(target_color) {
                    count += 1;
                }
            }
        }
        count
    }

    #[test]
    fn test_pine_tree_point_and_stroke() {
        let mut buffer = PixelBuffer::new(60, 60, WHITE);
        let mut state = WackyBrushState::default();

        apply_geometric_brush_point(
            &mut buffer,
            Point::new(30, 50),
            WackyBrushShape::PineTree,
            3,
            BLACK,
            WHITE,
            &mut state,
        );

        let black_count = count_colored_pixels(&buffer, BLACK);
        let needle_count = count_colored_pixels(&buffer, PINE_NEEDLE_GREEN);

        assert!(
            black_count > 15,
            "Pine tree trunk and branches should be drawn"
        );
        assert!(
            needle_count >= 2,
            "Pine tree must have green needle pixels decorated"
        );

        // Stroke test
        let mut stroke_buf = PixelBuffer::new(100, 60, WHITE);
        let mut stroke_state = WackyBrushState::default();
        draw_geometric_brush_stroke(
            &mut stroke_buf,
            Point::new(20, 45),
            Point::new(80, 45),
            WackyBrushShape::PineTree,
            3,
            BLACK,
            WHITE,
            &mut stroke_state,
        );

        let stroke_needles = count_colored_pixels(&stroke_buf, PINE_NEEDLE_GREEN);
        assert!(
            stroke_needles > needle_count,
            "Stroke should stamp multiple trees"
        );
    }

    #[test]
    fn test_concentric_circles_point_and_stroke() {
        let mut buffer = PixelBuffer::new(60, 60, WHITE);
        let mut state = WackyBrushState::default();

        apply_geometric_brush_point(
            &mut buffer,
            Point::new(30, 30),
            WackyBrushShape::ConcentricCircles,
            3,
            RED,
            WHITE,
            &mut state,
        );

        let red_count = count_colored_pixels(&buffer, RED);
        assert!(
            red_count > 30,
            "Concentric circles should produce multi-ring target"
        );

        // Verify center pixel is set
        assert_eq!(buffer.get_pixel(30, 30), Some(RED));

        // Stroke test
        let mut stroke_buf = PixelBuffer::new(120, 60, WHITE);
        let mut stroke_state = WackyBrushState::default();
        draw_geometric_brush_stroke(
            &mut stroke_buf,
            Point::new(20, 30),
            Point::new(100, 30),
            WackyBrushShape::ConcentricCircles,
            3,
            RED,
            WHITE,
            &mut stroke_state,
        );

        assert!(count_colored_pixels(&stroke_buf, RED) > red_count * 2);
    }

    #[test]
    fn test_starburst_point_and_stroke() {
        let mut buffer = PixelBuffer::new(60, 60, WHITE);
        let mut state = WackyBrushState::default();

        apply_geometric_brush_point(
            &mut buffer,
            Point::new(30, 30),
            WackyBrushShape::Starburst,
            3,
            BLACK,
            WHITE,
            &mut state,
        );

        let black_count = count_colored_pixels(&buffer, BLACK);
        assert!(black_count > 40, "Starburst spikes should be drawn");
        assert_eq!(buffer.get_pixel(30, 30), Some(BLACK));

        // Stroke test
        let mut stroke_buf = PixelBuffer::new(100, 60, WHITE);
        let mut stroke_state = WackyBrushState::default();
        draw_geometric_brush_stroke(
            &mut stroke_buf,
            Point::new(20, 30),
            Point::new(80, 30),
            WackyBrushShape::Starburst,
            3,
            BLACK,
            WHITE,
            &mut stroke_state,
        );

        assert!(count_colored_pixels(&stroke_buf, BLACK) > black_count);
    }

    #[test]
    fn test_swirl_point_and_stroke() {
        let mut buffer = PixelBuffer::new(60, 60, WHITE);
        let mut state = WackyBrushState::default();

        apply_geometric_brush_point(
            &mut buffer,
            Point::new(30, 30),
            WackyBrushShape::Swirl,
            3,
            RED,
            WHITE,
            &mut state,
        );

        let red_count1 = count_colored_pixels(&buffer, RED);
        assert!(red_count1 > 30, "Swirl spiral arms should be drawn");

        // Rotation phase variation with step
        let mut buffer2 = PixelBuffer::new(60, 60, WHITE);
        let mut state2 = WackyBrushState::default();
        state2.step = 5;
        apply_geometric_brush_point(
            &mut buffer2,
            Point::new(30, 30),
            WackyBrushShape::Swirl,
            3,
            RED,
            WHITE,
            &mut state2,
        );

        let mut diff = 0;
        for y in 0..60 {
            for x in 0..60 {
                if buffer.get_pixel(x, y) != buffer2.get_pixel(x, y) {
                    diff += 1;
                }
            }
        }
        assert!(diff > 0, "Swirl should rotate when state.step varies");
    }

    #[test]
    fn test_looper_ribbon_point_and_stroke() {
        let mut buffer = PixelBuffer::new(80, 80, WHITE);
        let mut state = WackyBrushState::default();

        // Initial point creates loop
        apply_geometric_brush_point(
            &mut buffer,
            Point::new(40, 40),
            WackyBrushShape::LooperRibbon,
            3,
            BLACK,
            WHITE,
            &mut state,
        );

        assert!(count_colored_pixels(&buffer, BLACK) > 10);
        assert!(
            !state.ribbon_trail.is_empty(),
            "Ribbon trail must record point"
        );

        // Ribbon stroke
        let mut stroke_buf = PixelBuffer::new(100, 80, WHITE);
        let mut stroke_state = WackyBrushState::default();
        draw_geometric_brush_stroke(
            &mut stroke_buf,
            Point::new(15, 20),
            Point::new(85, 60),
            WackyBrushShape::LooperRibbon,
            3,
            BLACK,
            WHITE,
            &mut stroke_state,
        );

        let stroke_pixels = count_colored_pixels(&stroke_buf, BLACK);
        assert!(
            stroke_pixels > 60,
            "Ribbon strands must be drawn continuously"
        );
    }

    #[test]
    fn test_zigzag_point_and_stroke() {
        let mut buffer = PixelBuffer::new(60, 60, WHITE);
        let mut state = WackyBrushState::default();

        apply_geometric_brush_point(
            &mut buffer,
            Point::new(30, 30),
            WackyBrushShape::ZigZag,
            3,
            RED,
            WHITE,
            &mut state,
        );

        let point_pixels = count_colored_pixels(&buffer, RED);
        assert!(point_pixels >= 12, "ZigZag point glyph should be rendered");

        // Stroke with acute teeth perpendicular to motion
        let mut stroke_buf = PixelBuffer::new(100, 60, WHITE);
        let mut stroke_state = WackyBrushState::default();
        draw_geometric_brush_stroke(
            &mut stroke_buf,
            Point::new(10, 30),
            Point::new(90, 30),
            WackyBrushShape::ZigZag,
            3,
            RED,
            WHITE,
            &mut stroke_state,
        );

        let stroke_pixels = count_colored_pixels(&stroke_buf, RED);
        assert!(stroke_pixels > 40, "Zigzag stroke teeth should be drawn");

        // Endpoints p0 and p1 must be connected
        assert_eq!(stroke_buf.get_pixel(10, 30), Some(RED));
        assert_eq!(stroke_buf.get_pixel(90, 30), Some(RED));
    }

    #[test]
    fn test_all_geometric_shapes_bounds_safety() {
        let shapes = [
            WackyBrushShape::PineTree,
            WackyBrushShape::ConcentricCircles,
            WackyBrushShape::Starburst,
            WackyBrushShape::Swirl,
            WackyBrushShape::LooperRibbon,
            WackyBrushShape::ZigZag,
        ];

        let mut buffer = PixelBuffer::new(30, 30, WHITE);
        let mut state = WackyBrushState::default();

        for shape in shapes {
            // Draw near borders and negative coords to test safety
            apply_geometric_brush_point(
                &mut buffer,
                Point::new(0, 0),
                shape,
                4,
                BLACK,
                WHITE,
                &mut state,
            );
            apply_geometric_brush_point(
                &mut buffer,
                Point::new(29, 29),
                shape,
                4,
                BLACK,
                WHITE,
                &mut state,
            );
            draw_geometric_brush_stroke(
                &mut buffer,
                Point::new(-10, -5),
                Point::new(40, 35),
                shape,
                4,
                BLACK,
                WHITE,
                &mut state,
            );
        }
    }
}

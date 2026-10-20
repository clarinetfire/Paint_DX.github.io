//! Airbrush (Spray paint) tool with 3 classic sizes.

use crate::geometry::Point;
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AirbrushSize {
    Small, // Radius ~4px
    #[default]
    Medium, // Radius ~8px
    Large, // Radius ~12px
}

impl AirbrushSize {
    pub const ALL: [AirbrushSize; 3] = [
        AirbrushSize::Small,
        AirbrushSize::Medium,
        AirbrushSize::Large,
    ];

    pub fn radius(&self) -> i32 {
        match self {
            AirbrushSize::Small => 4,
            AirbrushSize::Medium => 8,
            AirbrushSize::Large => 12,
        }
    }

    pub fn dots_per_spray(&self) -> usize {
        match self {
            AirbrushSize::Small => 8,
            AirbrushSize::Medium => 16,
            AirbrushSize::Large => 24,
        }
    }
}

/// Applies a spray burst around `center` with a deterministic LCG pseudo-random distribution.
pub fn apply_airbrush_spray(
    buffer: &mut PixelBuffer,
    center: Point,
    size: AirbrushSize,
    color: ColorRgba,
    spray_step: u32,
) {
    let r = size.radius() as f64;
    let count = size.dots_per_spray();

    // Simple deterministic LCG random generator based on coordinates and step
    let mut state = (center.x as u64)
        .wrapping_mul(31)
        .wrapping_add(center.y as u64 * 37)
        .wrapping_add(spray_step as u64 * 101)
        .wrapping_add(12345);

    for _ in 0..count {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand_radius = ((state & 0xFFFF) as f64 / 65535.0).sqrt() * r;
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand_angle = ((state & 0xFFFF) as f64 / 65535.0) * 2.0 * std::f64::consts::PI;

        let dx = (rand_radius * rand_angle.cos()).round() as i32;
        let dy = (rand_radius * rand_angle.sin()).round() as i32;

        buffer.set_pixel_i32(center.x + dx, center.y + dy, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, WHITE};

    #[test]
    fn test_airbrush_spray() {
        let mut buffer = PixelBuffer::new(30, 30, WHITE);
        apply_airbrush_spray(
            &mut buffer,
            Point::new(15, 15),
            AirbrushSize::Medium,
            BLACK,
            1,
        );

        let mut drawn_pixels = 0;
        for y in 0..30 {
            for x in 0..30 {
                if buffer.get_pixel_i32(x, y) == Some(BLACK) {
                    drawn_pixels += 1;
                }
            }
        }
        assert!(drawn_pixels > 0 && drawn_pixels <= AirbrushSize::Medium.dots_per_spray());
    }
}

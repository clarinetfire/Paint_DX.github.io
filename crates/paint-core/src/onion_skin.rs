//! Onion skinning compositing engine.
//!
//! Provides ghosting/preview of previous and next animation frames with
//! configurable frame counts, alpha falloff, and red/blue tinting.

use crate::frames::AnimationDocument;
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OnionSkinTint {
    #[default]
    None,
    RedBlue, // Previous frames tinted Red, Next frames tinted Blue
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnionSkinConfig {
    pub enabled: bool,
    pub prev_frames: usize,
    pub next_frames: usize,
    pub opacity: u8, // Base opacity for 1-step offset (0..=100)
    pub tint_mode: OnionSkinTint,
    pub loop_bounds: bool,
}

impl Default for OnionSkinConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            prev_frames: 1,
            next_frames: 1,
            opacity: 40,
            tint_mode: OnionSkinTint::RedBlue,
            loop_bounds: false,
        }
    }
}

/// Applies a tint color and opacity multiplier to a PixelBuffer.
pub fn apply_tint_and_opacity(
    buffer: &PixelBuffer,
    opacity_pct: u8,
    tint_color: Option<ColorRgba>,
) -> PixelBuffer {
    let mut result = PixelBuffer::new(buffer.width(), buffer.height(), ColorRgba::TRANSPARENT);
    let factor = opacity_pct as f32 / 100.0;

    for y in 0..buffer.height() {
        for x in 0..buffer.width() {
            let Some(p) = buffer.get_pixel(x, y) else {
                continue;
            };
            if p.a == 0 {
                continue;
            }

            let new_a = ((p.a as f32) * factor).round().clamp(0.0, 255.0) as u8;
            if new_a == 0 {
                continue;
            }

            let colored_pixel = if let Some(tint) = tint_color {
                // Blend original pixel color with tint color (50% tint)
                let r = ((p.r as u16 + tint.r as u16) / 2) as u8;
                let g = ((p.g as u16 + tint.g as u16) / 2) as u8;
                let b = ((p.b as u16 + tint.b as u16) / 2) as u8;
                ColorRgba::rgba(r, g, b, new_a)
            } else {
                ColorRgba::rgba(p.r, p.g, p.b, new_a)
            };

            result.set_pixel(x, y, colored_pixel);
        }
    }

    result
}

/// Composites the active frame along with previous and next onion skin ghost frames.
pub fn composite_with_onion_skin(
    doc: &AnimationDocument,
    active_frame_idx: usize,
    config: &OnionSkinConfig,
) -> PixelBuffer {
    let width = doc.width;
    let height = doc.height;
    let mut composite = PixelBuffer::new(width, height, ColorRgba::TRANSPARENT);

    let total_frames = doc.frames.len();
    if total_frames == 0 {
        return composite;
    }

    if !config.enabled || (config.prev_frames == 0 && config.next_frames == 0) {
        return doc.composite_frame(active_frame_idx);
    }

    // 1. Composite previous ghost frames (drawn first, behind active frame)
    for offset in (1..=config.prev_frames).rev() {
        let frame_idx = if active_frame_idx >= offset {
            Some(active_frame_idx - offset)
        } else if config.loop_bounds && total_frames > 0 {
            Some((total_frames + active_frame_idx - (offset % total_frames)) % total_frames)
        } else {
            None
        };

        if let Some(f_idx) = frame_idx {
            if f_idx < total_frames && f_idx != active_frame_idx {
                let frame_buf = doc.composite_frame(f_idx);
                // Opacity falls off with distance
                let dist_factor = 1.0 / (offset as f32);
                let op = ((config.opacity as f32 * dist_factor).round() as u8).clamp(5, 100);
                let tint = match config.tint_mode {
                    OnionSkinTint::RedBlue => Some(ColorRgba::rgb(255, 60, 60)), // Red for past
                    OnionSkinTint::None => None,
                };
                let ghost = apply_tint_and_opacity(&frame_buf, op, tint);
                composite.blend_buffer(&ghost, 0, 0);
            }
        }
    }

    // 2. Composite next ghost frames (also behind active frame, but in front of older past frames)
    for offset in (1..=config.next_frames).rev() {
        let frame_idx = if active_frame_idx + offset < total_frames {
            Some(active_frame_idx + offset)
        } else if config.loop_bounds && total_frames > 0 {
            Some((active_frame_idx + offset) % total_frames)
        } else {
            None
        };

        if let Some(f_idx) = frame_idx {
            if f_idx < total_frames && f_idx != active_frame_idx {
                let frame_buf = doc.composite_frame(f_idx);
                let dist_factor = 1.0 / (offset as f32);
                let op = ((config.opacity as f32 * dist_factor).round() as u8).clamp(5, 100);
                let tint = match config.tint_mode {
                    OnionSkinTint::RedBlue => Some(ColorRgba::rgb(60, 120, 255)), // Blue for future
                    OnionSkinTint::None => None,
                };
                let ghost = apply_tint_and_opacity(&frame_buf, op, tint);
                composite.blend_buffer(&ghost, 0, 0);
            }
        }
    }

    // 3. Composite active frame on top (crisp, 100% full opacity)
    let active_buf = doc.composite_frame(active_frame_idx);
    composite.blend_buffer(&active_buf, 0, 0);

    composite
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::LayerId;
    use crate::palette::{BLACK, RED, WHITE};

    #[test]
    fn test_apply_tint_and_opacity() {
        let buf = PixelBuffer::new(2, 2, WHITE);
        let ghost = apply_tint_and_opacity(&buf, 50, Some(RED));
        let p = ghost.get_pixel(0, 0).unwrap();
        assert_eq!(p.a, 128);
        assert!(p.r > 200);
    }

    #[test]
    fn test_onion_skin_disabled() {
        let doc = AnimationDocument::new(10, 10, WHITE);
        let active = doc.composite_frame(0);
        let config = OnionSkinConfig {
            enabled: false,
            ..Default::default()
        };
        let onion = composite_with_onion_skin(&doc, 0, &config);
        assert_eq!(active.as_raw(), onion.as_raw());
    }

    #[test]
    fn test_onion_skin_multi_frame() {
        let mut doc = AnimationDocument::new(10, 10, ColorRgba::TRANSPARENT);
        let l_id = LayerId(1);

        // Frame 0: dot at (0, 0)
        doc.get_cel_mut(0, l_id)
            .unwrap()
            .buffer
            .set_pixel(0, 0, BLACK);

        // Frame 1: dot at (5, 5)
        doc.add_frame(100, false);
        doc.get_cel_mut(1, l_id)
            .unwrap()
            .buffer
            .set_pixel(5, 5, BLACK);

        let config = OnionSkinConfig {
            enabled: true,
            prev_frames: 1,
            next_frames: 0,
            opacity: 50,
            tint_mode: OnionSkinTint::RedBlue,
            loop_bounds: false,
        };

        // When viewing Frame 1, pixel at (0, 0) from Frame 0 should be visible with red tint and alpha
        let onion = composite_with_onion_skin(&doc, 1, &config);
        let p_prev = onion.get_pixel(0, 0).unwrap();
        let p_curr = onion.get_pixel(5, 5).unwrap();

        assert!(p_prev.a > 0, "Previous frame pixel should have alpha");
        assert_eq!(p_curr, BLACK, "Current frame pixel should be solid black");
    }
}

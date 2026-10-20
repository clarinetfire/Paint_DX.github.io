//! Wacky Brushes, Rubber Stamps, and Pattern Fill inspired by Kid Pix.
//!
//! Pure Rust implementation (#![forbid(unsafe_code)]) with deterministic algorithms,
//! full RGBA support, and zero UI dependencies.

pub mod fill;
pub mod geometric;
pub mod particle;
pub mod stamps;
pub mod texture;

pub use fill::{
    pattern_flood_fill, pattern_flood_fill_with_options, FillPattern, PatternFillOptions,
};
pub use geometric::*;
pub use particle::*;
pub use stamps::{
    apply_stamp, apply_stamp_rotated, apply_stamp_with_options, StampOptions, StampShape,
    StampSpacing, StampTint,
};
pub use texture::*;

use crate::geometry::Point;
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Deterministic Linear Congruential Generator step for reproducible procedural brush effects.
#[inline]
pub fn next_lcg_rand(state: &mut u64) -> u32 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state >> 32) as u32
}

/// Helper to convert Hue (0..360), Saturation (0.0..1.0), Value (0.0..1.0) to ColorRgba.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> ColorRgba {
    let h = ((h % 360.0) + 360.0) % 360.0;
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    ColorRgba::rgb(
        ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// Helper to draw a filled disk on a buffer.
pub fn draw_filled_disk(buffer: &mut PixelBuffer, center: Point, radius: i32, color: ColorRgba) {
    if radius <= 0 {
        buffer.set_pixel_i32(center.x, center.y, color);
        return;
    }
    let r2 = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= r2 {
                buffer.set_pixel_i32(center.x + dx, center.y + dy, color);
            }
        }
    }
}

/// Helper to draw a blended disk on a buffer with alpha compositing.
pub fn blend_filled_disk(buffer: &mut PixelBuffer, center: Point, radius: i32, color: ColorRgba) {
    if radius <= 0 {
        buffer.blend_pixel_i32(center.x, center.y, color);
        return;
    }
    let r2 = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= r2 {
                buffer.blend_pixel_i32(center.x + dx, center.y + dy, color);
            }
        }
    }
}

/// Helper to stamp a cute pixel-art heart.
pub fn draw_pixel_heart(buffer: &mut PixelBuffer, center: Point, size: i32, color: ColorRgba) {
    if size <= 2 {
        let pattern = [
            (0, -2),
            (-1, -2),
            (1, -2),
            (-2, -1),
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (-2, 0),
            (-1, 0),
            (0, 0),
            (1, 0),
            (2, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            (0, 2),
        ];
        for (dx, dy) in pattern {
            buffer.set_pixel_i32(center.x + dx, center.y + dy, color);
        }
    } else {
        let scale = (size / 2).max(1);
        let pattern = [
            (-2, -2),
            (-1, -2),
            (1, -2),
            (2, -2),
            (-3, -1),
            (-2, -1),
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (3, -1),
            (-3, 0),
            (-2, 0),
            (-1, 0),
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (-2, 1),
            (-1, 1),
            (0, 1),
            (1, 1),
            (2, 1),
            (-1, 2),
            (0, 2),
            (1, 2),
            (0, 3),
        ];
        for (dx, dy) in pattern {
            for sy in 0..scale {
                for sx in 0..scale {
                    buffer.set_pixel_i32(
                        center.x + dx * scale + sx,
                        center.y + dy * scale + sy,
                        color,
                    );
                }
            }
        }
    }
}

// ============================================================================
// 1. WACKY BRUSHES (ALL 18 CLASSIC VARIANTS)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum WackyBrushShape {
    // 1. Procedural & Geometric
    #[default]
    PineTree,
    ConcentricCircles,
    Starburst,
    Swirl,
    LooperRibbon,
    ZigZag,

    // 2. Particle & Dynamic Ink
    LeakyPen,
    Bubbles,
    Splatter,
    NorthernLights,
    Confetti,
    ConnectTheDots,

    // 3. Texture & Special
    Rainbow,
    Caterpillar,
    PancakeStack,
    FuzzerFur,
    PatternRoll,
    Alphabet,
}

impl WackyBrushShape {
    pub const ALL: [WackyBrushShape; 18] = [
        // Sub-agent 1: Procedural & Geometric
        WackyBrushShape::PineTree,
        WackyBrushShape::ConcentricCircles,
        WackyBrushShape::Starburst,
        WackyBrushShape::Swirl,
        WackyBrushShape::LooperRibbon,
        WackyBrushShape::ZigZag,
        // Sub-agent 2: Particle & Dynamic Ink
        WackyBrushShape::LeakyPen,
        WackyBrushShape::Bubbles,
        WackyBrushShape::Splatter,
        WackyBrushShape::NorthernLights,
        WackyBrushShape::Confetti,
        WackyBrushShape::ConnectTheDots,
        // Sub-agent 3: Texture & Special
        WackyBrushShape::Rainbow,
        WackyBrushShape::Caterpillar,
        WackyBrushShape::PancakeStack,
        WackyBrushShape::FuzzerFur,
        WackyBrushShape::PatternRoll,
        WackyBrushShape::Alphabet,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            WackyBrushShape::PineTree => "Pine Tree / Sapin",
            WackyBrushShape::ConcentricCircles => "Concentric Circles / Cercles",
            WackyBrushShape::Starburst => "Starburst / Étoile Filante",
            WackyBrushShape::Swirl => "Swirl / Tourbillon",
            WackyBrushShape::LooperRibbon => "Looper Ribbon / Ruban Bouclé",
            WackyBrushShape::ZigZag => "Zig-Zag / Éclair Zigzag",
            WackyBrushShape::LeakyPen => "Leaky Pen / Stylo qui coule",
            WackyBrushShape::Bubbles => "Soap Bubbles / Bulles de savon",
            WackyBrushShape::Splatter => "Splatter / Éclaboussures",
            WackyBrushShape::NorthernLights => "Northern Lights / Aurore boréale",
            WackyBrushShape::Confetti => "Confetti / Fête Confettis",
            WackyBrushShape::ConnectTheDots => "Connect The Dots / Relier les points",
            WackyBrushShape::Rainbow => "Rainbow / Arc-en-ciel",
            WackyBrushShape::Caterpillar => "Caterpillar / Chenille",
            WackyBrushShape::PancakeStack => "Pancake Stack / Pâte à crêpe",
            WackyBrushShape::FuzzerFur => "Fuzzer Fur / Fourrure Poilue",
            WackyBrushShape::PatternRoll => "Pattern Roll / Rouleau Motif",
            WackyBrushShape::Alphabet => "Alphabet / Lettres ABC",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            WackyBrushShape::PineTree => "🌲",
            WackyBrushShape::ConcentricCircles => "🎯",
            WackyBrushShape::Starburst => "❇️",
            WackyBrushShape::Swirl => "🌀",
            WackyBrushShape::LooperRibbon => "🎀",
            WackyBrushShape::ZigZag => "⚡",
            WackyBrushShape::LeakyPen => "✒️",
            WackyBrushShape::Bubbles => "🫧",
            WackyBrushShape::Splatter => "💥",
            WackyBrushShape::NorthernLights => "🌌",
            WackyBrushShape::Confetti => "🎊",
            WackyBrushShape::ConnectTheDots => "🔢",
            WackyBrushShape::Rainbow => "🌈",
            WackyBrushShape::Caterpillar => "🐛",
            WackyBrushShape::PancakeStack => "🥞",
            WackyBrushShape::FuzzerFur => "🪶",
            WackyBrushShape::PatternRoll => "🏁",
            WackyBrushShape::Alphabet => "🔤",
        }
    }

    pub fn is_geometric(&self) -> bool {
        matches!(
            self,
            WackyBrushShape::PineTree
                | WackyBrushShape::ConcentricCircles
                | WackyBrushShape::Starburst
                | WackyBrushShape::Swirl
                | WackyBrushShape::LooperRibbon
                | WackyBrushShape::ZigZag
        )
    }

    pub fn is_particle(&self) -> bool {
        matches!(
            self,
            WackyBrushShape::LeakyPen
                | WackyBrushShape::Bubbles
                | WackyBrushShape::Splatter
                | WackyBrushShape::NorthernLights
                | WackyBrushShape::Confetti
                | WackyBrushShape::ConnectTheDots
        )
    }

    pub fn is_texture(&self) -> bool {
        matches!(
            self,
            WackyBrushShape::Rainbow
                | WackyBrushShape::Caterpillar
                | WackyBrushShape::PancakeStack
                | WackyBrushShape::FuzzerFur
                | WackyBrushShape::PatternRoll
                | WackyBrushShape::Alphabet
        )
    }
}

/// Spacing strategy along the interpolated stroke path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BrushSpacing {
    /// Draw at every Bresenham raster pixel (1 px step).
    Continuous,
    /// Default tool-dependent classic Kid Pix spacing (tied to brush size).
    #[default]
    Discrete,
    /// Custom Euclidean distance spacing in pixels between stamps.
    Custom(u32),
}

/// Angular orientation strategy for directional brushes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BrushAngleMode {
    /// Fixed angle in degrees (0° = standard orientation).
    Fixed(f32),
    /// Follow the tangent of the stroke vector: atan2(dy, dx).
    FollowStroke,
    /// Tangent angle with bounded pseudo-random angular jitter.
    FollowStrokeWithJitter { jitter_deg: f32 },
    /// Fixed angle plus random jitter.
    FixedWithJitter { base_deg: f32, jitter_deg: f32 },
    /// Continuous procedural rotation rate (degrees per step).
    SpinRate(f32),
}

impl Default for BrushAngleMode {
    fn default() -> Self {
        Self::Fixed(0.0)
    }
}

/// Seed control for deterministic frame reproduction vs dynamic sketching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SeedMode {
    /// Mutates and advances dynamic step counter across strokes.
    #[default]
    Dynamic,
    /// Explicit locked seed: strokes with the same seed yield bit-identical pixel outputs.
    Locked(u64),
}

/// Comprehensive configuration for wacky brush dynamics and pixel artist precision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WackyBrushOptions {
    /// Spacing between consecutive brush stamps along the path.
    pub spacing: BrushSpacing,
    /// Scatter/jitter radius perpendicular and radial to the path (0 = strictly on path).
    pub scatter_radius: u32,
    /// Relative particle density / burst multiplier (1.0 = standard classic density).
    pub particle_density: f32,
    /// Explicit particle burst count override (if `Some`, overrides the default density formula).
    pub burst_count_override: Option<u32>,
    /// Orientation and rotation behavior.
    pub angle_mode: BrushAngleMode,
    /// Deterministic seed locking for reproducible procedural drawings.
    pub seed_mode: SeedMode,
}

impl Default for WackyBrushOptions {
    fn default() -> Self {
        Self {
            spacing: BrushSpacing::Discrete,
            scatter_radius: 0,
            particle_density: 1.0,
            burst_count_override: None,
            angle_mode: BrushAngleMode::Fixed(0.0),
            seed_mode: SeedMode::Dynamic,
        }
    }
}

/// Internal state for wacky brush rendering across strokes and holding periods.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WackyBrushState {
    pub step: u64,
    pub alphabet_index: u8,
    pub hold_ticks: u32,
    pub last_point: Option<Point>,
    pub ribbon_trail: VecDeque<Point>,
    pub connect_dots_last: Option<Point>,
    pub distance_accumulator: f32,
    pub current_spin_angle: f32,
}

impl Default for WackyBrushState {
    fn default() -> Self {
        Self {
            step: 0,
            alphabet_index: 0,
            hold_ticks: 0,
            last_point: None,
            ribbon_trail: VecDeque::with_capacity(32),
            connect_dots_last: None,
            distance_accumulator: 0.0,
            current_spin_angle: 0.0,
        }
    }
}

impl WackyBrushState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_stroke(&mut self) {
        self.hold_ticks = 0;
        self.last_point = None;
        self.ribbon_trail.clear();
        self.connect_dots_last = None;
        self.distance_accumulator = 0.0;
    }
}

/// Applies a single wacky brush stamp/point at `center` with fine-grained options.
#[allow(clippy::too_many_arguments)]
pub fn apply_wacky_brush_point_with_options(
    buffer: &mut PixelBuffer,
    center: Point,
    shape: WackyBrushShape,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
    options: &WackyBrushOptions,
) {
    if let SeedMode::Locked(fixed_seed) = options.seed_mode {
        state.step = fixed_seed.wrapping_add(state.step.wrapping_mul(6364136223846793005));
    } else {
        state.step = state.step.wrapping_add(1);
    }

    // Apply scatter jitter if requested
    let actual_center = if options.scatter_radius > 0 {
        let mut rng = state.step;
        let r_rnd = (next_lcg_rand(&mut rng) % (options.scatter_radius + 1)) as i32;
        let angle_rnd = (next_lcg_rand(&mut rng) % 360) as f32 * std::f32::consts::PI / 180.0;
        let jx = (angle_rnd.cos() * r_rnd as f32).round() as i32;
        let jy = (angle_rnd.sin() * r_rnd as f32).round() as i32;
        Point::new(center.x + jx, center.y + jy)
    } else {
        center
    };

    if shape.is_geometric() {
        apply_geometric_brush_point(buffer, actual_center, shape, size, fg, bg, state);
    } else if shape.is_particle() {
        apply_particle_brush_point(buffer, actual_center, shape, size, fg, bg, state);
    } else {
        apply_texture_brush_point(buffer, actual_center, shape, size, fg, bg, state);
    }
    state.last_point = Some(actual_center);
}

/// Applies a single wacky brush stamp/point at `center`.
pub fn apply_wacky_brush_point(
    buffer: &mut PixelBuffer,
    center: Point,
    shape: WackyBrushShape,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    apply_wacky_brush_point_with_options(
        buffer,
        center,
        shape,
        size,
        fg,
        bg,
        state,
        &WackyBrushOptions::default(),
    );
}

/// Draws an interpolated stroke between `p0` and `p1` with fine-grained options.
#[allow(clippy::too_many_arguments)]
pub fn draw_wacky_brush_stroke_with_options(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    shape: WackyBrushShape,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
    options: &WackyBrushOptions,
) {
    match options.spacing {
        BrushSpacing::Discrete => {
            if shape.is_geometric() {
                draw_geometric_brush_stroke(buffer, p0, p1, shape, size, fg, bg, state);
            } else if shape.is_particle() {
                draw_particle_brush_stroke(buffer, p0, p1, shape, size, fg, bg, state);
            } else {
                draw_texture_brush_stroke(buffer, p0, p1, shape, size, fg, bg, state);
            }
        }
        BrushSpacing::Continuous => {
            let pts = crate::geometry::bresenham_line(p0, p1);
            if pts.is_empty() {
                apply_wacky_brush_point_with_options(
                    buffer, p0, shape, size, fg, bg, state, options,
                );
            } else {
                for pt in pts {
                    apply_wacky_brush_point_with_options(
                        buffer, pt, shape, size, fg, bg, state, options,
                    );
                }
            }
        }
        BrushSpacing::Custom(interval) => {
            let dx = (p1.x - p0.x) as f32;
            let dy = (p1.y - p0.y) as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            let step_size = (interval.max(1) as f32).max(1.0);

            if dist < 0.001 {
                apply_wacky_brush_point_with_options(
                    buffer, p0, shape, size, fg, bg, state, options,
                );
                return;
            }

            let mut t_dist = 0.0;
            while t_dist <= dist {
                let t = t_dist / dist;
                let pt = Point::new(
                    (p0.x as f32 + t * dx).round() as i32,
                    (p0.y as f32 + t * dy).round() as i32,
                );
                apply_wacky_brush_point_with_options(
                    buffer, pt, shape, size, fg, bg, state, options,
                );
                t_dist += step_size;
            }
        }
    }
}

/// Draws an interpolated stroke between `p0` and `p1` for the selected wacky brush.
#[allow(clippy::too_many_arguments)]
pub fn draw_wacky_brush_stroke(
    buffer: &mut PixelBuffer,
    p0: Point,
    p1: Point,
    shape: WackyBrushShape,
    size: u32,
    fg: ColorRgba,
    bg: ColorRgba,
    state: &mut WackyBrushState,
) {
    draw_wacky_brush_stroke_with_options(
        buffer,
        p0,
        p1,
        shape,
        size,
        fg,
        bg,
        state,
        &WackyBrushOptions::default(),
    );
}

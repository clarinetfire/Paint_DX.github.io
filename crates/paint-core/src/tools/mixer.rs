//! Kid Pix inspired Electric Mixer ("Mixeur électrique") tools.
//!
//! Provides 28 procedural image transformation and distortion effects divided into:
//! - Tray 1: 14 Classic image transformations, optical distortions, and filters.
//! - Tray 2: 14 Wacky, celebratory, textured, and graphic pattern mixers.
//!
//! Pure Rust implementation (`#![forbid(unsafe_code)]`) with deterministic pseudo-randomness,
//! zero UI dependencies, and full RGBA non-destructive rendering.

use crate::geometry::Rect;
use crate::image::PixelBuffer;
use crate::palette::{ColorRgba, WHITE};
use serde::{Deserialize, Serialize};

/// Deterministic Linear Congruential Generator for reproducible procedural mixer effects.
#[inline]
pub fn next_mixer_rand(state: &mut u64) -> u32 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state >> 32) as u32
}

/// Helper to generate a float in [0.0, 1.0) from LCG state.
#[inline]
fn next_rand_f32(state: &mut u64) -> f32 {
    (next_mixer_rand(state) as f32) / (u32::MAX as f32)
}

/// Helper to compute pixel perceived luminance in [0.0, 255.0].
#[inline]
pub fn pixel_luminance(c: ColorRgba) -> f32 {
    0.299 * c.r as f32 + 0.587 * c.g as f32 + 0.114 * c.b as f32
}

// ============================================================================
// MIXER TRAYS & EFFECTS ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MixerTray {
    #[default]
    Tray1,
    Tray2,
}

impl MixerTray {
    pub const ALL: [MixerTray; 2] = [MixerTray::Tray1, MixerTray::Tray2];

    pub fn display_name(&self) -> &'static str {
        match self {
            MixerTray::Tray1 => "Tray 1 (Classic Mixers)",
            MixerTray::Tray2 => "Tray 2 (Wacky & Party)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MixerEffect {
    // Tray 1: 14 Classic Transformations & Filters
    FlipSideToSide,
    FlipTopToBottom,
    Spin,
    WashOut,
    WackyColors,
    Wave,
    PictureShrinker,
    ColoringBook,
    Newspaper,
    Soften,
    PictureFlower,
    Quilt,
    WarpAndWoof,
    Spread,

    // Tray 2: 14 Wacky, Stylized & Party Mixers
    ItchAndScratch,
    MovingLights,
    Pompom,
    Pyramid,
    Pulsar,
    Confetti,
    DancingDisks,
    TwinkleTwinkle,
    Subway,
    DancingTrees,
    ColorCoins,
    DogBones,
    Stripes,
    NeonTube,
}

impl MixerEffect {
    pub const TRAY_1: [MixerEffect; 14] = [
        MixerEffect::FlipSideToSide,
        MixerEffect::FlipTopToBottom,
        MixerEffect::Spin,
        MixerEffect::WashOut,
        MixerEffect::WackyColors,
        MixerEffect::Wave,
        MixerEffect::PictureShrinker,
        MixerEffect::ColoringBook,
        MixerEffect::Newspaper,
        MixerEffect::Soften,
        MixerEffect::PictureFlower,
        MixerEffect::Quilt,
        MixerEffect::WarpAndWoof,
        MixerEffect::Spread,
    ];

    pub const TRAY_2: [MixerEffect; 14] = [
        MixerEffect::ItchAndScratch,
        MixerEffect::MovingLights,
        MixerEffect::Pompom,
        MixerEffect::Pyramid,
        MixerEffect::Pulsar,
        MixerEffect::Confetti,
        MixerEffect::DancingDisks,
        MixerEffect::TwinkleTwinkle,
        MixerEffect::Subway,
        MixerEffect::DancingTrees,
        MixerEffect::ColorCoins,
        MixerEffect::DogBones,
        MixerEffect::Stripes,
        MixerEffect::NeonTube,
    ];

    pub const ALL: [MixerEffect; 28] = [
        MixerEffect::FlipSideToSide,
        MixerEffect::FlipTopToBottom,
        MixerEffect::Spin,
        MixerEffect::WashOut,
        MixerEffect::WackyColors,
        MixerEffect::Wave,
        MixerEffect::PictureShrinker,
        MixerEffect::ColoringBook,
        MixerEffect::Newspaper,
        MixerEffect::Soften,
        MixerEffect::PictureFlower,
        MixerEffect::Quilt,
        MixerEffect::WarpAndWoof,
        MixerEffect::Spread,
        MixerEffect::ItchAndScratch,
        MixerEffect::MovingLights,
        MixerEffect::Pompom,
        MixerEffect::Pyramid,
        MixerEffect::Pulsar,
        MixerEffect::Confetti,
        MixerEffect::DancingDisks,
        MixerEffect::TwinkleTwinkle,
        MixerEffect::Subway,
        MixerEffect::DancingTrees,
        MixerEffect::ColorCoins,
        MixerEffect::DogBones,
        MixerEffect::Stripes,
        MixerEffect::NeonTube,
    ];

    pub fn tray(&self) -> MixerTray {
        match self {
            MixerEffect::FlipSideToSide
            | MixerEffect::FlipTopToBottom
            | MixerEffect::Spin
            | MixerEffect::WashOut
            | MixerEffect::WackyColors
            | MixerEffect::Wave
            | MixerEffect::PictureShrinker
            | MixerEffect::ColoringBook
            | MixerEffect::Newspaper
            | MixerEffect::Soften
            | MixerEffect::PictureFlower
            | MixerEffect::Quilt
            | MixerEffect::WarpAndWoof
            | MixerEffect::Spread => MixerTray::Tray1,

            MixerEffect::ItchAndScratch
            | MixerEffect::MovingLights
            | MixerEffect::Pompom
            | MixerEffect::Pyramid
            | MixerEffect::Pulsar
            | MixerEffect::Confetti
            | MixerEffect::DancingDisks
            | MixerEffect::TwinkleTwinkle
            | MixerEffect::Subway
            | MixerEffect::DancingTrees
            | MixerEffect::ColorCoins
            | MixerEffect::DogBones
            | MixerEffect::Stripes
            | MixerEffect::NeonTube => MixerTray::Tray2,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            MixerEffect::FlipSideToSide => "Flip Side to Side",
            MixerEffect::FlipTopToBottom => "Flip Top to Bottom",
            MixerEffect::Spin => "Spin",
            MixerEffect::WashOut => "Wash Out",
            MixerEffect::WackyColors => "Wacky Colors",
            MixerEffect::Wave => "Wave",
            MixerEffect::PictureShrinker => "Picture Shrinker",
            MixerEffect::ColoringBook => "Coloring Book",
            MixerEffect::Newspaper => "Newspaper",
            MixerEffect::Soften => "Soften",
            MixerEffect::PictureFlower => "Picture Flower",
            MixerEffect::Quilt => "Quilt",
            MixerEffect::WarpAndWoof => "Warp and Woof",
            MixerEffect::Spread => "Spread",

            MixerEffect::ItchAndScratch => "Itch and Scratch",
            MixerEffect::MovingLights => "Moving Lights",
            MixerEffect::Pompom => "Pompom",
            MixerEffect::Pyramid => "Pyramid",
            MixerEffect::Pulsar => "Pulsar",
            MixerEffect::Confetti => "Confetti",
            MixerEffect::DancingDisks => "Dancing Disks",
            MixerEffect::TwinkleTwinkle => "Twinkle, Twinkle",
            MixerEffect::Subway => "Subway",
            MixerEffect::DancingTrees => "Dancing Trees",
            MixerEffect::ColorCoins => "Color Coins",
            MixerEffect::DogBones => "Dog Bones",
            MixerEffect::Stripes => "Stripes",
            MixerEffect::NeonTube => "Neon Tube",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            MixerEffect::FlipSideToSide => "Flip H",
            MixerEffect::FlipTopToBottom => "Flip V",
            MixerEffect::Spin => "Spin",
            MixerEffect::WashOut => "Wash",
            MixerEffect::WackyColors => "Wacky",
            MixerEffect::Wave => "Wave",
            MixerEffect::PictureShrinker => "Shrink",
            MixerEffect::ColoringBook => "ColorBook",
            MixerEffect::Newspaper => "News",
            MixerEffect::Soften => "Soften",
            MixerEffect::PictureFlower => "Flower",
            MixerEffect::Quilt => "Quilt",
            MixerEffect::WarpAndWoof => "Warp",
            MixerEffect::Spread => "Spread",

            MixerEffect::ItchAndScratch => "Scratch",
            MixerEffect::MovingLights => "Lights",
            MixerEffect::Pompom => "Pompom",
            MixerEffect::Pyramid => "Pyramid",
            MixerEffect::Pulsar => "Pulsar",
            MixerEffect::Confetti => "Confetti",
            MixerEffect::DancingDisks => "Disks",
            MixerEffect::TwinkleTwinkle => "Twinkle",
            MixerEffect::Subway => "Subway",
            MixerEffect::DancingTrees => "Trees",
            MixerEffect::ColorCoins => "Coins",
            MixerEffect::DogBones => "Bones",
            MixerEffect::Stripes => "Stripes",
            MixerEffect::NeonTube => "Neon",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            MixerEffect::FlipSideToSide => "↔",
            MixerEffect::FlipTopToBottom => "↕",
            MixerEffect::Spin => "🔄",
            MixerEffect::WashOut => "🧴",
            MixerEffect::WackyColors => "🌈",
            MixerEffect::Wave => "🌊",
            MixerEffect::PictureShrinker => "🗜",
            MixerEffect::ColoringBook => "📖",
            MixerEffect::Newspaper => "📰",
            MixerEffect::Soften => "☁",
            MixerEffect::PictureFlower => "🌸",
            MixerEffect::Quilt => "🪡",
            MixerEffect::WarpAndWoof => "🧶",
            MixerEffect::Spread => "✨",

            MixerEffect::ItchAndScratch => "⚡",
            MixerEffect::MovingLights => "💫",
            MixerEffect::Pompom => "🧶",
            MixerEffect::Pyramid => "🔺",
            MixerEffect::Pulsar => "🌀",
            MixerEffect::Confetti => "🎉",
            MixerEffect::DancingDisks => "🔘",
            MixerEffect::TwinkleTwinkle => "⭐",
            MixerEffect::Subway => "🧱",
            MixerEffect::DancingTrees => "🌲",
            MixerEffect::ColorCoins => "🪙",
            MixerEffect::DogBones => "🦴",
            MixerEffect::Stripes => "💈",
            MixerEffect::NeonTube => "💡",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            MixerEffect::FlipSideToSide => "Flip image horizontally (left to right mirror)",
            MixerEffect::FlipTopToBottom => "Flip image vertically (top to bottom mirror)",
            MixerEffect::Spin => "Rotate image 90 degrees clockwise",
            MixerEffect::WashOut => "Wash out colors into high-key pastel watercolor",
            MixerEffect::WackyColors => "Psychedelic color cycle and vibrant inversion",
            MixerEffect::Wave => "Undulating sine-wave horizontal ripple distortion",
            MixerEffect::PictureShrinker => "Shrink and tile into 4 mini quadrants (2x2 grid)",
            MixerEffect::ColoringBook => {
                "Convert artwork into black & white coloring book outlines"
            }
            MixerEffect::Newspaper => "Convert image into coarse monochrome newsprint halftone",
            MixerEffect::Soften => "Gaussian blur to soften and smooth edges",
            MixerEffect::PictureFlower => "4-way radial kaleidoscopic floral bloom",
            MixerEffect::Quilt => "Patchwork quilt with alternating stitched flipped patches",
            MixerEffect::WarpAndWoof => "Simulate interlaced woven textile threads on a loom",
            MixerEffect::Spread => "Disperse pixels like frosted glass / wet paint splatter",

            MixerEffect::ItchAndScratch => {
                "Scatter sharp random etching scratches across the image"
            }
            MixerEffect::MovingLights => "Project glowing light beams and flares from highlights",
            MixerEffect::Pompom => "Scatter fluffy yarn pompom clusters across the canvas",
            MixerEffect::Pyramid => "3D stepped concentric pyramid extrusion with relief shading",
            MixerEffect::Pulsar => "Concentric cosmic shockwave ripples from the center",
            MixerEffect::Confetti => "Explosion of colorful celebration confetti fragments",
            MixerEffect::DancingDisks => "Pop-art mosaic of circular color-averaged disks",
            MixerEffect::TwinkleTwinkle => {
                "Scatter twinkling diamond starbursts across bright areas"
            }
            MixerEffect::Subway => "Glossy subway tile mosaic with bevels and dark grout",
            MixerEffect::DancingTrees => "Sprout whimsical Kid Pix pine trees across the landscape",
            MixerEffect::ColorCoins => {
                "Scatter shiny embossed metallic coins (gold, silver, bronze)"
            }
            MixerEffect::DogBones => "Scatter whimsical cartoon dog bones tumbled across artwork",
            MixerEffect::Stripes => "Bold alternating pop-art color stripes",
            MixerEffect::NeonTube => "Glowing electric neon sign outlines on dark midnight canvas",
        }
    }
}

/// Execution scope for Electric Mixer effects.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MixerScope {
    /// Apply to the entire pixel buffer.
    #[default]
    EntireCanvas,
    /// Restrict transformation strictly inside an axis-aligned bounding box.
    BoundingBox(Rect),
    /// Arbitrary free-form selection defined by a bounding rect and boolean pixel mask.
    Masked {
        bounds: Rect,
        /// Row-major mask of size bounds.width() * bounds.height()
        mask: Vec<bool>,
    },
}

/// Wave effect customization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WaveParams {
    pub amplitude: f32,
    pub wavelength: f32,
    pub vertical: bool,
}

impl Default for WaveParams {
    fn default() -> Self {
        Self {
            amplitude: 7.0,
            wavelength: 28.0,
            vertical: false,
        }
    }
}

/// Gaussian/box soften blur customization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftenParams {
    pub radius: u32,
    pub iterations: u32,
}

impl Default for SoftenParams {
    fn default() -> Self {
        Self {
            radius: 1,
            iterations: 1,
        }
    }
}

/// Halftone newspaper dither customization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NewspaperParams {
    pub matrix_size: u8,
    pub contrast: f32,
}

impl Default for NewspaperParams {
    fn default() -> Self {
        Self {
            matrix_size: 4,
            contrast: 1.0,
        }
    }
}

/// Edge detection coloring book customization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColoringBookParams {
    pub threshold: f32,
    pub inverted: bool,
}

impl Default for ColoringBookParams {
    fn default() -> Self {
        Self {
            threshold: 35.0,
            inverted: false,
        }
    }
}

/// Tunable parameters for key mixer effects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixerParams {
    pub wave: WaveParams,
    pub soften: SoftenParams,
    pub newspaper: NewspaperParams,
    pub coloring_book: ColoringBookParams,
    pub global_intensity: f32,
}

impl Default for MixerParams {
    fn default() -> Self {
        Self {
            wave: WaveParams::default(),
            soften: SoftenParams::default(),
            newspaper: NewspaperParams::default(),
            coloring_book: ColoringBookParams::default(),
            global_intensity: 1.0,
        }
    }
}

/// Master options for Electric Mixer operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixerOptions {
    pub seed: u64,
    pub scope: MixerScope,
    pub params: MixerParams,
}

impl MixerOptions {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            seed,
            scope: MixerScope::EntireCanvas,
            params: MixerParams::default(),
        }
    }
}

// ============================================================================
// MAIN APPLICATION DISPATCHER
// ============================================================================

/// Applies an Electric Mixer effect with fine-grained options (scoping, bounding box, selection mask).
pub fn apply_mixer_effect_with_options(
    buffer: &PixelBuffer,
    effect: MixerEffect,
    options: &MixerOptions,
) -> PixelBuffer {
    match &options.scope {
        MixerScope::EntireCanvas => apply_mixer_effect(buffer, effect, options.seed),
        MixerScope::BoundingBox(rect) => {
            let buf_w = buffer.width() as i32;
            let buf_h = buffer.height() as i32;

            let min_x = rect.min.x.clamp(0, buf_w);
            let min_y = rect.min.y.clamp(0, buf_h);
            let max_x = (rect.max.x + 1).clamp(0, buf_w);
            let max_y = (rect.max.y + 1).clamp(0, buf_h);

            let sub_w = (max_x - min_x).max(0) as u32;
            let sub_h = (max_y - min_y).max(0) as u32;

            if sub_w == 0 || sub_h == 0 {
                return buffer.clone();
            }

            let sub_buffer = buffer.crop(min_x as u32, min_y as u32, sub_w, sub_h);
            let processed = apply_mixer_effect(&sub_buffer, effect, options.seed);

            let mut result = buffer.clone();
            for dy in 0..sub_h {
                for dx in 0..sub_w {
                    if let Some(p) = processed.get_pixel(dx, dy) {
                        result.set_pixel_i32(min_x + dx as i32, min_y + dy as i32, p);
                    }
                }
            }
            result
        }
        MixerScope::Masked { bounds, mask } => {
            let buf_w = buffer.width() as i32;
            let buf_h = buffer.height() as i32;

            let min_x = bounds.min.x.clamp(0, buf_w);
            let min_y = bounds.min.y.clamp(0, buf_h);
            let max_x = (bounds.max.x + 1).clamp(0, buf_w);
            let max_y = (bounds.max.y + 1).clamp(0, buf_h);

            let sub_w = (max_x - min_x).max(0) as u32;
            let sub_h = (max_y - min_y).max(0) as u32;

            if sub_w == 0 || sub_h == 0 {
                return buffer.clone();
            }

            let sub_buffer = buffer.crop(min_x as u32, min_y as u32, sub_w, sub_h);
            let processed = apply_mixer_effect(&sub_buffer, effect, options.seed);

            let mut result = buffer.clone();
            let b_w = bounds.width();
            for dy in 0..sub_h {
                for dx in 0..sub_w {
                    let mask_idx = (dy as usize) * (b_w as usize) + (dx as usize);
                    if mask.get(mask_idx).copied().unwrap_or(false) {
                        if let Some(p) = processed.get_pixel(dx, dy) {
                            result.set_pixel_i32(min_x + dx as i32, min_y + dy as i32, p);
                        }
                    }
                }
            }
            result
        }
    }
}

/// Applies an Electric Mixer effect to a pixel buffer, returning the resulting buffer.
pub fn apply_mixer_effect(buffer: &PixelBuffer, effect: MixerEffect, seed: u64) -> PixelBuffer {
    let mut rng = seed;
    match effect {
        // Tray 1
        MixerEffect::FlipSideToSide => effect_flip_side_to_side(buffer),
        MixerEffect::FlipTopToBottom => effect_flip_top_to_bottom(buffer),
        MixerEffect::Spin => effect_spin(buffer),
        MixerEffect::WashOut => effect_wash_out(buffer),
        MixerEffect::WackyColors => effect_wacky_colors(buffer),
        MixerEffect::Wave => effect_wave(buffer),
        MixerEffect::PictureShrinker => effect_picture_shrinker(buffer),
        MixerEffect::ColoringBook => effect_coloring_book(buffer),
        MixerEffect::Newspaper => effect_newspaper(buffer),
        MixerEffect::Soften => effect_soften(buffer),
        MixerEffect::PictureFlower => effect_picture_flower(buffer),
        MixerEffect::Quilt => effect_quilt(buffer),
        MixerEffect::WarpAndWoof => effect_warp_and_woof(buffer),
        MixerEffect::Spread => effect_spread(buffer, &mut rng),

        // Tray 2
        MixerEffect::ItchAndScratch => effect_itch_and_scratch(buffer, &mut rng),
        MixerEffect::MovingLights => effect_moving_lights(buffer),
        MixerEffect::Pompom => effect_pompom(buffer, &mut rng),
        MixerEffect::Pyramid => effect_pyramid(buffer),
        MixerEffect::Pulsar => effect_pulsar(buffer),
        MixerEffect::Confetti => effect_confetti(buffer, &mut rng),
        MixerEffect::DancingDisks => effect_dancing_disks(buffer),
        MixerEffect::TwinkleTwinkle => effect_twinkle_twinkle(buffer, &mut rng),
        MixerEffect::Subway => effect_subway(buffer),
        MixerEffect::DancingTrees => effect_dancing_trees(buffer, &mut rng),
        MixerEffect::ColorCoins => effect_color_coins(buffer, &mut rng),
        MixerEffect::DogBones => effect_dog_bones(buffer, &mut rng),
        MixerEffect::Stripes => effect_stripes(buffer),
        MixerEffect::NeonTube => effect_neon_tube(buffer),
    }
}

// ============================================================================
// TRAY 1 IMPLEMENTATIONS
// ============================================================================

/// 1. Flip Side to Side (Horizontal Mirror)
pub fn effect_flip_side_to_side(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(w - 1 - x, y, p);
            }
        }
    }
    dst
}

/// 2. Flip Top to Bottom (Vertical Mirror)
pub fn effect_flip_top_to_bottom(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(x, h - 1 - y, p);
            }
        }
    }
    dst
}

/// 3. Spin (90° Clockwise Rotation)
pub fn effect_spin(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(h, w, WHITE);
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(h - 1 - y, x, p);
            }
        }
    }
    dst
}

/// 4. Wash Out (Pastel watercolor lighten & desaturate)
pub fn effect_wash_out(buffer: &PixelBuffer) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = dst.get_pixel(x, y) {
                // Lighten by 60% towards pure white
                let r = p.r as u32 + ((255 - p.r as u32) * 3) / 5;
                let g = p.g as u32 + ((255 - p.g as u32) * 3) / 5;
                let b = p.b as u32 + ((255 - p.b as u32) * 3) / 5;
                dst.set_pixel(
                    x,
                    y,
                    ColorRgba::rgba(r.min(255) as u8, g.min(255) as u8, b.min(255) as u8, p.a),
                );
            }
        }
    }
    dst
}

/// 5. Wacky Colors (Psychedelic channel rotation and inversion)
pub fn effect_wacky_colors(buffer: &PixelBuffer) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = dst.get_pixel(x, y) {
                // (r, g, b) -> (g, 255 - b, r)
                dst.set_pixel(x, y, ColorRgba::rgba(p.g, 255 - p.b, p.r, p.a));
            }
        }
    }
    dst
}

/// 6. Wave (Sine-wave horizontal ripple)
pub fn effect_wave(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    if w == 0 || h == 0 {
        return buffer.clone();
    }
    let mut dst = PixelBuffer::new(w, h, WHITE);
    let wavelength = 28.0_f32;
    let amplitude = 7.0_f32;

    for y in 0..h {
        let offset =
            (amplitude * (y as f32 * 2.0 * std::f32::consts::PI / wavelength).sin()).round() as i32;
        for x in 0..w {
            let src_x = (x as i32 - offset).rem_euclid(w as i32) as u32;
            if let Some(p) = buffer.get_pixel(src_x, y) {
                dst.set_pixel(x, y, p);
            }
        }
    }
    dst
}

/// 7. Picture Shrinker (2x2 mini-quadrant tiling)
pub fn effect_picture_shrinker(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    if w == 0 || h == 0 {
        return buffer.clone();
    }
    let mut dst = PixelBuffer::new(w, h, WHITE);
    let half_w = (w / 2).max(1);
    let half_h = (h / 2).max(1);

    for qy in 0..2 {
        let base_y = qy * half_h;
        for qx in 0..2 {
            let base_x = qx * half_w;
            for dy in 0..half_h {
                let sy = ((dy as u64 * h as u64) / half_h as u64).min((h - 1) as u64) as u32;
                for dx in 0..half_w {
                    let sx = ((dx as u64 * w as u64) / half_w as u64).min((w - 1) as u64) as u32;
                    let target_x = base_x + dx;
                    let target_y = base_y + dy;
                    if target_x < w && target_y < h {
                        if let Some(p) = buffer.get_pixel(sx, sy) {
                            dst.set_pixel(target_x, target_y, p);
                        }
                    }
                }
            }
        }
    }
    dst
}

/// 8. Coloring Book (Edge detection to crisp black outlines on white canvas)
pub fn effect_coloring_book(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);
    let threshold = 35.0_f32;

    for y in 0..h {
        for x in 0..w {
            let c = match buffer.get_pixel(x, y) {
                Some(p) => p,
                None => continue,
            };
            let lum = pixel_luminance(c);

            let lum_r = if x + 1 < w {
                pixel_luminance(buffer.get_pixel(x + 1, y).unwrap_or(c))
            } else {
                lum
            };
            let lum_d = if y + 1 < h {
                pixel_luminance(buffer.get_pixel(x, y + 1).unwrap_or(c))
            } else {
                lum
            };

            let diff = (lum - lum_r).abs().max((lum - lum_d).abs());
            if diff >= threshold {
                dst.set_pixel(x, y, ColorRgba::rgba(0, 0, 0, c.a));
            } else {
                dst.set_pixel(x, y, ColorRgba::rgba(255, 255, 255, c.a));
            }
        }
    }
    dst
}

/// 9. Newspaper (Coarse monochrome newsprint halftone with 4x4 Bayer matrix)
pub fn effect_newspaper(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);

    // 4x4 Bayer Dither Matrix scaled to [0, 255]
    #[rustfmt::skip]
    let bayer: [u8; 16] = [
         15, 135,  45, 165,
        195,  75, 225, 105,
         60, 180,  30, 150,
        240, 120, 210,  90,
    ];

    let ink = ColorRgba::rgb(24, 24, 28);
    let paper = ColorRgba::rgb(246, 244, 238);

    for y in 0..h {
        let bayer_row = (y % 4) * 4;
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                let lum = pixel_luminance(p) as u8;
                let thresh = bayer[(bayer_row + (x % 4)) as usize];
                let chosen = if lum > thresh { paper } else { ink };
                dst.set_pixel(x, y, ColorRgba::rgba(chosen.r, chosen.g, chosen.b, p.a));
            }
        }
    }
    dst
}

/// 10. Soften (3x3 Gaussian-weighted smoothing)
pub fn effect_soften(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);

    for y in 0..h {
        for x in 0..w {
            let mut sum_r = 0u32;
            let mut sum_g = 0u32;
            let mut sum_b = 0u32;
            let mut sum_a = 0u32;
            let mut weight_sum = 0u32;

            for dy in -1i32..=1 {
                let sy = y as i32 + dy;
                if sy < 0 || sy >= h as i32 {
                    continue;
                }
                for dx in -1i32..=1 {
                    let sx = x as i32 + dx;
                    if sx < 0 || sx >= w as i32 {
                        continue;
                    }
                    let weight = match (dx.abs(), dy.abs()) {
                        (0, 0) => 4,
                        (1, 0) | (0, 1) => 2,
                        _ => 1,
                    };
                    if let Some(p) = buffer.get_pixel(sx as u32, sy as u32) {
                        sum_r += p.r as u32 * weight;
                        sum_g += p.g as u32 * weight;
                        sum_b += p.b as u32 * weight;
                        sum_a += p.a as u32 * weight;
                        weight_sum += weight;
                    }
                }
            }

            let weight_sum = weight_sum.max(1);
            dst.set_pixel(
                x,
                y,
                ColorRgba::rgba(
                    (sum_r / weight_sum) as u8,
                    (sum_g / weight_sum) as u8,
                    (sum_b / weight_sum) as u8,
                    (sum_a / weight_sum) as u8,
                ),
            );
        }
    }
    dst
}

/// 11. Picture Flower (4-fold kaleidoscopic radial reflection from center)
pub fn effect_picture_flower(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);
    let cx = w / 2;
    let cy = h / 2;

    for y in 0..h {
        let sy = if y < cy {
            y
        } else {
            (h - 1).saturating_sub(y).min(cy.saturating_sub(1))
        };
        for x in 0..w {
            let sx = if x < cx {
                x
            } else {
                (w - 1).saturating_sub(x).min(cx.saturating_sub(1))
            };
            if let Some(p) = buffer.get_pixel(sx, sy) {
                dst.set_pixel(x, y, p);
            }
        }
    }
    dst
}

/// 12. Quilt (Patchwork grid of alternating flipped patches with stitched borders)
pub fn effect_quilt(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = buffer.clone();
    let patch_size = 28u32;

    let num_x = w.div_ceil(patch_size);
    let num_y = h.div_ceil(patch_size);

    for py in 0..num_y {
        for px in 0..num_x {
            let is_alt = (px + py) % 2 == 1;
            let start_x = px * patch_size;
            let start_y = py * patch_size;
            let end_x = (start_x + patch_size).min(w);
            let end_y = (start_y + patch_size).min(h);
            let cur_w = end_x.saturating_sub(start_x);

            if is_alt {
                // Flip patch horizontally and invert
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let offset_x = x - start_x;
                        let src_x = start_x + (cur_w - 1 - offset_x);
                        if let Some(p) = buffer.get_pixel(src_x, y) {
                            dst.set_pixel(
                                x,
                                y,
                                ColorRgba::rgba(255 - p.r, 255 - p.g, 255 - p.b, p.a),
                            );
                        }
                    }
                }
            }

            // Draw dashed quilt stitch border
            let stitch_color = ColorRgba::rgb(220, 220, 240);
            for x in start_x..end_x {
                if (x / 2) % 2 == 0 {
                    dst.set_pixel(x, start_y, stitch_color);
                    if end_y > 0 {
                        dst.set_pixel(x, end_y - 1, stitch_color);
                    }
                }
            }
            for y in start_y..end_y {
                if (y / 2) % 2 == 0 {
                    dst.set_pixel(start_x, y, stitch_color);
                    if end_x > 0 {
                        dst.set_pixel(end_x - 1, y, stitch_color);
                    }
                }
            }
        }
    }
    dst
}

/// 13. Warp and Woof (Interlaced woven textile fabric structure)
pub fn effect_warp_and_woof(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);

    for y in 0..h {
        for x in 0..w {
            let warp_thread = (x / 3) % 2 == 0;
            let woof_thread = (y / 3) % 2 == 0;
            let is_over = warp_thread ^ woof_thread;

            // Subtle woven shift
            let (sx, sy) = if is_over {
                (x, if (x % 3) == 0 { (y + 1).min(h - 1) } else { y })
            } else {
                (if (y % 3) == 0 { (x + 1).min(w - 1) } else { x }, y)
            };

            if let Some(p) = buffer.get_pixel(sx, sy) {
                // Modulate lightness for weave highlight and shadow
                let light_mod = if is_over { 22i16 } else { -22i16 };
                let r = (p.r as i16 + light_mod).clamp(0, 255) as u8;
                let g = (p.g as i16 + light_mod).clamp(0, 255) as u8;
                let b = (p.b as i16 + light_mod).clamp(0, 255) as u8;
                dst.set_pixel(x, y, ColorRgba::rgba(r, g, b, p.a));
            }
        }
    }
    dst
}

/// 14. Spread (Frosted-glass random pixel jitter dispersion)
pub fn effect_spread(buffer: &PixelBuffer, rng: &mut u64) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    if w == 0 || h == 0 {
        return buffer.clone();
    }
    let mut dst = buffer.clone();

    for y in 0..h {
        for x in 0..w {
            let r_val = next_mixer_rand(rng);
            let dx = (r_val % 7) as i32 - 3; // -3..=3
            let dy = ((r_val >> 4) % 7) as i32 - 3;

            let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as u32;
            let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as u32;

            if let Some(p) = buffer.get_pixel(sx, sy) {
                dst.set_pixel(x, y, p);
            }
        }
    }
    dst
}

// ============================================================================
// TRAY 2 IMPLEMENTATIONS
// ============================================================================

/// 15. Itch and Scratch (Sharp random etching lines across canvas)
pub fn effect_itch_and_scratch(buffer: &PixelBuffer, rng: &mut u64) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    if w == 0 || h == 0 {
        return dst;
    }

    let scratch_count = (w * h / 1200).clamp(30, 160);
    for _ in 0..scratch_count {
        let x0 = (next_mixer_rand(rng) % w) as i32;
        let y0 = (next_mixer_rand(rng) % h) as i32;
        let len = (next_mixer_rand(rng) % 35 + 12) as f32;
        let angle = next_rand_f32(rng) * 2.0 * std::f32::consts::PI;

        let x1 = (x0 as f32 + len * angle.cos()).round() as i32;
        let y1 = (y0 as f32 + len * angle.sin()).round() as i32;

        let scratch_kind = next_mixer_rand(rng) % 3;
        let color = match scratch_kind {
            0 => ColorRgba::rgb(255, 255, 255),
            1 => ColorRgba::rgb(15, 15, 20),
            _ => ColorRgba::rgb(230, 230, 100),
        };

        // Draw scratch line with Bresenham
        draw_simple_line(&mut dst, x0, y0, x1, y1, color);
    }
    dst
}

/// 16. Moving Lights (Glowing light flares projecting from highlights)
pub fn effect_moving_lights(buffer: &PixelBuffer) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();

    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                if pixel_luminance(p) > 170.0 {
                    // Shoot glowing light flare streak horizontally
                    let flare_len = 14u32;
                    for d in 1..=flare_len {
                        let target_x = x + d;
                        if target_x >= w {
                            break;
                        }
                        let falloff = (flare_len - d) as f32 / flare_len as f32;
                        if let Some(curr) = dst.get_pixel(target_x, y) {
                            let add_r = (p.r as f32 * falloff * 0.7) as u16;
                            let add_g = (p.g as f32 * falloff * 0.7) as u16;
                            let add_b = (p.b as f32 * falloff * 0.7) as u16;
                            dst.set_pixel(
                                target_x,
                                y,
                                ColorRgba::rgba(
                                    (curr.r as u16 + add_r).min(255) as u8,
                                    (curr.g as u16 + add_g).min(255) as u8,
                                    (curr.b as u16 + add_b).min(255) as u8,
                                    curr.a,
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
    dst
}

/// 17. Pompom (Fluffy circular pom-pom yarn clusters)
pub fn effect_pompom(buffer: &PixelBuffer, rng: &mut u64) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    if w == 0 || h == 0 {
        return dst;
    }

    let pompom_count = (w * h / 2800).clamp(12, 45);
    for _ in 0..pompom_count {
        let cx = (next_mixer_rand(rng) % w) as i32;
        let cy = (next_mixer_rand(rng) % h) as i32;
        let radius = (next_mixer_rand(rng) % 9 + 8) as i32;

        let base_color = buffer
            .get_pixel(cx as u32, cy as u32)
            .unwrap_or(ColorRgba::rgb(255, 120, 180));

        // Draw dozens of radial yarn filaments
        let filaments = 70;
        for _ in 0..filaments {
            let angle = next_rand_f32(rng) * 2.0 * std::f32::consts::PI;
            let r = next_rand_f32(rng).sqrt() * radius as f32;
            let px = (cx as f32 + r * angle.cos()).round() as i32;
            let py = (cy as f32 + r * angle.sin()).round() as i32;

            if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                let jitter = (next_mixer_rand(rng) % 31) as i16 - 15;
                let yarn_col = ColorRgba::rgba(
                    (base_color.r as i16 + jitter).clamp(0, 255) as u8,
                    (base_color.g as i16 + jitter).clamp(0, 255) as u8,
                    (base_color.b as i16 + jitter).clamp(0, 255) as u8,
                    base_color.a,
                );
                dst.set_pixel(px as u32, py as u32, yarn_col);
            }
        }
    }
    dst
}

/// 18. Pyramid (Concentric 3D stepped pyramid with directional relief shading)
pub fn effect_pyramid(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = buffer.clone();
    let step_size = 10u32;

    for y in 0..h {
        for x in 0..w {
            let dist_x = x.min(w.saturating_sub(1 + x));
            let dist_y = y.min(h.saturating_sub(1 + y));
            let d = dist_x.min(dist_y);
            let step = (d / step_size) as i16;

            // Relief lighting based on 4-sided pyramid face
            let face_light: i16 = if y <= x && (y + x) < w {
                36 // Top face (bright light)
            } else if x <= y && (x + y) < h {
                18 // Left face (mild light)
            } else if y >= x && (y + x) >= (w + h) / 2 {
                -36 // Bottom face (deep shadow)
            } else {
                -18 // Right face (mild shadow)
            };

            let step_shade = (step % 2) * 12;
            let total_mod = face_light + step_shade;

            if let Some(p) = buffer.get_pixel(x, y) {
                let r = (p.r as i16 + total_mod).clamp(0, 255) as u8;
                let g = (p.g as i16 + total_mod).clamp(0, 255) as u8;
                let b = (p.b as i16 + total_mod).clamp(0, 255) as u8;
                dst.set_pixel(x, y, ColorRgba::rgba(r, g, b, p.a));
            }
        }
    }
    dst
}

/// 19. Pulsar (Concentric circular shockwave ripples expanding from center)
pub fn effect_pulsar(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    if w == 0 || h == 0 {
        return buffer.clone();
    }
    let mut dst = PixelBuffer::new(w, h, WHITE);
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;

    for y in 0..h {
        let dy = y as f32 - cy;
        for x in 0..w {
            let dx = x as f32 - cx;
            let dist = (dx * dx + dy * dy).sqrt();
            let dr = 8.0_f32 * (dist / 6.0_f32).sin();

            let factor = if dist > 0.001 {
                (dist + dr) / dist
            } else {
                1.0
            };
            let sx = (cx + dx * factor).round() as i32;
            let sy = (cy + dy * factor).round() as i32;

            let clamped_x = sx.clamp(0, w as i32 - 1) as u32;
            let clamped_y = sy.clamp(0, h as i32 - 1) as u32;

            if let Some(p) = buffer.get_pixel(clamped_x, clamped_y) {
                dst.set_pixel(x, y, p);
            }
        }
    }
    dst
}

/// 20. Confetti (Explosion of festive colorful confetti fragments)
pub fn effect_confetti(buffer: &PixelBuffer, rng: &mut u64) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    if w == 0 || h == 0 {
        return dst;
    }

    let confetti_colors = [
        ColorRgba::rgb(255, 40, 100), // Hot pink
        ColorRgba::rgb(40, 220, 255), // Cyan
        ColorRgba::rgb(255, 230, 20), // Yellow
        ColorRgba::rgb(80, 240, 60),  // Bright green
        ColorRgba::rgb(255, 130, 30), // Orange
        ColorRgba::rgb(180, 70, 255), // Purple
        ColorRgba::rgb(255, 255, 255),
    ];

    let count = (w * h / 700).clamp(60, 350);
    for _ in 0..count {
        let cx = (next_mixer_rand(rng) % w) as i32;
        let cy = (next_mixer_rand(rng) % h) as i32;
        let color = confetti_colors[(next_mixer_rand(rng) as usize) % confetti_colors.len()];
        let cw = (next_mixer_rand(rng) % 6 + 3) as i32;
        let ch = (next_mixer_rand(rng) % 6 + 3) as i32;

        for dy in 0..ch {
            for dx in 0..cw {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    dst.set_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
    dst
}

/// 21. Dancing Disks (Pop-art circular color-averaged disks)
pub fn effect_dancing_disks(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let cell_size = 12u32;
    let radius = 5i32;
    let bg_color = ColorRgba::rgb(20, 24, 32);
    let mut dst = PixelBuffer::new(w, h, bg_color);

    let num_x = w.div_ceil(cell_size);
    let num_y = h.div_ceil(cell_size);

    for cy in 0..num_y {
        for cx in 0..num_x {
            let start_x = cx * cell_size;
            let start_y = cy * cell_size;
            let center_x = (start_x + cell_size / 2) as i32;
            let center_y = (start_y + cell_size / 2) as i32;

            // Compute cell average color
            let mut sum_r = 0u64;
            let mut sum_g = 0u64;
            let mut sum_b = 0u64;
            let mut count = 0u64;

            for dy in 0..cell_size {
                let py = start_y + dy;
                if py >= h {
                    continue;
                }
                for dx in 0..cell_size {
                    let px = start_x + dx;
                    if px >= w {
                        continue;
                    }
                    if let Some(p) = buffer.get_pixel(px, py) {
                        sum_r += p.r as u64;
                        sum_g += p.g as u64;
                        sum_b += p.b as u64;
                        count += 1;
                    }
                }
            }

            if count == 0 {
                continue;
            }
            let avg_col = ColorRgba::rgb(
                (sum_r / count) as u8,
                (sum_g / count) as u8,
                (sum_b / count) as u8,
            );

            // Draw filled disk
            for dy in -radius..=radius {
                let py = center_y + dy;
                if py < 0 || py >= h as i32 {
                    continue;
                }
                for dx in -radius..=radius {
                    let px = center_x + dx;
                    if px < 0 || px >= w as i32 {
                        continue;
                    }
                    if dx * dx + dy * dy <= radius * radius {
                        dst.set_pixel(px as u32, py as u32, avg_col);
                    }
                }
            }
        }
    }
    dst
}

/// 22. Twinkle, Twinkle (Star glints and diamond sparkles)
pub fn effect_twinkle_twinkle(buffer: &PixelBuffer, rng: &mut u64) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    if w == 0 || h == 0 {
        return dst;
    }

    let star_count = (w * h / 1600).clamp(25, 120);
    for _ in 0..star_count {
        let cx = (next_mixer_rand(rng) % w) as i32;
        let cy = (next_mixer_rand(rng) % h) as i32;
        let star_size = (next_mixer_rand(rng) % 4 + 3) as i32;

        // Draw 4-pointed diamond glint
        let white = ColorRgba::rgb(255, 255, 255);
        let ray_color = ColorRgba::rgb(180, 230, 255);

        for d in 1..=star_size {
            for (dx, dy) in [(0, -d), (0, d), (-d, 0), (d, 0)] {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    dst.set_pixel(px as u32, py as u32, ray_color);
                }
            }
        }
        // Center core
        if cx >= 0 && cx < w as i32 && cy >= 0 && cy < h as i32 {
            dst.set_pixel(cx as u32, cy as u32, white);
        }
    }
    dst
}

/// 23. Subway (Glossy beveled subway tile brick mosaic)
pub fn effect_subway(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = buffer.clone();
    let tile_w = 26u32;
    let tile_h = 13u32;
    let grout_col = ColorRgba::rgb(35, 38, 45);

    for y in 0..h {
        let row = y / tile_h;
        let offset_x = if row % 2 == 1 { tile_w / 2 } else { 0 };
        let local_y = y % tile_h;

        for x in 0..w {
            let adjusted_x = (x + offset_x) % tile_w;

            // Grout lines
            if local_y == 0 || adjusted_x == 0 {
                dst.set_pixel(x, y, grout_col);
            } else if local_y == 1 || adjusted_x == 1 {
                // Bevel highlight
                if let Some(p) = buffer.get_pixel(x, y) {
                    let r = p.r.saturating_add(60);
                    let g = p.g.saturating_add(60);
                    let b = p.b.saturating_add(60);
                    dst.set_pixel(x, y, ColorRgba::rgba(r, g, b, p.a));
                }
            } else if local_y == tile_h - 1 || adjusted_x == tile_w - 1 {
                // Bevel shadow
                if let Some(p) = buffer.get_pixel(x, y) {
                    let r = p.r.saturating_sub(60);
                    let g = p.g.saturating_sub(60);
                    let b = p.b.saturating_sub(60);
                    dst.set_pixel(x, y, ColorRgba::rgba(r, g, b, p.a));
                }
            }
        }
    }
    dst
}

/// 24. Dancing Trees (Kid Pix procedural mini pine trees across canvas)
pub fn effect_dancing_trees(buffer: &PixelBuffer, rng: &mut u64) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    if w == 0 || h == 0 {
        return dst;
    }

    let tree_count = (w / 22).clamp(8, 32);
    for _ in 0..tree_count {
        let base_x = (next_mixer_rand(rng) % (w - 10).max(1) + 5) as i32;
        let base_y = (next_mixer_rand(rng) % (h / 2).max(1) + h / 2) as i32;

        let trunk_col = ColorRgba::rgb(115, 65, 30);
        let green_col = ColorRgba::rgb(35, 160, 55);
        let green_light = ColorRgba::rgb(65, 205, 80);

        // Trunk
        for ty in 0..6 {
            let py = base_y - ty;
            if py >= 0 && py < h as i32 {
                dst.set_pixel(base_x as u32, py as u32, trunk_col);
            }
        }

        // 3 Tiers of foliage
        for tier in 0..3 {
            let tier_y = base_y - 6 - tier * 4;
            let width = 5 - tier;
            for dy in 0..3 {
                let py = tier_y - dy;
                if py < 0 || py >= h as i32 {
                    continue;
                }
                for dx in -width..=width {
                    let px = base_x + dx;
                    if px >= 0 && px < w as i32 {
                        let c = if dx < 0 { green_light } else { green_col };
                        dst.set_pixel(px as u32, py as u32, c);
                    }
                }
            }
        }
    }
    dst
}

/// 25. Color Coins (Shiny embossed metallic coins)
pub fn effect_color_coins(buffer: &PixelBuffer, rng: &mut u64) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    if w == 0 || h == 0 {
        return dst;
    }

    let coin_count = (w * h / 3200).clamp(10, 36);
    let coin_metals = [
        (
            ColorRgba::rgb(255, 215, 0),
            ColorRgba::rgb(255, 245, 140),
            ColorRgba::rgb(180, 140, 0),
        ), // Gold
        (
            ColorRgba::rgb(210, 215, 225),
            ColorRgba::rgb(250, 250, 255),
            ColorRgba::rgb(140, 145, 155),
        ), // Silver
        (
            ColorRgba::rgb(205, 127, 50),
            ColorRgba::rgb(245, 175, 110),
            ColorRgba::rgb(140, 80, 30),
        ), // Bronze
    ];

    for _ in 0..coin_count {
        let cx = (next_mixer_rand(rng) % w) as i32;
        let cy = (next_mixer_rand(rng) % h) as i32;
        let radius = (next_mixer_rand(rng) % 4 + 7) as i32;
        let (body, highlight, shadow) =
            coin_metals[(next_mixer_rand(rng) as usize) % coin_metals.len()];

        for dy in -radius..=radius {
            let py = cy + dy;
            if py < 0 || py >= h as i32 {
                continue;
            }
            for dx in -radius..=radius {
                let px = cx + dx;
                if px < 0 || px >= w as i32 {
                    continue;
                }
                let r2 = dx * dx + dy * dy;
                if r2 <= radius * radius {
                    let col = if r2 >= (radius - 1) * (radius - 1) {
                        if dx + dy < 0 {
                            highlight
                        } else {
                            shadow
                        }
                    } else if r2 <= 4 {
                        highlight
                    } else {
                        body
                    };
                    dst.set_pixel(px as u32, py as u32, col);
                }
            }
        }
    }
    dst
}

/// 26. Dog Bones (Scattered retro cartoon dog bones)
pub fn effect_dog_bones(buffer: &PixelBuffer, rng: &mut u64) -> PixelBuffer {
    let mut dst = buffer.clone();
    let w = dst.width();
    let h = dst.height();
    if w == 0 || h == 0 {
        return dst;
    }

    let bone_count = (w * h / 2400).clamp(12, 40);
    for _ in 0..bone_count {
        let cx = (next_mixer_rand(rng) % (w - 12).max(1) + 6) as i32;
        let cy = (next_mixer_rand(rng) % (h - 12).max(1) + 6) as i32;
        let is_diag = next_mixer_rand(rng) % 2 == 1;

        let bone_white = ColorRgba::rgb(255, 255, 245);

        if !is_diag {
            // Horizontal Dog Bone:
            // (**)======(**)
            for x in (cx - 7)..=(cx + 7) {
                for y in (cy - 1)..=(cy + 1) {
                    if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                        dst.set_pixel(x as u32, y as u32, bone_white);
                    }
                }
            }
            // 4 End Knobs
            for (kx, ky) in [
                (cx - 7, cy - 3),
                (cx - 7, cy + 3),
                (cx + 7, cy - 3),
                (cx + 7, cy + 3),
            ] {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let px = kx + dx;
                        let py = ky + dy;
                        if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                            dst.set_pixel(px as u32, py as u32, bone_white);
                        }
                    }
                }
            }
        } else {
            // Diagonal Dog Bone
            for d in -6..=6 {
                for offset in -1..=1 {
                    let px = cx + d + offset;
                    let py = cy + d - offset;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        dst.set_pixel(px as u32, py as u32, bone_white);
                    }
                }
            }
        }
    }
    dst
}

/// 27. Stripes (Vibrant pop-art alternating color stripes)
pub fn effect_stripes(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = buffer.clone();
    let stripe_w = 12u32;

    for y in 0..h {
        let is_stripe = (y / stripe_w) % 2 == 1;
        if !is_stripe {
            continue;
        }
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                // Invert and hue modulate stripe
                let r = 255 - p.r;
                let g = (p.g as u16 + 40).min(255) as u8;
                let b = (p.b as u16 + 80).min(255) as u8;
                dst.set_pixel(x, y, ColorRgba::rgba(r, g, b, p.a));
            }
        }
    }
    dst
}

/// 28. Neon Tube (Electric glowing neon outlines on midnight dark canvas)
pub fn effect_neon_tube(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let bg_midnight = ColorRgba::rgb(10, 12, 22);
    let mut dst = PixelBuffer::new(w, h, bg_midnight);

    let neon_palette = [
        ColorRgba::rgb(255, 30, 140), // Hot Neon Pink
        ColorRgba::rgb(0, 240, 255),  // Electric Cyan
        ColorRgba::rgb(60, 255, 80),  // Acid Green
        ColorRgba::rgb(255, 230, 0),  // Neon Yellow
    ];

    let threshold = 30.0_f32;

    // Detect edges and paint glowing neon lines
    for y in 0..h {
        for x in 0..w {
            let c = match buffer.get_pixel(x, y) {
                Some(p) => p,
                None => continue,
            };
            let lum = pixel_luminance(c);
            let lum_r = if x + 1 < w {
                pixel_luminance(buffer.get_pixel(x + 1, y).unwrap_or(c))
            } else {
                lum
            };
            let lum_d = if y + 1 < h {
                pixel_luminance(buffer.get_pixel(x, y + 1).unwrap_or(c))
            } else {
                lum
            };

            let diff = (lum - lum_r).abs().max((lum - lum_d).abs());
            if diff >= threshold {
                let color_idx = ((x / 16 + y / 16) as usize) % neon_palette.len();
                let neon_core = neon_palette[color_idx];

                // Core pixel
                dst.set_pixel(x, y, ColorRgba::rgb(255, 255, 255));

                // 1px Neon Halo
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let hx = x as i32 + dx;
                    let hy = y as i32 + dy;
                    if hx >= 0
                        && hx < w as i32
                        && hy >= 0
                        && hy < h as i32
                        && dst.get_pixel(hx as u32, hy as u32) == Some(bg_midnight)
                    {
                        dst.set_pixel(hx as u32, hy as u32, neon_core);
                    }
                }
            }
        }
    }
    dst
}

// ============================================================================
// HELPER DRAWING FUNCTIONS
// ============================================================================

/// Helper: Draw a simple 1px line using Bresenham algorithm.
fn draw_simple_line(
    buffer: &mut PixelBuffer,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: ColorRgba,
) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && x0 < buffer.width() as i32 && y0 >= 0 && y0 < buffer.height() as i32 {
            buffer.set_pixel(x0 as u32, y0 as u32, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_28_mixer_effects_dispatch() {
        let mut buffer = PixelBuffer::new(64, 48, WHITE);
        // Draw some pattern to give the effects texture to manipulate
        for y in 10..38 {
            for x in 10..54 {
                buffer.set_pixel(x, y, ColorRgba::rgb(200, 50, 80));
            }
        }

        for &effect in &MixerEffect::ALL {
            let res = apply_mixer_effect(&buffer, effect, 123456789);
            if effect == MixerEffect::Spin {
                assert_eq!(res.width(), 48);
                assert_eq!(res.height(), 64);
            } else {
                assert_eq!(res.width(), 64);
                assert_eq!(res.height(), 48);
            }
        }
    }

    #[test]
    fn test_tray_assignment() {
        for &effect in &MixerEffect::TRAY_1 {
            assert_eq!(effect.tray(), MixerTray::Tray1);
        }
        for &effect in &MixerEffect::TRAY_2 {
            assert_eq!(effect.tray(), MixerTray::Tray2);
        }
        assert_eq!(MixerEffect::TRAY_1.len(), 14);
        assert_eq!(MixerEffect::TRAY_2.len(), 14);
        assert_eq!(MixerEffect::ALL.len(), 28);
    }

    #[test]
    fn test_deterministic_reproducibility() {
        let buffer = PixelBuffer::new(32, 32, ColorRgba::rgb(100, 150, 200));
        let res1 = apply_mixer_effect(&buffer, MixerEffect::Confetti, 42);
        let res2 = apply_mixer_effect(&buffer, MixerEffect::Confetti, 42);
        assert_eq!(res1, res2);

        let res3 = apply_mixer_effect(&buffer, MixerEffect::Confetti, 9999);
        assert_ne!(res1, res3);
    }
}

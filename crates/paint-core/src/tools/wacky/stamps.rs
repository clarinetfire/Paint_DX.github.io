//! Rubber Stamps inspired by Kid Pix.
//!
//! Pure Rust implementation (#![forbid(unsafe_code)]) with deterministic algorithms,
//! full RGBA support, and zero UI dependencies.

use crate::geometry::Point;
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum StampShape {
    #[default]
    Smiley,
    Heart,
    Star,
    MusicNote,
    Bomb,
    Sun,
    Flower,
    Tree,
    Cat,
    Ghost,
    Car,
    Gift,
    Lightning,
    Moon,
    Diamond,
    ThumbsUp,
}

impl StampShape {
    pub const ALL: [StampShape; 16] = [
        StampShape::Smiley,
        StampShape::Heart,
        StampShape::Star,
        StampShape::MusicNote,
        StampShape::Bomb,
        StampShape::Sun,
        StampShape::Flower,
        StampShape::Tree,
        StampShape::Cat,
        StampShape::Ghost,
        StampShape::Car,
        StampShape::Gift,
        StampShape::Lightning,
        StampShape::Moon,
        StampShape::Diamond,
        StampShape::ThumbsUp,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            StampShape::Smiley => "Smiley / Sourire",
            StampShape::Heart => "Heart / Cœur",
            StampShape::Star => "Star / Étoile",
            StampShape::MusicNote => "Music / Musique",
            StampShape::Bomb => "Bomb / Bombe",
            StampShape::Sun => "Sun / Soleil",
            StampShape::Flower => "Flower / Fleur",
            StampShape::Tree => "Tree / Arbre",
            StampShape::Cat => "Cat / Chat",
            StampShape::Ghost => "Ghost / Fantôme",
            StampShape::Car => "Car / Voiture",
            StampShape::Gift => "Gift / Cadeau",
            StampShape::Lightning => "Lightning / Éclair",
            StampShape::Moon => "Moon / Lune",
            StampShape::Diamond => "Diamond / Diamant",
            StampShape::ThumbsUp => "Thumbs Up / Bravo",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            StampShape::Smiley => "🙂",
            StampShape::Heart => "❤️",
            StampShape::Star => "⭐",
            StampShape::MusicNote => "🎵",
            StampShape::Bomb => "💣",
            StampShape::Sun => "☀️",
            StampShape::Flower => "🌸",
            StampShape::Tree => "🌲",
            StampShape::Cat => "🐱",
            StampShape::Ghost => "👻",
            StampShape::Car => "🚗",
            StampShape::Gift => "🎁",
            StampShape::Lightning => "⚡",
            StampShape::Moon => "🌙",
            StampShape::Diamond => "💎",
            StampShape::ThumbsUp => "👍",
        }
    }

    /// 12x12 1-bit pixel bitmap for each stamp (1 = foreground, 0 = transparent)
    pub fn bitmap(&self) -> &'static [&'static str; 12] {
        match self {
            StampShape::Smiley => &[
                "....XXXX....",
                "..XX....XX..",
                ".X........X.",
                "X...X..X...X",
                "X...X..X...X",
                "X..........X",
                "X..X....X..X",
                "X...XXXX...X",
                ".X........X.",
                "..XX....XX..",
                "....XXXX....",
                "............",
            ],
            StampShape::Heart => &[
                "..XXXX..XXXX",
                ".XXXXXX.XXXX",
                "XXXXXXXXXXXX",
                "XXXXXXXXXXXX",
                "XXXXXXXXXXXX",
                ".XXXXXXXXXX.",
                "..XXXXXXXX..",
                "...XXXXXX...",
                "....XXXX....",
                ".....XX.....",
                "......X.....",
                "............",
            ],
            StampShape::Star => &[
                ".....XX.....",
                ".....XX.....",
                "....XXXX....",
                "XXXXXXXXXXXX",
                ".XXXXXXXXXX.",
                "..XXXXXXXX..",
                "...XXXXXX...",
                "..XXXXXXXX..",
                ".XX..XX..XX.",
                "XX........XX",
                "X..........X",
                "............",
            ],
            StampShape::MusicNote => &[
                ".....XXXXXX.",
                ".....X....X.",
                ".....X....X.",
                ".....X....X.",
                ".....X....X.",
                "..XXXX....X.",
                ".XXXXX.XXXXX",
                ".XXXXX.XXXXX",
                "..XXX...XXX.",
                "............",
                "............",
                "............",
            ],
            StampShape::Bomb => &[
                ".......XX...",
                "......X..X..",
                ".....X......",
                "...XXXXX....",
                "..XXXXXXX...",
                ".XXXXXXXXX..",
                ".XXXXXXXXX..",
                ".XXXXXXXXX..",
                "..XXXXXXX...",
                "...XXXXX....",
                "............",
                "............",
            ],
            StampShape::Sun => &[
                "X....XX....X",
                ".X...XX...X.",
                "..X..XX..X..",
                "...XXXXXX...",
                "XXXXXXXXXXXX",
                "XXXXXXXXXXXX",
                "...XXXXXX...",
                "..X..XX..X..",
                ".X...XX...X.",
                "X....XX....X",
                "............",
                "............",
            ],
            StampShape::Flower => &[
                "....XXXX....",
                "...XXXXXX...",
                ".XX..XX..XX.",
                "XXXX.XX.XXXX",
                ".XX..XX..XX.",
                "...XXXXXX...",
                "....XXXX....",
                ".....XX.....",
                "....XXXX....",
                ".....XX.....",
                ".....XX.....",
                "............",
            ],
            StampShape::Tree => &[
                ".....XX.....",
                "....XXXX....",
                "...XXXXXX...",
                "....XXXX....",
                "...XXXXXX...",
                "..XXXXXXXX..",
                ".XXXXXXXXXX.",
                "....XXXX....",
                ".....XX.....",
                ".....XX.....",
                "....XXXX....",
                "............",
            ],
            StampShape::Cat => &[
                "X..........X",
                "XX........XX",
                "XXX..XX..XXX",
                "XXXXXXXXXXXX",
                "XXXXXXXXXXXX",
                "XX.X.XX.X.XX",
                "XXXXXXXXXXXX",
                ".XXXXXXXXXX.",
                "..X.XXXX.X..",
                "...XXXXXX...",
                "............",
                "............",
            ],
            StampShape::Ghost => &[
                "....XXXX....",
                "..XXXXXXXX..",
                ".XXXXXXXXXX.",
                ".XX.XXXX.XX.",
                ".XX.XXXX.XX.",
                ".XXXXXXXXXX.",
                ".XXXXXXXXXX.",
                ".XXXXXXXXXX.",
                ".X.XX..XX.X.",
                "X..XX..XX..X",
                "............",
                "............",
            ],
            StampShape::Car => &[
                "............",
                "....XXXX....",
                "...XXXXXX...",
                "..XXXXXXXX..",
                ".XXXXXXXXXX.",
                "XXXXXXXXXXXX",
                "XXXXXXXXXXXX",
                ".XX.XXXX.XX.",
                "..X......X..",
                "............",
                "............",
                "............",
            ],
            StampShape::Gift => &[
                "....XXXX....",
                "...XX..XX...",
                "..XXXXXXXX..",
                "..XX.XX.XX..",
                ".XXXXXXXXXX.",
                ".XXXXXXXXXX.",
                ".XX.XXXX.XX.",
                ".XXXXXXXXXX.",
                ".XXXXXXXXXX.",
                "..XXXXXXXX..",
                "............",
                "............",
            ],
            StampShape::Lightning => &[
                "......XXXX..",
                ".....XXXX...",
                "....XXXX....",
                "...XXXX.....",
                "..XXXXXXXX..",
                "......XXXX..",
                ".....XXXX...",
                "....XXXX....",
                "...XXXX.....",
                "..XX........",
                ".X..........",
                "............",
            ],
            StampShape::Moon => &[
                "....XXXX....",
                "...XXXXXX...",
                "..XXXXX.....",
                ".XXXX.......",
                ".XXXX.......",
                ".XXXX.......",
                ".XXXX.......",
                "..XXXXX.....",
                "...XXXXXX...",
                "....XXXX....",
                "............",
                "............",
            ],
            StampShape::Diamond => &[
                "....XXXX....",
                "..XXXXXXXX..",
                ".XXXXXXXXXX.",
                "XXXXXXXXXXXX",
                ".XXXXXXXXXX.",
                "..XXXXXXXX..",
                "...XXXXXX...",
                "....XXXX....",
                ".....XX.....",
                "......X.....",
                "............",
                "............",
            ],
            StampShape::ThumbsUp => &[
                ".....XX.....",
                "....XXXX....",
                "....XXXX....",
                "...XXXXX....",
                "..XXXXXXXX..",
                ".XXXXXXXXXX.",
                ".XXXXXXXXXX.",
                "..XXXXXXXX..",
                "...XXXXXX...",
                "....XXXX....",
                "............",
                "............",
            ],
        }
    }
    /// Iconic preset color for each stamp in Kid Pix signature palette.
    pub fn default_color(&self) -> ColorRgba {
        match self {
            StampShape::Smiley => ColorRgba::rgb(255, 215, 0),
            StampShape::Heart => ColorRgba::rgb(230, 30, 60),
            StampShape::Star => ColorRgba::rgb(255, 230, 20),
            StampShape::MusicNote => ColorRgba::rgb(50, 80, 220),
            StampShape::Bomb => ColorRgba::rgb(40, 40, 40),
            StampShape::Sun => ColorRgba::rgb(255, 165, 0),
            StampShape::Flower => ColorRgba::rgb(240, 80, 160),
            StampShape::Tree => ColorRgba::rgb(34, 145, 55),
            StampShape::Cat => ColorRgba::rgb(180, 100, 40),
            StampShape::Ghost => ColorRgba::rgb(220, 235, 255),
            StampShape::Car => ColorRgba::rgb(220, 50, 40),
            StampShape::Gift => ColorRgba::rgb(150, 40, 200),
            StampShape::Lightning => ColorRgba::rgb(255, 240, 0),
            StampShape::Moon => ColorRgba::rgb(240, 240, 180),
            StampShape::Diamond => ColorRgba::rgb(40, 200, 240),
            StampShape::ThumbsUp => ColorRgba::rgb(255, 200, 150),
        }
    }
}

/// Color rendering and tinting modes for rubber stamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum StampTint {
    /// 1-bit monochrome bitmap stamped with the specified foreground color (classic XP style).
    Monochrome(ColorRgba),
    /// Two-tone stamp: filled interior with `fg`, background bounding box with `bg`.
    TwoTone { fg: ColorRgba, bg: ColorRgba },
    /// Primary fill with a 1-pixel crisp outer border drawn in `outline`.
    Outlined { fill: ColorRgba, outline: ColorRgba },
    /// Iconic signature preset colors for each Kid Pix stamp.
    #[default]
    OriginalPalette,
}

/// Stamp trail spacing behavior during drag interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum StampSpacing {
    /// Single stamp per click (drag does not leave continuous trail).
    Single,
    /// Automatically spaced trail based on stamp scale: (12 * scale) px.
    #[default]
    Auto,
    /// Explicit pixel spacing interval between consecutive stamps along the drag path.
    Interval(u32),
}

/// Precision options for rubber stamp placement, orientation, and color styling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StampOptions {
    /// Scale factor (1x = 12x12 px, 2x = 24x24 px, up to 16x).
    pub scale: u32,
    /// Rotation in degrees (supports 15° steps, 90° quadrants, or arbitrary angles).
    pub rotation_deg: f32,
    /// Spacing behavior when dragged continuously across the canvas.
    pub spacing: StampSpacing,
    /// Tinting and multi-color rendering style.
    pub tint: StampTint,
}

impl Default for StampOptions {
    fn default() -> Self {
        Self {
            scale: 2,
            rotation_deg: 0.0,
            spacing: StampSpacing::Auto,
            tint: StampTint::OriginalPalette,
        }
    }
}

/// Applies a rubber stamp with full fine-grained options (scale, rotation, tinting, outline).
pub fn apply_stamp_with_options(
    buffer: &mut PixelBuffer,
    center: Point,
    stamp: StampShape,
    options: &StampOptions,
) {
    let scale = options.scale.clamp(1, 16) as i32;
    let bitmap = stamp.bitmap();
    let rows = bitmap.len() as i32;
    let cols = 12;

    let (primary_color, secondary_color, is_two_tone, is_outlined) = match options.tint {
        StampTint::Monochrome(c) => (c, ColorRgba::TRANSPARENT, false, false),
        StampTint::TwoTone { fg, bg } => (fg, bg, true, false),
        StampTint::Outlined { fill, outline } => (fill, outline, false, true),
        StampTint::OriginalPalette => (stamp.default_color(), ColorRgba::TRANSPARENT, false, false),
    };

    let norm_deg = (options.rotation_deg % 360.0 + 360.0) % 360.0;
    let is_exact_quadrant =
        (norm_deg.round() - norm_deg).abs() < 0.001 && ((norm_deg.round() as i32) % 90 == 0);

    if is_exact_quadrant {
        let rot = ((norm_deg.round() as i32) / 90) % 4;
        let half_w = (cols * scale) / 2;
        let half_h = (rows * scale) / 2;

        let mut fg_pixels = Vec::new();

        for (r, row_str) in bitmap.iter().enumerate() {
            for (c, ch) in row_str.chars().enumerate() {
                let (rc, rr) = match rot {
                    1 => (cols - 1 - (r as i32), c as i32),
                    2 => (cols - 1 - (c as i32), rows - 1 - (r as i32)),
                    3 => (r as i32, rows - 1 - (c as i32)),
                    _ => (c as i32, r as i32),
                };
                let px = center.x - half_w + rc * scale;
                let py = center.y - half_h + rr * scale;

                if ch == 'X' {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            fg_pixels.push((px + sx, py + sy));
                        }
                    }
                } else if is_two_tone {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            buffer.set_pixel_i32(px + sx, py + sy, secondary_color);
                        }
                    }
                }
            }
        }

        if is_outlined {
            // Render 1px outline around fg_pixels
            let mut outline_set = std::collections::HashSet::new();
            let fg_set: std::collections::HashSet<(i32, i32)> = fg_pixels.iter().copied().collect();

            for &(fx, fy) in &fg_pixels {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let neighbor = (fx + dx, fy + dy);
                        if !fg_set.contains(&neighbor) {
                            outline_set.insert(neighbor);
                        }
                    }
                }
            }

            for (ox, oy) in outline_set {
                buffer.set_pixel_i32(ox, oy, secondary_color);
            }
        }

        for (fx, fy) in fg_pixels {
            buffer.set_pixel_i32(fx, fy, primary_color);
        }
    } else {
        // Arbitrary nearest-neighbor rotation
        let rad = -norm_deg * std::f32::consts::PI / 180.0;
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        let w_f = (cols * scale) as f32;
        let h_f = (rows * scale) as f32;
        let half_w = (w_f.abs() * cos_a.abs() + h_f.abs() * sin_a.abs()) * 0.5;
        let half_h = (w_f.abs() * sin_a.abs() + h_f.abs() * cos_a.abs()) * 0.5;
        let rw = (half_w.ceil() as i32).max(1);
        let rh = (half_h.ceil() as i32).max(1);

        let mut fg_pixels = Vec::new();
        let center_x_f = (cols as f32 * scale as f32) * 0.5;
        let center_y_f = (rows as f32 * scale as f32) * 0.5;

        for dy in -rh..=rh {
            for dx in -rw..=rw {
                // Inverse rotate to find source pixel in stamp
                let src_x = (dx as f32 * cos_a - dy as f32 * sin_a) + center_x_f;
                let src_y = (dx as f32 * sin_a + dy as f32 * cos_a) + center_y_f;

                let c = (src_x / scale as f32).floor() as i32;
                let r = (src_y / scale as f32).floor() as i32;

                if c >= 0 && c < cols && r >= 0 && r < rows {
                    let ch = bitmap[r as usize].as_bytes()[c as usize];
                    let px = center.x + dx;
                    let py = center.y + dy;

                    if ch == b'X' {
                        fg_pixels.push((px, py));
                    } else if is_two_tone {
                        buffer.set_pixel_i32(px, py, secondary_color);
                    }
                }
            }
        }

        if is_outlined {
            let mut outline_set = std::collections::HashSet::new();
            let fg_set: std::collections::HashSet<(i32, i32)> = fg_pixels.iter().copied().collect();

            for &(fx, fy) in &fg_pixels {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let neighbor = (fx + dx, fy + dy);
                        if !fg_set.contains(&neighbor) {
                            outline_set.insert(neighbor);
                        }
                    }
                }
            }

            for (ox, oy) in outline_set {
                buffer.set_pixel_i32(ox, oy, secondary_color);
            }
        }

        for (fx, fy) in fg_pixels {
            buffer.set_pixel_i32(fx, fy, primary_color);
        }
    }
}

/// Applies a rubber stamp on the pixel buffer centered at `center` with rotation (0, 90, 180, 270 degrees).
pub fn apply_stamp_rotated(
    buffer: &mut PixelBuffer,
    center: Point,
    stamp: StampShape,
    scale: u32,
    rotation_deg: u16,
    fg: ColorRgba,
) {
    let options = StampOptions {
        scale,
        rotation_deg: rotation_deg as f32,
        spacing: StampSpacing::Auto,
        tint: StampTint::Monochrome(fg),
    };
    apply_stamp_with_options(buffer, center, stamp, &options);
}

/// Applies a rubber stamp on the pixel buffer centered at `center`.
pub fn apply_stamp(
    buffer: &mut PixelBuffer,
    center: Point,
    stamp: StampShape,
    scale: u32,
    fg: ColorRgba,
) {
    apply_stamp_rotated(buffer, center, stamp, scale, 0, fg);
}

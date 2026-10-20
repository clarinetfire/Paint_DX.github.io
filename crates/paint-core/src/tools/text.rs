//! Text tool implementation with pure-Rust rasterization and opaque/transparent background support.

use crate::geometry::Point;
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use crate::tools::selection::SelectionMode;
use serde::{Deserialize, Serialize};

// Classic 8x8 bitmap font table for ASCII characters 32..127
pub static ASCII_FONT_8X8: [[u8; 8]; 96] = [
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // ' '
    [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00], // '!'
    [0x66, 0x66, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00], // '"'
    [0x6C, 0x6C, 0xFE, 0x6C, 0xFE, 0x6C, 0x6C, 0x00], // '#'
    [0x18, 0x7E, 0xC0, 0x7C, 0x06, 0xFC, 0x18, 0x00], // '$'
    [0x00, 0xC6, 0xCC, 0x18, 0x30, 0x66, 0xC6, 0x00], // '%'
    [0x38, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0x76, 0x00], // '&'
    [0x60, 0x60, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00], // '''
    [0x18, 0x30, 0x60, 0x60, 0x60, 0x30, 0x18, 0x00], // '('
    [0x18, 0x0C, 0x06, 0x06, 0x06, 0x0C, 0x18, 0x00], // ')'
    [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00], // '*'
    [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00], // '+'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30], // ','
    [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00], // '-'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00], // '.'
    [0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x80, 0x00], // '/'
    [0x7C, 0xC6, 0xCE, 0xD6, 0xE6, 0xC6, 0x7C, 0x00], // '0'
    [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00], // '1'
    [0x7C, 0xC6, 0x06, 0x1C, 0x30, 0x60, 0xFE, 0x00], // '2'
    [0x7C, 0xC6, 0x06, 0x3C, 0x06, 0xC6, 0x7C, 0x00], // '3'
    [0x1C, 0x3C, 0x6C, 0xCC, 0xFE, 0x0C, 0x1E, 0x00], // '4'
    [0xFE, 0xC0, 0xFC, 0x06, 0x06, 0xC6, 0x7C, 0x00], // '5'
    [0x7C, 0xC6, 0xC0, 0xFC, 0xC6, 0xC6, 0x7C, 0x00], // '6'
    [0xFE, 0xC6, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00], // '7'
    [0x7C, 0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0x7C, 0x00], // '8'
    [0x7C, 0xC6, 0xC6, 0x7E, 0x06, 0xC6, 0x7C, 0x00], // '9'
    [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00], // ':'
    [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x30, 0x00], // ';'
    [0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00], // '<'
    [0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00], // '='
    [0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x00], // '>'
    [0x7C, 0xC6, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00], // '?'
    [0x7C, 0xC6, 0xDE, 0xDE, 0xDC, 0xC0, 0x7C, 0x00], // '@'
    [0x38, 0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0x00], // 'A'
    [0xFC, 0x66, 0x66, 0x7C, 0x66, 0x66, 0xFC, 0x00], // 'B'
    [0x3C, 0x66, 0xC0, 0xC0, 0xC0, 0x66, 0x3C, 0x00], // 'C'
    [0xF8, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0xF8, 0x00], // 'D'
    [0xFE, 0x62, 0x68, 0x78, 0x68, 0x62, 0xFE, 0x00], // 'E'
    [0xFE, 0x62, 0x68, 0x78, 0x68, 0x60, 0xF0, 0x00], // 'F'
    [0x3C, 0x66, 0xC0, 0xC0, 0xCE, 0x66, 0x3E, 0x00], // 'G'
    [0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0x00], // 'H'
    [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'I'
    [0x1E, 0x0C, 0x0C, 0x0C, 0xCC, 0xCC, 0x78, 0x00], // 'J'
    [0xE6, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0xE6, 0x00], // 'K'
    [0xF0, 0x60, 0x60, 0x60, 0x62, 0x66, 0xFE, 0x00], // 'L'
    [0xC6, 0xEE, 0xFE, 0xFE, 0xD6, 0xC6, 0xC6, 0x00], // 'M'
    [0xC6, 0xE6, 0xF6, 0xDE, 0xCE, 0xC6, 0xC6, 0x00], // 'N'
    [0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00], // 'O'
    [0xFC, 0x66, 0x66, 0x7C, 0x60, 0x60, 0xF0, 0x00], // 'P'
    [0x7C, 0xC6, 0xC6, 0xC6, 0xD6, 0xDE, 0x7C, 0x06], // 'Q'
    [0xFC, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0xE6, 0x00], // 'R'
    [0x7C, 0xC6, 0x60, 0x38, 0x0C, 0xC6, 0x7C, 0x00], // 'S'
    [0x7E, 0x5A, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'T'
    [0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00], // 'U'
    [0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00], // 'V'
    [0xC6, 0xC6, 0xD6, 0xFE, 0xFE, 0xEE, 0xC6, 0x00], // 'W'
    [0xC6, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0xC6, 0x00], // 'X'
    [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x3C, 0x00], // 'Y'
    [0xFE, 0xC6, 0x0C, 0x18, 0x30, 0x62, 0xFE, 0x00], // 'Z'
    [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00], // '['
    [0xC0, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x02, 0x00], // '\'
    [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00], // ']'
    [0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00], // '^'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF], // '_'
    [0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00], // '`'
    [0x00, 0x00, 0x78, 0x0C, 0x7C, 0xCC, 0x76, 0x00], // 'a'
    [0xE0, 0x60, 0x7C, 0x66, 0x66, 0x66, 0xDC, 0x00], // 'b'
    [0x00, 0x00, 0x7C, 0xC6, 0xC0, 0xC6, 0x7C, 0x00], // 'c'
    [0x1C, 0x0C, 0x7C, 0xCC, 0xCC, 0xCC, 0x76, 0x00], // 'd'
    [0x00, 0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0x7C, 0x00], // 'e'
    [0x38, 0x6C, 0x60, 0xF0, 0x60, 0x60, 0xF0, 0x00], // 'f'
    [0x00, 0x00, 0x76, 0xCC, 0xCC, 0x7C, 0x0C, 0xF8], // 'g'
    [0xE0, 0x60, 0x6C, 0x76, 0x66, 0x66, 0xE6, 0x00], // 'h'
    [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'i'
    [0x06, 0x00, 0x06, 0x06, 0x06, 0x66, 0x66, 0x3C], // 'j'
    [0xE0, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0xE6, 0x00], // 'k'
    [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'l'
    [0x00, 0x00, 0xEC, 0xFE, 0xD6, 0xD6, 0xD6, 0x00], // 'm'
    [0x00, 0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x00], // 'n'
    [0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0x7C, 0x00], // 'o'
    [0x00, 0x00, 0xDC, 0x66, 0x66, 0x7C, 0x60, 0xF0], // 'p'
    [0x00, 0x00, 0x76, 0xCC, 0xCC, 0x7C, 0x0C, 0x1E], // 'q'
    [0x00, 0x00, 0xDC, 0x76, 0x60, 0x60, 0xF0, 0x00], // 'r'
    [0x00, 0x00, 0x7C, 0xC0, 0x7C, 0x06, 0xFC, 0x00], // 's'
    [0x30, 0x30, 0xFC, 0x30, 0x30, 0x34, 0x18, 0x00], // 't'
    [0x00, 0x00, 0xCC, 0xCC, 0xCC, 0xCC, 0x76, 0x00], // 'u'
    [0x00, 0x00, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00], // 'v'
    [0x00, 0x00, 0xC6, 0xD6, 0xFE, 0xFE, 0x6C, 0x00], // 'w'
    [0x00, 0x00, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0x00], // 'x'
    [0x00, 0x00, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0xFC], // 'y'
    [0x00, 0x00, 0xFE, 0x8C, 0x18, 0x32, 0xFE, 0x00], // 'z'
    [0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0x00], // '{'
    [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // '|'
    [0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0x00], // '}'
    [0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // '~'
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], // DEL / block
];

/// Returns the 8x8 bitmap row bytes for any ASCII character (32..=127).
pub fn get_ascii_glyph(ch: char) -> [u8; 8] {
    let idx = if ch.is_ascii() && (ch as usize) >= 32 && (ch as usize) <= 127 {
        ch as usize - 32
    } else {
        0
    };
    ASCII_FONT_8X8[idx]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FontFamilyChoice {
    #[default]
    Pixel8x8,
    SansSerif,
    Monospace,
    Serif,
}

impl FontFamilyChoice {
    pub const ALL: [FontFamilyChoice; 4] = [
        FontFamilyChoice::Pixel8x8,
        FontFamilyChoice::SansSerif,
        FontFamilyChoice::Monospace,
        FontFamilyChoice::Serif,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            FontFamilyChoice::Pixel8x8 => "Pixel 8x8",
            FontFamilyChoice::SansSerif => "Sans Serif",
            FontFamilyChoice::Monospace => "Monospace",
            FontFamilyChoice::Serif => "Serif",
        }
    }
}

/// Draws text string at `origin` with pixel scaling factor `scale` (1 = 8x8, 2 = 16x16, etc.).
pub fn draw_text_bitmap(
    buffer: &mut PixelBuffer,
    origin: Point,
    text: &str,
    scale: u32,
    mode: SelectionMode,
    text_color: ColorRgba,
    bg_color: ColorRgba,
) {
    let scale = scale.max(1) as i32;
    let char_w = 8 * scale;
    let char_h = 8 * scale;

    let mut cur_x = origin.x;
    let mut cur_y = origin.y;

    for ch in text.chars() {
        if ch == '\n' {
            cur_x = origin.x;
            cur_y += char_h + 2 * scale;
            continue;
        }

        let glyph_idx = if ch.is_ascii() && (ch as usize) >= 32 && (ch as usize) <= 127 {
            ch as usize - 32
        } else {
            0 // Space
        };

        let glyph_rows = &ASCII_FONT_8X8[glyph_idx];

        // Draw background if Opaque mode
        if mode == SelectionMode::Opaque {
            for dy in 0..char_h {
                for dx in 0..char_w {
                    buffer.set_pixel_i32(cur_x + dx, cur_y + dy, bg_color);
                }
            }
        }

        // Draw glyph pixels
        for (row_idx, &row_byte) in glyph_rows.iter().enumerate() {
            for bit in 0..8_i32 {
                if (row_byte & (0x80 >> bit)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cur_x + bit * scale + sx;
                            let py = cur_y + (row_idx as i32) * scale + sy;
                            buffer.set_pixel_i32(px, py, text_color);
                        }
                    }
                }
            }
        }

        cur_x += char_w;
    }
}

/// Draws proportional text with optional serif decoration and crisp thresholding.
fn draw_text_proportional(
    buffer: &mut PixelBuffer,
    origin: Point,
    text: &str,
    scale: i32,
    serif: bool,
    text_color: ColorRgba,
) {
    let mut cur_x = origin.x;
    let mut cur_y = origin.y;
    let line_h = 9 * scale;

    for ch in text.chars() {
        if ch == '\n' {
            cur_x = origin.x;
            cur_y += line_h;
            continue;
        }

        let glyph_idx = if ch.is_ascii() && (ch as usize) >= 32 && (ch as usize) <= 127 {
            ch as usize - 32
        } else {
            0
        };

        let glyph_rows = &ASCII_FONT_8X8[glyph_idx];

        // Compute glyph width (trim trailing blank columns for proportional spacing)
        let mut min_col = 7;
        let mut max_col = 0;
        let mut has_pixel = false;

        for &row_byte in glyph_rows.iter() {
            if row_byte != 0 {
                has_pixel = true;
                for bit in 0..8 {
                    if (row_byte & (0x80 >> bit)) != 0 {
                        min_col = min_col.min(bit);
                        max_col = max_col.max(bit);
                    }
                }
            }
        }

        if !has_pixel {
            // Space width
            cur_x += 4 * scale;
            continue;
        }

        let start_col = if min_col > 0 && min_col <= 2 {
            min_col - 1
        } else {
            min_col
        };
        let end_col = (max_col + 1).min(8);

        for (row_idx, &row_byte) in glyph_rows.iter().enumerate() {
            for bit in start_col..end_col {
                if (row_byte & (0x80 >> bit)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cur_x + (bit - start_col) * scale + sx;
                            let py = cur_y + (row_idx as i32) * scale + sy;
                            buffer.set_pixel_i32(px, py, text_color);
                        }
                    }
                }
            }
        }

        // Add subtle serifs for serif mode on vertical stems
        if serif && scale >= 2 {
            for sy in 0..(scale / 2).max(1) {
                buffer.set_pixel_i32(cur_x - sy - 1, cur_y + line_h - scale, text_color);
            }
        }

        let char_advance = ((end_col - start_col) + 1) * scale;
        cur_x += char_advance;
    }
}

/// Draws text with selectable font family, font size, and background mode.
#[allow(clippy::too_many_arguments)]
pub fn draw_text(
    buffer: &mut PixelBuffer,
    origin: Point,
    text: &str,
    font_family: FontFamilyChoice,
    font_size: u32,
    mode: SelectionMode,
    text_color: ColorRgba,
    bg_color: ColorRgba,
) {
    let font_size = font_size.clamp(8, 128);
    let scale = (font_size / 8).max(1) as i32;

    if mode == SelectionMode::Opaque {
        // Calculate bounding box and fill background
        let lines: Vec<&str> = text.split('\n').collect();
        let max_chars = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);
        let char_w = match font_family {
            FontFamilyChoice::Pixel8x8 | FontFamilyChoice::Monospace => 8 * scale,
            FontFamilyChoice::SansSerif | FontFamilyChoice::Serif => 6 * scale,
        };
        let line_h = 10 * scale;
        let total_w = (max_chars as i32 * char_w).max(char_w);
        let total_h = (lines.len() as i32 * line_h).max(line_h);

        for dy in 0..total_h {
            for dx in 0..total_w {
                buffer.set_pixel_i32(origin.x + dx, origin.y + dy, bg_color);
            }
        }
    }

    match font_family {
        FontFamilyChoice::Pixel8x8 | FontFamilyChoice::Monospace => {
            draw_text_bitmap(
                buffer,
                origin,
                text,
                scale as u32,
                SelectionMode::Transparent,
                text_color,
                bg_color,
            );
        }
        FontFamilyChoice::SansSerif => {
            draw_text_proportional(buffer, origin, text, scale, false, text_color);
        }
        FontFamilyChoice::Serif => {
            draw_text_proportional(buffer, origin, text, scale, true, text_color);
        }
    }
}

/// Draws text using an `ab_glyph` Font with crisp thresholding (XP non-anti-aliased style).
#[allow(clippy::too_many_arguments)]
pub fn draw_text_with_font<F: ab_glyph::Font>(
    buffer: &mut PixelBuffer,
    origin: Point,
    text: &str,
    font: &F,
    px_size: f32,
    mode: SelectionMode,
    text_color: ColorRgba,
    bg_color: ColorRgba,
) {
    use ab_glyph::ScaleFont;
    let scale = ab_glyph::PxScale::from(px_size);
    let scaled_font = font.as_scaled(scale);
    let mut pen_x = origin.x as f32;
    let baseline_y = origin.y as f32 + scaled_font.ascent();
    let line_height = scaled_font.height() + scaled_font.line_gap();
    let mut pen_y = baseline_y;

    if mode == SelectionMode::Opaque {
        // Fill approximate background
        let lines: Vec<&str> = text.split('\n').collect();
        let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(1);
        let bg_w = (max_len as f32 * px_size * 0.6) as i32;
        let bg_h = (lines.len() as f32 * line_height) as i32;
        for dy in 0..bg_h {
            for dx in 0..bg_w {
                buffer.set_pixel_i32(origin.x + dx, origin.y + dy, bg_color);
            }
        }
    }

    for ch in text.chars() {
        if ch == '\n' {
            pen_x = origin.x as f32;
            pen_y += line_height;
            continue;
        }

        let mut glyph = scaled_font.scaled_glyph(ch);
        glyph.position = ab_glyph::point(pen_x, pen_y);
        let advance = scaled_font.h_advance(glyph.id);

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, coverage| {
                if coverage >= 0.5 {
                    let px = bounds.min.x as i32 + x as i32;
                    let py = bounds.min.y as i32 + y as i32;
                    buffer.set_pixel_i32(px, py, text_color);
                }
            });
        }
        pen_x += advance;
    }
}

/// Measures the pixel width and height needed to render `text`.
pub fn measure_text(text: &str, font_family: FontFamilyChoice, font_size: u32) -> (u32, u32) {
    let font_size = font_size.clamp(8, 128);
    let scale = (font_size / 8).max(1) as i32;
    let lines: Vec<&str> = text.split('\n').collect();
    let max_chars = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);
    let char_w = match font_family {
        FontFamilyChoice::Pixel8x8 | FontFamilyChoice::Monospace => 8 * scale,
        FontFamilyChoice::SansSerif | FontFamilyChoice::Serif => 6 * scale,
    };
    let line_h = 10 * scale;
    let total_w = ((max_chars as i32 * char_w).max(char_w) + 4) as u32;
    let total_h = ((lines.len() as i32 * line_h).max(line_h) + 4) as u32;
    (total_w, total_h)
}

/// Renders `text` to an isolated PixelBuffer for preview or rotation.
pub fn render_text_to_buffer(
    text: &str,
    font_family: FontFamilyChoice,
    font_size: u32,
    mode: SelectionMode,
    text_color: ColorRgba,
    bg_color: ColorRgba,
) -> PixelBuffer {
    let (w, h) = measure_text(text, font_family, font_size);
    let bg = if mode == SelectionMode::Opaque {
        bg_color
    } else {
        ColorRgba::TRANSPARENT
    };
    let mut buf = PixelBuffer::new(w, h, bg);
    draw_text(
        &mut buf,
        Point::new(2, 2),
        text,
        font_family,
        font_size,
        mode,
        text_color,
        bg_color,
    );
    buf
}

/// Draws text rotated by 0, 90, 180, or 270 degrees.
#[allow(clippy::too_many_arguments)]
pub fn draw_text_rotated(
    buffer: &mut PixelBuffer,
    origin: Point,
    text: &str,
    font_family: FontFamilyChoice,
    font_size: u32,
    mode: SelectionMode,
    text_color: ColorRgba,
    bg_color: ColorRgba,
    rotation_deg: u16,
) {
    let rotation = rotation_deg % 360;
    if rotation == 0 {
        draw_text(
            buffer,
            origin,
            text,
            font_family,
            font_size,
            mode,
            text_color,
            bg_color,
        );
        return;
    }

    let text_buf = render_text_to_buffer(text, font_family, font_size, mode, text_color, bg_color);
    let rotated = match rotation / 90 {
        1 => crate::transform::rotate_90(&text_buf),
        2 => crate::transform::rotate_180(&text_buf),
        3 => crate::transform::rotate_270(&text_buf),
        _ => text_buf,
    };

    let w = rotated.width() as i32;
    let h = rotated.height() as i32;
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = rotated.get_pixel_i32(x, y) {
                if mode == SelectionMode::Transparent {
                    if p.a > 0 {
                        buffer.blend_pixel_i32(origin.x + x, origin.y + y, p);
                    }
                } else {
                    buffer.set_pixel_i32(origin.x + x, origin.y + y, p);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, RED, WHITE};

    #[test]
    fn test_draw_text() {
        let mut buffer = PixelBuffer::new(40, 20, WHITE);
        draw_text_bitmap(
            &mut buffer,
            Point::new(2, 2),
            "HI",
            1,
            SelectionMode::Opaque,
            BLACK,
            RED,
        );

        // Check that pixels were rendered
        let mut black_count = 0;
        let mut red_count = 0;
        for y in 2..10 {
            for x in 2..18 {
                if buffer.get_pixel_i32(x, y) == Some(BLACK) {
                    black_count += 1;
                } else if buffer.get_pixel_i32(x, y) == Some(RED) {
                    red_count += 1;
                }
            }
        }
        assert!(black_count > 0);
        assert!(red_count > 0);
    }

    #[test]
    fn test_font_families_and_sizes() {
        let mut buffer = PixelBuffer::new(100, 100, WHITE);
        for font in [
            FontFamilyChoice::Pixel8x8,
            FontFamilyChoice::SansSerif,
            FontFamilyChoice::Monospace,
            FontFamilyChoice::Serif,
        ] {
            draw_text(
                &mut buffer,
                Point::new(0, 0),
                "Test",
                font,
                16,
                SelectionMode::Transparent,
                BLACK,
                WHITE,
            );
        }
    }

    #[test]
    fn test_draw_text_rotated() {
        let mut b0 = PixelBuffer::new(60, 60, WHITE);
        let mut b90 = PixelBuffer::new(60, 60, WHITE);
        let mut b180 = PixelBuffer::new(60, 60, WHITE);
        let mut b270 = PixelBuffer::new(60, 60, WHITE);

        draw_text_rotated(
            &mut b0,
            Point::new(10, 10),
            "ROT",
            FontFamilyChoice::Pixel8x8,
            16,
            SelectionMode::Transparent,
            BLACK,
            WHITE,
            0,
        );
        draw_text_rotated(
            &mut b90,
            Point::new(10, 10),
            "ROT",
            FontFamilyChoice::Pixel8x8,
            16,
            SelectionMode::Transparent,
            BLACK,
            WHITE,
            90,
        );
        draw_text_rotated(
            &mut b180,
            Point::new(10, 10),
            "ROT",
            FontFamilyChoice::Pixel8x8,
            16,
            SelectionMode::Transparent,
            BLACK,
            WHITE,
            180,
        );
        draw_text_rotated(
            &mut b270,
            Point::new(10, 10),
            "ROT",
            FontFamilyChoice::Pixel8x8,
            16,
            SelectionMode::Transparent,
            BLACK,
            WHITE,
            270,
        );

        // Verify non-empty
        assert!(b0.as_raw().chunks_exact(4).any(|c| c == [0, 0, 0, 255]));
        assert!(b90.as_raw().chunks_exact(4).any(|c| c == [0, 0, 0, 255]));
        assert!(b180.as_raw().chunks_exact(4).any(|c| c == [0, 0, 0, 255]));
        assert!(b270.as_raw().chunks_exact(4).any(|c| c == [0, 0, 0, 255]));
    }
}

//! Classic 28-color palette of Windows XP Paint (2 rows of 14 swatches).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorRgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

pub const BLACK: ColorRgba = ColorRgba::rgb(0, 0, 0);
pub const WHITE: ColorRgba = ColorRgba::rgb(255, 255, 255);

/// Classic 28-color XP Paint palette
pub const XP_PALETTE_28: [ColorRgba; 28] = [
    // Top Row (14 colors)
    ColorRgba::rgb(0, 0, 0),       // Black
    ColorRgba::rgb(128, 128, 128), // Dark Gray
    ColorRgba::rgb(128, 0, 0),     // Dark Red
    ColorRgba::rgb(128, 128, 0),   // Olive
    ColorRgba::rgb(0, 128, 0),     // Dark Green
    ColorRgba::rgb(0, 128, 128),   // Dark Cyan / Slate
    ColorRgba::rgb(0, 0, 128),     // Dark Blue
    ColorRgba::rgb(128, 0, 128),   // Dark Magenta / Purple
    ColorRgba::rgb(128, 128, 64),  // Dull Yellow
    ColorRgba::rgb(0, 64, 64),     // Dark Teal
    ColorRgba::rgb(0, 128, 255),   // Light Navy
    ColorRgba::rgb(0, 64, 128),    // Midnight Blue
    ColorRgba::rgb(128, 0, 255),   // Violet
    ColorRgba::rgb(128, 64, 0),    // Brown
    // Bottom Row (14 colors)
    ColorRgba::rgb(255, 255, 255), // White
    ColorRgba::rgb(192, 192, 192), // Light Gray
    ColorRgba::rgb(255, 0, 0),     // Red
    ColorRgba::rgb(255, 255, 0),   // Yellow
    ColorRgba::rgb(0, 255, 0),     // Lime Green
    ColorRgba::rgb(0, 255, 255),   // Cyan
    ColorRgba::rgb(0, 0, 255),     // Blue
    ColorRgba::rgb(255, 0, 255),   // Magenta
    ColorRgba::rgb(255, 255, 128), // Light Yellow
    ColorRgba::rgb(0, 255, 128),   // Light Green
    ColorRgba::rgb(128, 255, 255), // Light Cyan
    ColorRgba::rgb(128, 128, 255), // Sky Blue
    ColorRgba::rgb(255, 0, 128),   // Bright Pink
    ColorRgba::rgb(255, 128, 64),  // Orange
];

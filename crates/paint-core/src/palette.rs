//! Classic 28-color palette of Windows XP Paint (2 rows of 14 swatches)
//! and 48 basic colors for the Edit Colors dialog.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorRgba {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

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

pub const TRANSPARENT: ColorRgba = ColorRgba::rgba(0, 0, 0, 0);
pub const BLACK: ColorRgba = ColorRgba::rgb(0, 0, 0);
pub const WHITE: ColorRgba = ColorRgba::rgb(255, 255, 255);
pub const RED: ColorRgba = ColorRgba::rgb(255, 0, 0);
pub const GREEN: ColorRgba = ColorRgba::rgb(0, 255, 0);
pub const BLUE: ColorRgba = ColorRgba::rgb(0, 0, 255);
pub const YELLOW: ColorRgba = ColorRgba::rgb(255, 255, 0);
pub const CYAN: ColorRgba = ColorRgba::rgb(0, 255, 255);
pub const MAGENTA: ColorRgba = ColorRgba::rgb(255, 0, 255);
pub const DARK_RED: ColorRgba = ColorRgba::rgb(128, 0, 0);
pub const DARK_GREEN: ColorRgba = ColorRgba::rgb(0, 128, 0);
pub const DARK_BLUE: ColorRgba = ColorRgba::rgb(0, 0, 128);
pub const PURPLE: ColorRgba = ColorRgba::rgb(128, 0, 128);
pub const LIGHT_RED: ColorRgba = ColorRgba::rgb(255, 128, 128);
pub const LIGHT_GREEN: ColorRgba = ColorRgba::rgb(0, 255, 128);
pub const LIGHT_BLUE: ColorRgba = ColorRgba::rgb(128, 128, 255);
pub const LIGHT_YELLOW: ColorRgba = ColorRgba::rgb(255, 255, 128);

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

/// Apple II authentic 16-color palette expanded to 28 slots
pub const APPLE_II_PALETTE_28: [ColorRgba; 28] = [
    // Top row: darks / bases
    ColorRgba::rgb(0, 0, 0),       // 0: Black
    ColorRgba::rgb(114, 38, 64),   // 1: Deep Red
    ColorRgba::rgb(65, 52, 142),   // 2: Dark Blue
    ColorRgba::rgb(180, 77, 207),  // 3: Purple
    ColorRgba::rgb(0, 105, 65),    // 4: Dark Green
    ColorRgba::rgb(128, 128, 128), // 5: Gray 1
    ColorRgba::rgb(13, 147, 221),  // 6: Medium Blue
    ColorRgba::rgb(146, 178, 245), // 7: Light Blue
    ColorRgba::rgb(90, 70, 0),     // Extra shade: Dark Olive / Earth
    ColorRgba::rgb(35, 35, 35),    // Extra shade: Dark Charcoal
    ColorRgba::rgb(70, 15, 25),    // Extra shade: Maroon
    ColorRgba::rgb(20, 50, 90),    // Extra shade: Navy
    ColorRgba::rgb(0, 50, 30),     // Extra shade: Forest
    ColorRgba::rgb(80, 80, 80),    // Extra shade: Medium Dark Gray
    // Bottom row: lights / brights
    ColorRgba::rgb(136, 114, 0),   // 8: Brown
    ColorRgba::rgb(229, 102, 34),  // 9: Orange
    ColorRgba::rgb(192, 192, 192), // 10: Gray 2
    ColorRgba::rgb(241, 149, 169), // 11: Pink
    ColorRgba::rgb(85, 203, 33),   // 12: Light Green
    ColorRgba::rgb(230, 230, 80),  // 13: Yellow
    ColorRgba::rgb(141, 222, 174), // 14: Aquamarine
    ColorRgba::rgb(255, 255, 255), // 15: White
    ColorRgba::rgb(255, 200, 150), // Extra shade: Peach
    ColorRgba::rgb(255, 170, 100), // Extra shade: Warm Gold
    ColorRgba::rgb(180, 240, 130), // Extra shade: Mint
    ColorRgba::rgb(190, 220, 255), // Extra shade: Pale Cyan
    ColorRgba::rgb(230, 190, 255), // Extra shade: Lavender
    ColorRgba::rgb(220, 220, 220), // Extra shade: Silver
];

/// Commodore 64 16-color palette expanded to 28 slots
pub const COMMODORE_64_PALETTE_28: [ColorRgba; 28] = [
    // Top Row
    ColorRgba::rgb(0, 0, 0),       // 0: Black
    ColorRgba::rgb(136, 0, 0),     // 2: Red
    ColorRgba::rgb(170, 68, 187),  // 4: Purple
    ColorRgba::rgb(0, 136, 85),    // 5: Green
    ColorRgba::rgb(0, 0, 170),     // 6: Blue
    ColorRgba::rgb(102, 68, 0),    // 9: Brown
    ColorRgba::rgb(51, 51, 51),    // Extra shade: Very Dark Gray
    ColorRgba::rgb(68, 68, 68),    // 11: Dark Gray
    ColorRgba::rgb(119, 119, 119), // 12: Medium Gray
    ColorRgba::rgb(0, 68, 102),    // Extra shade: Deep Cyan
    ColorRgba::rgb(85, 0, 85),     // Extra shade: Deep Magenta
    ColorRgba::rgb(0, 85, 0),      // Extra shade: Forest
    ColorRgba::rgb(85, 34, 0),     // Extra shade: Dark Amber
    ColorRgba::rgb(20, 20, 60),    // Extra shade: Midnight
    // Bottom Row
    ColorRgba::rgb(255, 255, 255), // 1: White
    ColorRgba::rgb(170, 255, 238), // 3: Cyan
    ColorRgba::rgb(238, 238, 119), // 7: Yellow
    ColorRgba::rgb(221, 136, 85),  // 8: Orange
    ColorRgba::rgb(255, 119, 119), // 10: Light Red
    ColorRgba::rgb(170, 255, 102), // 13: Light Green
    ColorRgba::rgb(0, 136, 255),   // 14: Light Blue
    ColorRgba::rgb(187, 187, 187), // 15: Light Gray
    ColorRgba::rgb(255, 204, 153), // Extra shade: Cream
    ColorRgba::rgb(255, 170, 204), // Extra shade: Pastel Rose
    ColorRgba::rgb(204, 153, 255), // Extra shade: Lilac
    ColorRgba::rgb(153, 255, 204), // Extra shade: Pale Seafoam
    ColorRgba::rgb(255, 255, 187), // Extra shade: Pale Yellow
    ColorRgba::rgb(204, 238, 255), // Extra shade: Ice Blue
];

/// IBM PC CGA (Color Graphics Adapter) 16-color palette expanded to 28 slots
pub const CGA_PALETTE_28: [ColorRgba; 28] = [
    // Top Row (Darker 8 colors + variations)
    ColorRgba::rgb(0, 0, 0),       // 0: Black
    ColorRgba::rgb(0, 0, 170),     // 1: Blue
    ColorRgba::rgb(0, 170, 0),     // 2: Green
    ColorRgba::rgb(0, 170, 170),   // 3: Cyan
    ColorRgba::rgb(170, 0, 0),     // 4: Red
    ColorRgba::rgb(170, 0, 170),   // 5: Magenta
    ColorRgba::rgb(170, 85, 0),    // 6: Brown
    ColorRgba::rgb(170, 170, 170), // 7: Light Gray
    ColorRgba::rgb(85, 85, 85),    // 8: Dark Gray
    ColorRgba::rgb(0, 85, 85),     // Low intensity Teal
    ColorRgba::rgb(85, 0, 85),     // Low intensity Purple
    ColorRgba::rgb(85, 85, 0),     // Low intensity Olive
    ColorRgba::rgb(0, 0, 85),      // Low intensity Navy
    ColorRgba::rgb(85, 0, 0),      // Low intensity Maroon
    // Bottom Row (High intensity colors)
    ColorRgba::rgb(85, 85, 255),   // 9: Bright Blue
    ColorRgba::rgb(85, 255, 85),   // 10: Bright Green
    ColorRgba::rgb(85, 255, 255),  // 11: Bright Cyan
    ColorRgba::rgb(255, 85, 85),   // 12: Bright Red
    ColorRgba::rgb(255, 85, 255),  // 13: Bright Magenta
    ColorRgba::rgb(255, 255, 85),  // 14: Bright Yellow
    ColorRgba::rgb(255, 255, 255), // 15: Bright White
    ColorRgba::rgb(255, 170, 85),  // Amber orange
    ColorRgba::rgb(170, 255, 170), // Soft Green
    ColorRgba::rgb(170, 255, 255), // Soft Cyan
    ColorRgba::rgb(255, 170, 255), // Soft Pink
    ColorRgba::rgb(255, 255, 170), // Soft Yellow
    ColorRgba::rgb(200, 200, 200), // Silver
    ColorRgba::rgb(128, 128, 128), // Gray
];

/// Nintendo Game Boy (DMG-01) classic 4-shade greenish palette with variations
pub const GAME_BOY_PALETTE_28: [ColorRgba; 28] = [
    // Top Row (Dark tones)
    ColorRgba::rgb(15, 56, 15),
    ColorRgba::rgb(20, 65, 20),
    ColorRgba::rgb(25, 75, 25),
    ColorRgba::rgb(30, 85, 30),
    ColorRgba::rgb(35, 95, 35),
    ColorRgba::rgb(40, 105, 40),
    ColorRgba::rgb(48, 98, 48), // DMG dark shade 2
    ColorRgba::rgb(55, 115, 55),
    ColorRgba::rgb(60, 125, 60),
    ColorRgba::rgb(70, 135, 70),
    ColorRgba::rgb(80, 145, 80),
    ColorRgba::rgb(90, 155, 90),
    ColorRgba::rgb(100, 160, 100),
    ColorRgba::rgb(110, 165, 110),
    // Bottom Row (Mid to Bright tones)
    ColorRgba::rgb(139, 172, 15), // DMG mid-light shade 3
    ColorRgba::rgb(145, 178, 25),
    ColorRgba::rgb(150, 185, 35),
    ColorRgba::rgb(155, 190, 45),
    ColorRgba::rgb(160, 195, 55),
    ColorRgba::rgb(165, 200, 65),
    ColorRgba::rgb(170, 205, 75),
    ColorRgba::rgb(175, 210, 85),
    ColorRgba::rgb(180, 215, 95),
    ColorRgba::rgb(185, 220, 105),
    ColorRgba::rgb(155, 188, 15), // DMG light
    ColorRgba::rgb(165, 198, 25),
    ColorRgba::rgb(175, 208, 35),
    ColorRgba::rgb(202, 220, 159), // DMG brightest highlight
];

/// ZX Spectrum 16-color palette (8 normal + 8 bright) expanded to 28 slots
pub const ZX_SPECTRUM_PALETTE_28: [ColorRgba; 28] = [
    // Top Row: Standard intensity
    ColorRgba::rgb(0, 0, 0),       // Black
    ColorRgba::rgb(0, 0, 215),     // Blue
    ColorRgba::rgb(215, 0, 0),     // Red
    ColorRgba::rgb(215, 0, 215),   // Magenta
    ColorRgba::rgb(0, 215, 0),     // Green
    ColorRgba::rgb(0, 215, 215),   // Cyan
    ColorRgba::rgb(215, 215, 0),   // Yellow
    ColorRgba::rgb(215, 215, 215), // White
    ColorRgba::rgb(100, 0, 0),     // Dark Red
    ColorRgba::rgb(0, 100, 0),     // Dark Green
    ColorRgba::rgb(0, 0, 100),     // Dark Blue
    ColorRgba::rgb(100, 100, 0),   // Dark Yellow / Olive
    ColorRgba::rgb(100, 0, 100),   // Dark Magenta
    ColorRgba::rgb(0, 100, 100),   // Dark Cyan
    // Bottom Row: Bright intensity
    ColorRgba::rgb(0, 0, 255),     // Bright Blue
    ColorRgba::rgb(255, 0, 0),     // Bright Red
    ColorRgba::rgb(255, 0, 255),   // Bright Magenta
    ColorRgba::rgb(0, 255, 0),     // Bright Green
    ColorRgba::rgb(0, 255, 255),   // Bright Cyan
    ColorRgba::rgb(255, 255, 0),   // Bright Yellow
    ColorRgba::rgb(255, 255, 255), // Bright White
    ColorRgba::rgb(255, 128, 0),   // Bright Orange
    ColorRgba::rgb(128, 255, 0),   // Bright Chartreuse
    ColorRgba::rgb(0, 255, 128),   // Bright Spring Green
    ColorRgba::rgb(0, 128, 255),   // Bright Dodger Blue
    ColorRgba::rgb(128, 0, 255),   // Bright Electric Indigo
    ColorRgba::rgb(255, 0, 128),   // Bright Deep Pink
    ColorRgba::rgb(180, 180, 180), // Bright Gray
];

/// PICO-8 fantasy console 16-color palette expanded with secret colors to 28 slots
pub const PICO_8_PALETTE_28: [ColorRgba; 28] = [
    // Top Row: Standard darks and midtones
    ColorRgba::rgb(0, 0, 0),       // 0: Black
    ColorRgba::rgb(29, 43, 83),    // 1: Dark Blue
    ColorRgba::rgb(126, 37, 83),   // 2: Dark Purple
    ColorRgba::rgb(0, 135, 81),    // 3: Dark Green
    ColorRgba::rgb(171, 82, 54),   // 4: Brown
    ColorRgba::rgb(95, 87, 79),    // 5: Dark Gray
    ColorRgba::rgb(194, 195, 199), // 6: Light Gray
    ColorRgba::rgb(255, 241, 232), // 7: White
    ColorRgba::rgb(130, 28, 64),   // Secret: Deep Red
    ColorRgba::rgb(41, 173, 255),  // 12: Blue
    ColorRgba::rgb(131, 118, 156), // 13: Indigo
    ColorRgba::rgb(255, 119, 168), // 14: Pink
    ColorRgba::rgb(255, 204, 170), // 15: Peach
    ColorRgba::rgb(40, 53, 64),    // Secret: Charcoal
    // Bottom Row: Standard vibrant colors & extensions
    ColorRgba::rgb(255, 0, 77),    // 8: Red
    ColorRgba::rgb(255, 163, 0),   // 9: Orange
    ColorRgba::rgb(255, 236, 39),  // 10: Yellow
    ColorRgba::rgb(0, 228, 54),    // 11: Green
    ColorRgba::rgb(255, 108, 36),  // Secret: Tangerine
    ColorRgba::rgb(255, 231, 125), // Secret: Lemon
    ColorRgba::rgb(0, 181, 67),    // Secret: Emerald
    ColorRgba::rgb(24, 130, 177),  // Secret: Sea
    ColorRgba::rgb(117, 70, 101),  // Secret: Plum
    ColorRgba::rgb(218, 95, 142),  // Secret: Rose
    ColorRgba::rgb(247, 190, 160), // Secret: Sand
    ColorRgba::rgb(116, 121, 129), // Secret: Slate
    ColorRgba::rgb(162, 136, 121), // Secret: Taupe
    ColorRgba::rgb(255, 255, 255), // Pure White
];

/// Nintendo Entertainment System (NES / Famicom) curated 28-color palette
pub const NES_PALETTE_28: [ColorRgba; 28] = [
    // Top Row: Deep & mid tones
    ColorRgba::rgb(0, 0, 0),       // Black
    ColorRgba::rgb(0, 30, 116),    // Dark Blue
    ColorRgba::rgb(8, 16, 144),    // Navy
    ColorRgba::rgb(48, 0, 136),    // Dark Indigo
    ColorRgba::rgb(68, 0, 100),    // Dark Violet
    ColorRgba::rgb(92, 0, 48),     // Deep Maroon
    ColorRgba::rgb(84, 4, 0),      // Dark Red
    ColorRgba::rgb(60, 20, 0),     // Dark Brown
    ColorRgba::rgb(0, 48, 0),      // Deep Forest
    ColorRgba::rgb(0, 52, 0),      // Dark Olive
    ColorRgba::rgb(0, 60, 0),      // Dark Green
    ColorRgba::rgb(0, 50, 60),     // Dark Cyan
    ColorRgba::rgb(0, 0, 0),       // Black alt
    ColorRgba::rgb(116, 116, 116), // Medium Gray
    // Bottom Row: Bright & pastel tones
    ColorRgba::rgb(252, 252, 252), // White
    ColorRgba::rgb(60, 188, 252),  // Sky Blue
    ColorRgba::rgb(89, 148, 252),  // Light Blue
    ColorRgba::rgb(152, 120, 248), // Lavender
    ColorRgba::rgb(248, 120, 248), // Pink
    ColorRgba::rgb(248, 88, 152),  // Rose
    ColorRgba::rgb(248, 120, 88),  // Salmon / Coral
    ColorRgba::rgb(252, 160, 68),  // Light Orange
    ColorRgba::rgb(248, 184, 0),   // Gold / Yellow
    ColorRgba::rgb(184, 248, 24),  // Lime
    ColorRgba::rgb(88, 216, 84),   // Green
    ColorRgba::rgb(0, 232, 216),   // Cyan / Turquoise
    ColorRgba::rgb(188, 188, 188), // Light Gray
    ColorRgba::rgb(248, 216, 120), // Light Sand
];

/// Commodore Amiga Workbench classic 16-color palette expanded to 28 slots
pub const AMIGA_PALETTE_28: [ColorRgba; 28] = [
    // Top Row
    ColorRgba::rgb(0, 85, 170),    // Workbench Blue
    ColorRgba::rgb(255, 255, 255), // White
    ColorRgba::rgb(0, 0, 0),       // Black
    ColorRgba::rgb(255, 136, 0),   // Orange
    ColorRgba::rgb(0, 34, 102),    // Dark Blue
    ColorRgba::rgb(170, 0, 0),     // Amiga Red
    ColorRgba::rgb(0, 170, 0),     // Amiga Green
    ColorRgba::rgb(170, 170, 0),   // Amiga Yellow
    ColorRgba::rgb(85, 85, 85),    // Gray 1
    ColorRgba::rgb(119, 119, 119), // Gray 2
    ColorRgba::rgb(153, 153, 153), // Gray 3
    ColorRgba::rgb(187, 187, 187), // Gray 4
    ColorRgba::rgb(102, 51, 0),    // Brown
    ColorRgba::rgb(85, 0, 170),    // Purple
    // Bottom Row
    ColorRgba::rgb(0, 170, 170),   // Cyan
    ColorRgba::rgb(255, 0, 255),   // Magenta
    ColorRgba::rgb(255, 255, 0),   // Bright Yellow
    ColorRgba::rgb(0, 255, 0),     // Bright Green
    ColorRgba::rgb(0, 255, 255),   // Bright Cyan
    ColorRgba::rgb(0, 136, 255),   // Bright Sky
    ColorRgba::rgb(255, 102, 102), // Light Red
    ColorRgba::rgb(255, 204, 0),   // Amber
    ColorRgba::rgb(204, 255, 102), // Pale Lime
    ColorRgba::rgb(102, 255, 204), // Aquamarine
    ColorRgba::rgb(204, 204, 255), // Periwinkle
    ColorRgba::rgb(255, 204, 255), // Pink Rose
    ColorRgba::rgb(221, 221, 221), // Light Silver
    ColorRgba::rgb(240, 240, 240), // Off White
];

/// Presets for classic and retro computer palettes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetroPalettePreset {
    WindowsXp,
    AppleII,
    Commodore64,
    Cga,
    GameBoy,
    ZxSpectrum,
    Pico8,
    Nes,
    Amiga,
}

impl RetroPalettePreset {
    pub const ALL: [RetroPalettePreset; 9] = [
        RetroPalettePreset::WindowsXp,
        RetroPalettePreset::AppleII,
        RetroPalettePreset::Commodore64,
        RetroPalettePreset::Cga,
        RetroPalettePreset::GameBoy,
        RetroPalettePreset::ZxSpectrum,
        RetroPalettePreset::Pico8,
        RetroPalettePreset::Nes,
        RetroPalettePreset::Amiga,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::WindowsXp => "Windows XP (Classic)",
            Self::AppleII => "Apple II",
            Self::Commodore64 => "Commodore 64",
            Self::Cga => "IBM PC CGA",
            Self::GameBoy => "Game Boy (DMG)",
            Self::ZxSpectrum => "ZX Spectrum",
            Self::Pico8 => "PICO-8",
            Self::Nes => "NES",
            Self::Amiga => "Amiga Workbench",
        }
    }

    pub fn swatches(&self) -> [ColorRgba; 28] {
        match self {
            Self::WindowsXp => XP_PALETTE_28,
            Self::AppleII => APPLE_II_PALETTE_28,
            Self::Commodore64 => COMMODORE_64_PALETTE_28,
            Self::Cga => CGA_PALETTE_28,
            Self::GameBoy => GAME_BOY_PALETTE_28,
            Self::ZxSpectrum => ZX_SPECTRUM_PALETTE_28,
            Self::Pico8 => PICO_8_PALETTE_28,
            Self::Nes => NES_PALETTE_28,
            Self::Amiga => AMIGA_PALETTE_28,
        }
    }
}

/// 48 Basic Colors in Windows XP Edit Colors Dialog (6 rows of 8 columns)
pub const XP_BASIC_COLORS_48: [ColorRgba; 48] = [
    // Row 1
    ColorRgba::rgb(255, 128, 128),
    ColorRgba::rgb(255, 255, 128),
    ColorRgba::rgb(128, 255, 128),
    ColorRgba::rgb(0, 255, 128),
    ColorRgba::rgb(128, 255, 255),
    ColorRgba::rgb(0, 128, 255),
    ColorRgba::rgb(255, 128, 192),
    ColorRgba::rgb(255, 128, 255),
    // Row 2
    ColorRgba::rgb(255, 0, 0),
    ColorRgba::rgb(255, 255, 0),
    ColorRgba::rgb(128, 255, 0),
    ColorRgba::rgb(0, 255, 64),
    ColorRgba::rgb(0, 255, 255),
    ColorRgba::rgb(0, 128, 192),
    ColorRgba::rgb(128, 128, 192),
    ColorRgba::rgb(255, 0, 255),
    // Row 3
    ColorRgba::rgb(128, 64, 64),
    ColorRgba::rgb(255, 128, 64),
    ColorRgba::rgb(0, 128, 0),
    ColorRgba::rgb(0, 128, 64),
    ColorRgba::rgb(0, 64, 128),
    ColorRgba::rgb(128, 128, 255),
    ColorRgba::rgb(128, 0, 64),
    ColorRgba::rgb(255, 0, 128),
    // Row 4
    ColorRgba::rgb(128, 0, 0),
    ColorRgba::rgb(255, 128, 0),
    ColorRgba::rgb(0, 64, 0),
    ColorRgba::rgb(0, 64, 64),
    ColorRgba::rgb(0, 0, 255),
    ColorRgba::rgb(0, 0, 128),
    ColorRgba::rgb(128, 0, 128),
    ColorRgba::rgb(128, 0, 255),
    // Row 5
    ColorRgba::rgb(64, 0, 0),
    ColorRgba::rgb(128, 64, 0),
    ColorRgba::rgb(0, 32, 0),
    ColorRgba::rgb(0, 32, 32),
    ColorRgba::rgb(0, 0, 128),
    ColorRgba::rgb(0, 0, 64),
    ColorRgba::rgb(64, 0, 64),
    ColorRgba::rgb(64, 0, 128),
    // Row 6
    ColorRgba::rgb(0, 0, 0),
    ColorRgba::rgb(128, 128, 128),
    ColorRgba::rgb(192, 192, 192),
    ColorRgba::rgb(255, 255, 255),
    ColorRgba::rgb(128, 128, 64),
    ColorRgba::rgb(128, 128, 0),
    ColorRgba::rgb(64, 64, 64),
    ColorRgba::rgb(192, 192, 192),
];

/// Converts RGB (0..=255) to HSL (Hue: 0..=240, Sat: 0..=240, Lum: 0..=240) in Windows XP style.
pub fn rgb_to_hsl_win(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let mut h = 0.0_f32;
    let mut s = 0.0_f32;
    let l = (max + min) / 2.0;

    if delta > 0.0001 {
        s = if l <= 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        if (max - rf).abs() < 0.0001 {
            h = (gf - bf) / delta + (if gf < bf { 6.0 } else { 0.0 });
        } else if (max - gf).abs() < 0.0001 {
            h = (bf - rf) / delta + 2.0;
        } else {
            h = (rf - gf) / delta + 4.0;
        }
        h /= 6.0;
    }

    let win_h = (h * 240.0).round().clamp(0.0, 240.0) as u8;
    let win_s = (s * 240.0).round().clamp(0.0, 240.0) as u8;
    let win_l = (l * 240.0).round().clamp(0.0, 240.0) as u8;

    (win_h, win_s, win_l)
}

/// Converts Windows HSL (Hue: 0..=240, Sat: 0..=240, Lum: 0..=240) to RGB (0..=255).
pub fn hsl_win_to_rgb(h: u8, s: u8, l: u8) -> (u8, u8, u8) {
    let hf = (h as f32 / 240.0).clamp(0.0, 1.0);
    let sf = (s as f32 / 240.0).clamp(0.0, 1.0);
    let lf = (l as f32 / 240.0).clamp(0.0, 1.0);

    if sf <= 0.0001 {
        let v = (lf * 255.0).round().clamp(0.0, 255.0) as u8;
        return (v, v, v);
    }

    let q = if lf < 0.5 {
        lf * (1.0 + sf)
    } else {
        lf + sf - lf * sf
    };
    let p = 2.0 * lf - q;

    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    let r = hue_to_rgb(p, q, hf + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, hf);
    let b = hue_to_rgb(p, q, hf - 1.0 / 3.0);

    (
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsl_conversions() {
        // Pure red
        let (h, s, l) = rgb_to_hsl_win(255, 0, 0);
        assert_eq!((h, s, l), (0, 240, 120));
        let (r, g, b) = hsl_win_to_rgb(h, s, l);
        assert_eq!((r, g, b), (255, 0, 0));

        // White
        let (_h, s, l) = rgb_to_hsl_win(255, 255, 255);
        assert_eq!((s, l), (0, 240));
        let (r, g, b) = hsl_win_to_rgb(0, 0, 240);
        assert_eq!((r, g, b), (255, 255, 255));

        // Black
        let (_h, s, l) = rgb_to_hsl_win(0, 0, 0);
        assert_eq!((s, l), (0, 0));
        let (r, g, b) = hsl_win_to_rgb(0, 0, 0);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_retro_palettes() {
        assert_eq!(RetroPalettePreset::ALL.len(), 9);
        for preset in RetroPalettePreset::ALL {
            assert!(!preset.name().is_empty());
            let swatches = preset.swatches();
            assert_eq!(swatches.len(), 28);
            // Verify all swatches have alpha 255
            for color in swatches {
                assert_eq!(color.a, 255);
            }
        }

        // Apple II specifically has expected iconic colors
        let apple = RetroPalettePreset::AppleII.swatches();
        assert_eq!(apple[0], ColorRgba::rgb(0, 0, 0)); // Black
        assert_eq!(apple[1], ColorRgba::rgb(114, 38, 64)); // Deep Red
        assert_eq!(apple[21], ColorRgba::rgb(255, 255, 255)); // White
    }
}

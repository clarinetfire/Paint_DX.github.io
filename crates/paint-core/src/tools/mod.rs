//! The 16 classic tools of Windows XP Paint and their common interfaces.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    FreeformSelect,
    Select,
    Eraser,
    Fill,
    PickColor,
    Magnifier,
    Pencil,
    Brush,
    Airbrush,
    Text,
    Line,
    Curve,
    Rectangle,
    Polygon,
    Ellipse,
    RoundedRectangle,
    Move,
    // Kid Pix creative tools
    WackyBrush,
    Stamp,
    PatternFill,
    ElectricMixer,
}

impl ToolKind {
    pub const ALL: [ToolKind; 21] = [
        ToolKind::FreeformSelect,
        ToolKind::Select,
        ToolKind::Eraser,
        ToolKind::Fill,
        ToolKind::PickColor,
        ToolKind::Magnifier,
        ToolKind::Pencil,
        ToolKind::Brush,
        ToolKind::Airbrush,
        ToolKind::Text,
        ToolKind::Line,
        ToolKind::Curve,
        ToolKind::Rectangle,
        ToolKind::Polygon,
        ToolKind::Ellipse,
        ToolKind::RoundedRectangle,
        ToolKind::Move,
        ToolKind::WackyBrush,
        ToolKind::Stamp,
        ToolKind::PatternFill,
        ToolKind::ElectricMixer,
    ];

    pub const CLASSIC_TOOLS: [ToolKind; 17] = [
        ToolKind::FreeformSelect,
        ToolKind::Select,
        ToolKind::Move,
        ToolKind::Eraser,
        ToolKind::Fill,
        ToolKind::PickColor,
        ToolKind::Magnifier,
        ToolKind::Pencil,
        ToolKind::Brush,
        ToolKind::Airbrush,
        ToolKind::Text,
        ToolKind::Line,
        ToolKind::Curve,
        ToolKind::Rectangle,
        ToolKind::Polygon,
        ToolKind::Ellipse,
        ToolKind::RoundedRectangle,
    ];

    pub const KIDPIX_TOOLS: [ToolKind; 5] = [
        ToolKind::WackyBrush,
        ToolKind::Stamp,
        ToolKind::PatternFill,
        ToolKind::ElectricMixer,
        ToolKind::Eraser,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            ToolKind::FreeformSelect => "Free-Form Select",
            ToolKind::Select => "Select",
            ToolKind::Eraser => "Eraser / Color Eraser",
            ToolKind::Fill => "Fill With Color",
            ToolKind::PickColor => "Pick Color",
            ToolKind::Magnifier => "Magnifier",
            ToolKind::Pencil => "Pencil",
            ToolKind::Brush => "Brush",
            ToolKind::Airbrush => "Airbrush",
            ToolKind::Text => "Text",
            ToolKind::Line => "Line",
            ToolKind::Curve => "Curve",
            ToolKind::Rectangle => "Rectangle",
            ToolKind::Polygon => "Polygon",
            ToolKind::Ellipse => "Ellipse",
            ToolKind::RoundedRectangle => "Rounded Rectangle",
            ToolKind::Move => "Move / Hand",
            ToolKind::WackyBrush => "Wacky Brush",
            ToolKind::Stamp => "Rubber Stamp",
            ToolKind::PatternFill => "Pattern Fill",
            ToolKind::ElectricMixer => "Electric Mixer",
        }
    }
}

pub mod airbrush;
pub mod brush;
pub mod eraser;
pub mod fill;
pub mod line;
pub mod mixer;
pub mod pencil;
pub mod selection;
pub mod shapes;
pub mod text;
pub mod wacky;

pub use airbrush::AirbrushSize;
pub use brush::BrushShape;
pub use eraser::EraserSize;
pub use mixer::{
    apply_mixer_effect, apply_mixer_effect_with_options, ColoringBookParams, MixerEffect,
    MixerOptions, MixerParams, MixerScope, MixerTray, NewspaperParams, SoftenParams, WaveParams,
};
pub use selection::SelectionMode;
pub use shapes::FillStyle;
pub use text::{
    draw_text, draw_text_bitmap, draw_text_rotated, measure_text, render_text_to_buffer,
    FontFamilyChoice,
};
pub use wacky::{
    apply_stamp, apply_stamp_rotated, apply_stamp_with_options, apply_wacky_brush_point,
    apply_wacky_brush_point_with_options, draw_pixel_heart, draw_wacky_brush_stroke,
    draw_wacky_brush_stroke_with_options, pattern_flood_fill, pattern_flood_fill_with_options,
    BrushAngleMode, BrushSpacing, FillPattern, PatternFillOptions, SeedMode, StampOptions,
    StampShape, StampSpacing, StampTint, WackyBrushOptions, WackyBrushShape, WackyBrushState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineOptions {
    pub width: u32,
}

impl Default for LineOptions {
    fn default() -> Self {
        Self { width: 1 }
    }
}

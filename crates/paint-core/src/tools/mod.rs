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
}

impl ToolKind {
    pub const ALL: [ToolKind; 16] = [
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
        }
    }
}

//! Sprite Sheet export settings and layout models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SheetLayout {
    #[default]
    Grid,
    HorizontalStrip,
    VerticalStrip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteSheetOptions {
    pub layout: SheetLayout,
    pub columns: u32,
    pub spacing: u32,
    pub border: u32,
    pub scale: u32,
}

impl Default for SpriteSheetOptions {
    fn default() -> Self {
        Self {
            layout: SheetLayout::Grid,
            columns: 4,
            spacing: 0,
            border: 0,
            scale: 1,
        }
    }
}

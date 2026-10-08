//! Frame and Cel animation data models.

use crate::image::PixelBuffer;
use crate::layers::LayerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cel {
    pub x: i32,
    pub y: i32,
    pub buffer: PixelBuffer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub duration_ms: u32,
    pub cels: HashMap<LayerId, Cel>,
}

impl Frame {
    pub fn new(duration_ms: u32) -> Self {
        Self {
            duration_ms,
            cels: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimTag {
    pub name: String,
    pub from_frame: usize,
    pub to_frame: usize,
}

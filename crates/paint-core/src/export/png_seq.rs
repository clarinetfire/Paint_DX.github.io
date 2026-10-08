//! PNG Sequence export settings and pipeline.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PngSequenceExportOptions {
    pub pattern: String,
    pub start_index: u32,
    pub padding_digits: usize,
    pub scale: u32,
}

impl Default for PngSequenceExportOptions {
    fn default() -> Self {
        Self {
            pattern: "frame_".to_string(),
            start_index: 0,
            padding_digits: 4,
            scale: 1,
        }
    }
}

//! Project serialization and .paintproj manifest structure.

use serde::{Deserialize, Serialize};

pub const PROJECT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        Self {
            version: PROJECT_VERSION,
            width: 800,
            height: 600,
            fps: 12,
        }
    }
}

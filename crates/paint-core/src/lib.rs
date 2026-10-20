//! paint-core: Core data models, pixel manipulations, and business logic for Paint DX.
//!
//! This crate contains pure logic: no UI code, no direct OS calls, and no unsafe code.

#![forbid(unsafe_code)]

pub mod export;
pub mod frames;
pub mod geometry;
pub mod history;
pub mod image;
pub mod io;
pub mod layers;
pub mod onion_skin;
pub mod palette;
pub mod project;
pub mod tools;
pub mod transform;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_initialization() {
        assert_eq!(palette::XP_PALETTE_28.len(), 28);
        assert_eq!(palette::XP_BASIC_COLORS_48.len(), 48);
    }
}

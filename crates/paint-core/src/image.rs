//! RGBA pixel buffer operations and blitting.

use crate::palette::ColorRgba;
use image::RgbaImage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelBuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>, // RGBA format, length = width * height * 4
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32, fill: ColorRgba) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        let mut pixels = Vec::with_capacity(size);
        for _ in 0..(width * height) {
            pixels.extend_from_slice(&fill.to_array());
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn as_raw(&self) -> &[u8] {
        &self.pixels
    }

    pub fn as_raw_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<ColorRgba> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        Some(ColorRgba::rgba(
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ))
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: ColorRgba) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.pixels[idx] = color.r;
        self.pixels[idx + 1] = color.g;
        self.pixels[idx + 2] = color.b;
        self.pixels[idx + 3] = color.a;
    }

    pub fn clear(&mut self, color: ColorRgba) {
        let arr = color.to_array();
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&arr);
        }
    }

    pub fn to_rgba_image(&self) -> Option<RgbaImage> {
        RgbaImage::from_raw(self.width, self.height, self.pixels.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_buffer_creation_and_modification() {
        let mut buf = PixelBuffer::new(10, 10, ColorRgba::rgb(255, 255, 255));
        assert_eq!(buf.width(), 10);
        assert_eq!(buf.height(), 10);
        assert_eq!(buf.get_pixel(0, 0), Some(ColorRgba::rgb(255, 255, 255)));

        let red = ColorRgba::rgb(255, 0, 0);
        buf.set_pixel(5, 5, red);
        assert_eq!(buf.get_pixel(5, 5), Some(red));
        assert_eq!(buf.get_pixel(0, 0), Some(ColorRgba::rgb(255, 255, 255)));
        assert_eq!(buf.get_pixel(10, 10), None);
    }

    #[test]
    fn test_pixel_buffer_clear() {
        let mut buf = PixelBuffer::new(5, 5, ColorRgba::rgb(0, 0, 0));
        let blue = ColorRgba::rgb(0, 0, 255);
        buf.clear(blue);
        assert_eq!(buf.get_pixel(2, 2), Some(blue));
    }
}

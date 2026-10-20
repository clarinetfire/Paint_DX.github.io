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

    pub fn replace_with(&mut self, width: u32, height: u32, pixels: Vec<u8>) {
        self.width = width;
        self.height = height;
        self.pixels = pixels;
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

    pub fn get_pixel_i32(&self, x: i32, y: i32) -> Option<ColorRgba> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        self.get_pixel(x as u32, y as u32)
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

    pub fn set_pixel_i32(&mut self, x: i32, y: i32, color: ColorRgba) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.set_pixel(x as u32, y as u32, color);
    }

    pub fn blend_pixel(&mut self, x: u32, y: u32, src: ColorRgba) {
        if x >= self.width || y >= self.height {
            return;
        }
        if src.a == 0 {
            return;
        }
        if src.a == 255 {
            self.set_pixel(x, y, src);
            return;
        }

        let idx = ((y * self.width + x) * 4) as usize;
        let dst_r = self.pixels[idx] as u32;
        let dst_g = self.pixels[idx + 1] as u32;
        let dst_b = self.pixels[idx + 2] as u32;
        let dst_a = self.pixels[idx + 3] as u32;

        let sa = src.a as u32;
        let inv_sa = 255 - sa;
        let out_a = sa + (dst_a * inv_sa + 127) / 255;

        if out_a == 0 {
            self.pixels[idx] = 0;
            self.pixels[idx + 1] = 0;
            self.pixels[idx + 2] = 0;
            self.pixels[idx + 3] = 0;
            return;
        }

        let out_r = ((src.r as u32 * sa) + (dst_r * dst_a * inv_sa + 127) / 255) / out_a;
        let out_g = ((src.g as u32 * sa) + (dst_g * dst_a * inv_sa + 127) / 255) / out_a;
        let out_b = ((src.b as u32 * sa) + (dst_b * dst_a * inv_sa + 127) / 255) / out_a;

        self.pixels[idx] = out_r.min(255) as u8;
        self.pixels[idx + 1] = out_g.min(255) as u8;
        self.pixels[idx + 2] = out_b.min(255) as u8;
        self.pixels[idx + 3] = out_a.min(255) as u8;
    }

    pub fn blend_pixel_i32(&mut self, x: i32, y: i32, color: ColorRgba) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.blend_pixel(x as u32, y as u32, color);
    }

    pub fn clear(&mut self, color: ColorRgba) {
        let arr = color.to_array();
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&arr);
        }
    }

    /// Extracts raw RGBA bytes for sub-rectangle `[x..x+w, y..y+h]` in row-major order.
    pub fn extract_rect(&self, x: u32, y: u32, w: u32, h: u32) -> Vec<u8> {
        let mut data = Vec::with_capacity((w as usize) * (h as usize) * 4);
        for row in y..y + h {
            if row >= self.height {
                break;
            }
            let start_idx = ((row * self.width + x) * 4) as usize;
            let len = (w as usize * 4).min(self.pixels.len().saturating_sub(start_idx));
            data.extend_from_slice(&self.pixels[start_idx..start_idx + len]);
        }
        data
    }

    /// Applies raw RGBA bytes for sub-rectangle `[x..x+w, y..y+h]`.
    pub fn apply_rect(&mut self, x: u32, y: u32, w: u32, h: u32, data: &[u8]) {
        let mut data_idx = 0;
        for row in y..y + h {
            if row >= self.height {
                break;
            }
            let start_idx = ((row * self.width + x) * 4) as usize;
            let row_bytes = (w as usize * 4).min(data.len().saturating_sub(data_idx));
            if row_bytes == 0 || start_idx + row_bytes > self.pixels.len() {
                break;
            }
            self.pixels[start_idx..start_idx + row_bytes]
                .copy_from_slice(&data[data_idx..data_idx + row_bytes]);
            data_idx += row_bytes;
        }
    }

    pub fn to_rgba_image(&self) -> Option<RgbaImage> {
        RgbaImage::from_raw(self.width, self.height, self.pixels.clone())
    }

    /// Blends `src` buffer onto `self` at the specified offset.
    pub fn blend_buffer(&mut self, src: &PixelBuffer, offset_x: i32, offset_y: i32) {
        let min_x = offset_x.max(0) as u32;
        let min_y = offset_y.max(0) as u32;
        let max_x = ((offset_x + src.width as i32).max(0) as u32).min(self.width);
        let max_y = ((offset_y + src.height as i32).max(0) as u32).min(self.height);

        for y in min_y..max_y {
            let src_y = (y as i32 - offset_y) as u32;
            for x in min_x..max_x {
                let src_x = (x as i32 - offset_x) as u32;
                if let Some(src_px) = src.get_pixel(src_x, src_y) {
                    if src_px.a > 0 {
                        self.blend_pixel(x, y, src_px);
                    }
                }
            }
        }
    }

    /// Multiplies all non-transparent pixels' alpha by opacity percentage (0..=100).
    pub fn apply_opacity(&mut self, opacity_pct: u8) {
        if opacity_pct >= 100 {
            return;
        }
        let opacity = opacity_pct as u32;
        for chunk in self.pixels.chunks_exact_mut(4) {
            let a = chunk[3] as u32;
            if a > 0 {
                chunk[3] = ((a * opacity + 50) / 100).min(255) as u8;
            }
        }
    }

    /// Computes the bounding box `(min_x, min_y, width, height)` of all non-transparent pixels (alpha > 0).
    /// Returns `None` if the buffer is completely transparent or empty.
    pub fn bounding_box_non_transparent(&self) -> Option<(u32, u32, u32, u32)> {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return None;
        }

        let mut min_x = u32::MAX;
        let mut min_y = u32::MAX;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..h {
            let row_offset = (y * w * 4) as usize;
            for x in 0..w {
                let alpha = self.pixels[row_offset + (x * 4) as usize + 3];
                if alpha > 0 {
                    found = true;
                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }
            }
        }

        if found {
            Some((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
        } else {
            None
        }
    }

    /// Crops a sub-region `[x..x+w, y..y+h]` into a new `PixelBuffer`.
    pub fn crop(&self, x: u32, y: u32, w: u32, h: u32) -> PixelBuffer {
        let mut dst = PixelBuffer::new(w, h, ColorRgba::TRANSPARENT);
        if w == 0 || h == 0 {
            return dst;
        }

        for dy in 0..h {
            let src_y = y + dy;
            if src_y >= self.height {
                break;
            }
            for dx in 0..w {
                let src_x = x + dx;
                if src_x >= self.width {
                    break;
                }
                if let Some(px) = self.get_pixel(src_x, src_y) {
                    dst.set_pixel(dx, dy, px);
                }
            }
        }
        dst
    }

    /// Expands the buffer by adding `pad_pixels` around all four borders filled with `fill`.
    pub fn pad(&self, pad_pixels: u32, fill: ColorRgba) -> PixelBuffer {
        if pad_pixels == 0 {
            return self.clone();
        }
        let new_w = self.width.saturating_add(pad_pixels * 2);
        let new_h = self.height.saturating_add(pad_pixels * 2);
        let mut dst = PixelBuffer::new(new_w, new_h, fill);
        dst.blend_buffer(self, pad_pixels as i32, pad_pixels as i32);
        dst
    }

    /// Scales the buffer by an integer multiple `scale` (1x..=8x) using crisp nearest-neighbor interpolation.
    pub fn scale_nearest_neighbor(&self, scale: u32) -> PixelBuffer {
        if scale <= 1 {
            return self.clone();
        }
        let scale = scale.max(1);
        let new_w = self.width.saturating_mul(scale);
        let new_h = self.height.saturating_mul(scale);
        let mut dst = PixelBuffer::new(new_w, new_h, ColorRgba::TRANSPARENT);

        for y in 0..new_h {
            let src_y = y / scale;
            for x in 0..new_w {
                let src_x = x / scale;
                if let Some(px) = self.get_pixel(src_x, src_y) {
                    dst.set_pixel(x, y, px);
                }
            }
        }
        dst
    }

    /// Flattens the RGBA buffer onto a solid background color, resulting in opaque alpha (255) for all pixels.
    pub fn flatten_on_color(&self, bg_color: ColorRgba) -> PixelBuffer {
        let solid_bg = ColorRgba::rgb(bg_color.r, bg_color.g, bg_color.b);
        let mut dst = PixelBuffer::new(self.width, self.height, solid_bg);
        dst.blend_buffer(self, 0, 0);
        // Ensure strictly 255 alpha everywhere
        for chunk in dst.pixels.chunks_exact_mut(4) {
            chunk[3] = 255;
        }
        dst
    }
}

/// Computes the minimal bounding box `(min_x, min_y, width, height)` of pixel differences between two buffers.
/// Returns `None` if buffers are identical.
pub fn compute_bounding_diff(a: &PixelBuffer, b: &PixelBuffer) -> Option<(u32, u32, u32, u32)> {
    if a.width() != b.width() || a.height() != b.height() {
        return Some((0, 0, a.width().max(b.width()), a.height().max(b.height())));
    }

    let w = a.width();
    let h = a.height();
    let a_raw = a.as_raw();
    let b_raw = b.as_raw();

    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found_diff = false;

    for y in 0..h {
        let row_offset = (y * w * 4) as usize;
        for x in 0..w {
            let px_offset = row_offset + (x * 4) as usize;
            if a_raw[px_offset..px_offset + 4] != b_raw[px_offset..px_offset + 4] {
                found_diff = true;
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
    }

    if found_diff {
        Some((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
    } else {
        None
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

    #[test]
    fn test_alpha_blending() {
        let mut buf = PixelBuffer::new(4, 4, ColorRgba::rgb(255, 255, 255));
        // Blend semi-transparent black (alpha=128) over white (255, 255, 255, 255)
        buf.blend_pixel(0, 0, ColorRgba::rgba(0, 0, 0, 128));
        let blended = buf.get_pixel(0, 0).unwrap();
        // Should be around ~127 gray with alpha 255
        assert!((blended.r as i32 - 127).abs() <= 2);
        assert!((blended.g as i32 - 127).abs() <= 2);
        assert!((blended.b as i32 - 127).abs() <= 2);
        assert_eq!(blended.a, 255);
    }

    #[test]
    fn test_rect_extract_apply_and_bounding_diff() {
        let mut a = PixelBuffer::new(10, 10, ColorRgba::rgb(255, 255, 255));
        let mut b = a.clone();

        assert_eq!(compute_bounding_diff(&a, &b), None);

        b.set_pixel(3, 4, ColorRgba::rgb(255, 0, 0));
        b.set_pixel(6, 7, ColorRgba::rgb(0, 255, 0));

        let diff = compute_bounding_diff(&a, &b);
        assert_eq!(diff, Some((3, 4, 4, 4)));

        let (x, y, w, h) = diff.unwrap();
        let extracted_b = b.extract_rect(x, y, w, h);
        a.apply_rect(x, y, w, h, &extracted_b);

        assert_eq!(a, b);
        assert_eq!(compute_bounding_diff(&a, &b), None);
    }

    #[test]
    fn test_bounding_box_non_transparent_and_crop() {
        let mut buf = PixelBuffer::new(20, 20, ColorRgba::TRANSPARENT);
        assert_eq!(buf.bounding_box_non_transparent(), None);

        buf.set_pixel(5, 6, ColorRgba::rgb(255, 0, 0));
        buf.set_pixel(12, 14, ColorRgba::rgb(0, 255, 0));

        let bbox = buf.bounding_box_non_transparent();
        assert_eq!(bbox, Some((5, 6, 8, 9)));

        let (x, y, w, h) = bbox.unwrap();
        let cropped = buf.crop(x, y, w, h);
        assert_eq!(cropped.width(), 8);
        assert_eq!(cropped.height(), 9);
        assert_eq!(cropped.get_pixel(0, 0), Some(ColorRgba::rgb(255, 0, 0)));
        assert_eq!(cropped.get_pixel(7, 8), Some(ColorRgba::rgb(0, 255, 0)));
    }

    #[test]
    fn test_pad_scale_and_flatten() {
        let mut buf = PixelBuffer::new(4, 4, ColorRgba::TRANSPARENT);
        buf.set_pixel(1, 1, ColorRgba::rgba(255, 0, 0, 128));

        // Pad
        let padded = buf.pad(2, ColorRgba::TRANSPARENT);
        assert_eq!(padded.width(), 8);
        assert_eq!(padded.height(), 8);
        assert_eq!(
            padded.get_pixel(3, 3),
            Some(ColorRgba::rgba(255, 0, 0, 128))
        );

        // Scale nearest neighbor 2x
        let scaled = buf.scale_nearest_neighbor(2);
        assert_eq!(scaled.width(), 8);
        assert_eq!(scaled.height(), 8);
        assert_eq!(
            scaled.get_pixel(2, 2),
            Some(ColorRgba::rgba(255, 0, 0, 128))
        );
        assert_eq!(
            scaled.get_pixel(3, 3),
            Some(ColorRgba::rgba(255, 0, 0, 128))
        );

        // Flatten on white
        let flattened = buf.flatten_on_color(ColorRgba::rgb(255, 255, 255));
        assert_eq!(
            flattened.get_pixel(0, 0),
            Some(ColorRgba::rgb(255, 255, 255))
        );
        let semi = flattened.get_pixel(1, 1).unwrap();
        assert_eq!(semi.a, 255);
        assert!(semi.r > 200); // 128 alpha red over white
    }
}

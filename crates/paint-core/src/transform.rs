//! Image transformation algorithms: Flip, Rotate, Invert, Stretch, Skew, Canvas Resize, and Monochrome conversion.

use crate::image::PixelBuffer;
use crate::palette::{ColorRgba, BLACK, WHITE};

/// Flips the image horizontally (mirror along vertical axis).
pub fn flip_horizontal(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);

    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(w - 1 - x, y, p);
            }
        }
    }
    dst
}

/// Flips the image vertically (mirror along horizontal axis).
pub fn flip_vertical(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);

    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(x, h - 1 - y, p);
            }
        }
    }
    dst
}

/// Rotates the image 90 degrees clockwise.
pub fn rotate_90(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    // New dimensions: width = old height, height = old width
    let mut dst = PixelBuffer::new(h, w, WHITE);

    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(h - 1 - y, x, p);
            }
        }
    }
    dst
}

/// Rotates the image 180 degrees.
pub fn rotate_180(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(w, h, WHITE);

    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(w - 1 - x, h - 1 - y, p);
            }
        }
    }
    dst
}

/// Rotates the image 270 degrees clockwise (90 degrees counter-clockwise).
pub fn rotate_270(buffer: &PixelBuffer) -> PixelBuffer {
    let w = buffer.width();
    let h = buffer.height();
    let mut dst = PixelBuffer::new(h, w, WHITE);

    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(y, w - 1 - x, p);
            }
        }
    }
    dst
}

/// Inverts colors (RGB = 255 - RGB) in-place.
pub fn invert_colors(buffer: &mut PixelBuffer) {
    let w = buffer.width();
    let h = buffer.height();
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                buffer.set_pixel(x, y, ColorRgba::rgba(255 - p.r, 255 - p.g, 255 - p.b, p.a));
            }
        }
    }
}

/// Stretches/scales the image by integer percentage (e.g. 100% = 1.0x, 200% = 2.0x, 50% = 0.5x)
/// using nearest-neighbor pixel sampling.
pub fn stretch(buffer: &PixelBuffer, scale_x_pct: u32, scale_y_pct: u32) -> PixelBuffer {
    let orig_w = buffer.width();
    let orig_h = buffer.height();
    let new_w = ((orig_w as u64 * scale_x_pct as u64 + 50) / 100).max(1) as u32;
    let new_h = ((orig_h as u64 * scale_y_pct as u64 + 50) / 100).max(1) as u32;

    let mut dst = PixelBuffer::new(new_w, new_h, WHITE);

    for y in 0..new_h {
        let src_y = ((y as u64 * orig_h as u64) / new_h as u64).min((orig_h - 1) as u64) as u32;
        for x in 0..new_w {
            let src_x = ((x as u64 * orig_w as u64) / new_w as u64).min((orig_w - 1) as u64) as u32;
            if let Some(p) = buffer.get_pixel(src_x, src_y) {
                dst.set_pixel(x, y, p);
            }
        }
    }
    dst
}

/// Skews the image horizontally and/or vertically (angles between -89 and 89 degrees).
pub fn skew(
    buffer: &PixelBuffer,
    skew_x_deg: i32,
    skew_y_deg: i32,
    fill: ColorRgba,
) -> PixelBuffer {
    let mut current = buffer.clone();

    // Horizontal skew
    if skew_x_deg != 0 {
        let angle = (skew_x_deg.clamp(-89, 89) as f64).to_radians();
        let tan_val = angle.tan();
        let w = current.width();
        let h = current.height();
        let max_shift = (h as f64 * tan_val.abs()).ceil() as u32;
        let new_w = w + max_shift;
        let mut skewed = PixelBuffer::new(new_w, h, fill);

        for y in 0..h {
            let shift = if skew_x_deg > 0 {
                ((h - 1 - y) as f64 * tan_val).round() as i32
            } else {
                ((y as f64) * (-tan_val)).round() as i32
            };

            for x in 0..w {
                if let Some(p) = current.get_pixel(x, y) {
                    let dst_x = x as i32 + shift;
                    skewed.set_pixel_i32(dst_x, y as i32, p);
                }
            }
        }
        current = skewed;
    }

    // Vertical skew
    if skew_y_deg != 0 {
        let angle = (skew_y_deg.clamp(-89, 89) as f64).to_radians();
        let tan_val = angle.tan();
        let w = current.width();
        let h = current.height();
        let max_shift = (w as f64 * tan_val.abs()).ceil() as u32;
        let new_h = h + max_shift;
        let mut skewed = PixelBuffer::new(w, new_h, fill);

        for x in 0..w {
            let shift = if skew_y_deg > 0 {
                ((w - 1 - x) as f64 * tan_val).round() as i32
            } else {
                ((x as f64) * (-tan_val)).round() as i32
            };

            for y in 0..h {
                if let Some(p) = current.get_pixel(x, y) {
                    let dst_y = y as i32 + shift;
                    skewed.set_pixel_i32(x as i32, dst_y, p);
                }
            }
        }
        current = skewed;
    }

    current
}

/// Resizes the canvas dimensions (cropping or expanding with fill color).
pub fn resize_canvas(
    buffer: &PixelBuffer,
    new_width: u32,
    new_height: u32,
    fill: ColorRgba,
) -> PixelBuffer {
    let mut dst = PixelBuffer::new(new_width.max(1), new_height.max(1), fill);
    let copy_w = buffer.width().min(new_width);
    let copy_h = buffer.height().min(new_height);

    for y in 0..copy_h {
        for x in 0..copy_w {
            if let Some(p) = buffer.get_pixel(x, y) {
                dst.set_pixel(x, y, p);
            }
        }
    }
    dst
}

/// Rotates the image by an arbitrary angle in degrees clockwise around its center,
/// keeping crisp nearest-neighbor pixel fidelity (anti-aliasing disabled by default).
pub fn rotate_arbitrary(buffer: &PixelBuffer, angle_deg: f32, fill: ColorRgba) -> PixelBuffer {
    let rad = angle_deg.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();

    let orig_w = buffer.width() as f32;
    let orig_h = buffer.height() as f32;

    // Bounding box of rotated rectangle
    let p1 = (orig_w * cos_a, orig_w * sin_a);
    let p2 = (
        orig_w * cos_a - orig_h * sin_a,
        orig_w * sin_a + orig_h * cos_a,
    );
    let p3 = (-orig_h * sin_a, orig_h * cos_a);

    let min_x = 0.0_f32.min(p1.0).min(p2.0).min(p3.0);
    let max_x = 0.0_f32.max(p1.0).max(p2.0).max(p3.0);
    let min_y = 0.0_f32.min(p1.1).min(p2.1).min(p3.1);
    let max_y = 0.0_f32.max(p1.1).max(p2.1).max(p3.1);

    let new_w = ((max_x - min_x).ceil() as u32).max(1);
    let new_h = ((max_y - min_y).ceil() as u32).max(1);

    let cx_dst = new_w as f32 / 2.0;
    let cy_dst = new_h as f32 / 2.0;
    let cx_src = orig_w / 2.0;
    let cy_src = orig_h / 2.0;

    let mut dst = PixelBuffer::new(new_w, new_h, fill);

    for y in 0..new_h {
        let dy = y as f32 - cy_dst;
        for x in 0..new_w {
            let dx = x as f32 - cx_dst;

            // Inverse rotation to sample from source: (x_src, y_src)
            let src_x = (dx * cos_a + dy * sin_a + cx_src).round() as i32;
            let src_y = (-dx * sin_a + dy * cos_a + cy_src).round() as i32;

            if src_x >= 0
                && src_x < buffer.width() as i32
                && src_y >= 0
                && src_y < buffer.height() as i32
            {
                if let Some(p) = buffer.get_pixel_i32(src_x, src_y) {
                    dst.set_pixel(x, y, p);
                }
            }
        }
    }
    dst
}

/// Converts image in-place to pure black and white (monochrome 1-bit style thresholding).
pub fn convert_to_black_and_white(buffer: &mut PixelBuffer) {
    let w = buffer.width();
    let h = buffer.height();
    for y in 0..h {
        for x in 0..w {
            if let Some(p) = buffer.get_pixel(x, y) {
                let lum = 0.299 * p.r as f32 + 0.587 * p.g as f32 + 0.114 * p.b as f32;
                let bw = if lum >= 128.0 { WHITE } else { BLACK };
                buffer.set_pixel(x, y, ColorRgba::rgba(bw.r, bw.g, bw.b, p.a));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::RED;

    #[test]
    fn test_flip_horizontal_and_vertical() {
        let mut buf = PixelBuffer::new(4, 4, WHITE);
        buf.set_pixel(0, 0, RED);

        let flip_h = flip_horizontal(&buf);
        assert_eq!(flip_h.get_pixel(3, 0), Some(RED));
        assert_eq!(flip_h.get_pixel(0, 0), Some(WHITE));

        let flip_v = flip_vertical(&buf);
        assert_eq!(flip_v.get_pixel(0, 3), Some(RED));
        assert_eq!(flip_v.get_pixel(0, 0), Some(WHITE));
    }

    #[test]
    fn test_rotations() {
        let mut buf = PixelBuffer::new(3, 2, WHITE);
        buf.set_pixel(0, 0, RED); // Top-left

        let r90 = rotate_90(&buf);
        assert_eq!(r90.width(), 2);
        assert_eq!(r90.height(), 3);
        assert_eq!(r90.get_pixel(1, 0), Some(RED)); // Top-right in 2x3

        let r180 = rotate_180(&buf);
        assert_eq!(r180.width(), 3);
        assert_eq!(r180.height(), 2);
        assert_eq!(r180.get_pixel(2, 1), Some(RED)); // Bottom-right

        let r270 = rotate_270(&buf);
        assert_eq!(r270.width(), 2);
        assert_eq!(r270.height(), 3);
        assert_eq!(r270.get_pixel(0, 2), Some(RED)); // Bottom-left
    }

    #[test]
    fn test_invert_colors() {
        let mut buf = PixelBuffer::new(2, 2, ColorRgba::rgb(255, 0, 100));
        invert_colors(&mut buf);
        assert_eq!(buf.get_pixel(0, 0), Some(ColorRgba::rgb(0, 255, 155)));
    }

    #[test]
    fn test_stretch() {
        let mut buf = PixelBuffer::new(2, 2, WHITE);
        buf.set_pixel(0, 0, RED);

        let stretched = stretch(&buf, 200, 200);
        assert_eq!(stretched.width(), 4);
        assert_eq!(stretched.height(), 4);
        assert_eq!(stretched.get_pixel(0, 0), Some(RED));
        assert_eq!(stretched.get_pixel(1, 1), Some(RED));
        assert_eq!(stretched.get_pixel(2, 2), Some(WHITE));
    }

    #[test]
    fn test_resize_canvas() {
        let buf = PixelBuffer::new(2, 2, RED);
        let resized = resize_canvas(&buf, 4, 4, WHITE);
        assert_eq!(resized.width(), 4);
        assert_eq!(resized.height(), 4);
        assert_eq!(resized.get_pixel(0, 0), Some(RED));
        assert_eq!(resized.get_pixel(1, 1), Some(RED));
        assert_eq!(resized.get_pixel(2, 2), Some(WHITE));
    }
}

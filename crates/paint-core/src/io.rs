//! Image I/O support for BMP, JPEG, GIF, TIFF, and PNG formats.

use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, Write};
use std::path::Path;

use image::{DynamicImage, ImageError, ImageFormat, ImageReader, RgbaImage};
use thiserror::Error;

use crate::image::PixelBuffer;
use crate::palette::ColorRgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFileFormat {
    Png,
    Bmp,
    Jpeg,
    Gif,
    Tiff,
}

impl ImageFileFormat {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let ext = path.as_ref().extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "png" => Some(Self::Png),
            "bmp" | "dib" => Some(Self::Bmp),
            "jpg" | "jpeg" | "jpe" | "jfif" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "tif" | "tiff" => Some(Self::Tiff),
            _ => None,
        }
    }

    pub fn to_image_format(self) -> ImageFormat {
        match self {
            Self::Png => ImageFormat::Png,
            Self::Bmp => ImageFormat::Bmp,
            Self::Jpeg => ImageFormat::Jpeg,
            Self::Gif => ImageFormat::Gif,
            Self::Tiff => ImageFormat::Tiff,
        }
    }

    pub fn default_extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Bmp => "bmp",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::Tiff => "tif",
        }
    }

    pub fn filter_name(self) -> &'static str {
        match self {
            Self::Png => "PNG Image (*.png)",
            Self::Bmp => "Bitmap Image (*.bmp)",
            Self::Jpeg => "JPEG Image (*.jpg, *.jpeg)",
            Self::Gif => "GIF Image (*.gif)",
            Self::Tiff => "TIFF Image (*.tif, *.tiff)",
        }
    }
}

#[derive(Error, Debug)]
pub enum ImageIoError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image codec error: {0}")]
    Codec(#[from] ImageError),
    #[error("Unsupported or unknown format")]
    UnknownFormat,
    #[error("Failed to construct pixel buffer from raw image")]
    InvalidPixelBuffer,
}

/// Decodes an image from any `Read + Seek` stream into a `PixelBuffer`.
pub fn load_from_reader<R: Read + Seek>(reader: R) -> Result<PixelBuffer, ImageIoError> {
    let img_reader = ImageReader::new(BufReader::new(reader))
        .with_guessed_format()
        .map_err(ImageIoError::Io)?;
    let dynamic_img = img_reader.decode().map_err(ImageIoError::Codec)?;
    let rgba = dynamic_img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let raw = rgba.into_raw();

    let mut buf = PixelBuffer::new(width, height, ColorRgba::rgb(255, 255, 255));
    buf.as_raw_mut().copy_from_slice(&raw);
    Ok(buf)
}

/// Decodes an image from in-memory byte slice into a `PixelBuffer`.
pub fn load_from_memory(data: &[u8]) -> Result<PixelBuffer, ImageIoError> {
    load_from_reader(Cursor::new(data))
}

/// Loads and decodes an image file from a filesystem path.
pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<PixelBuffer, ImageIoError> {
    let file = File::open(path)?;
    load_from_reader(file)
}

/// Encodes a `PixelBuffer` into in-memory bytes using the requested format.
pub fn save_to_memory(
    buffer: &PixelBuffer,
    format: ImageFileFormat,
) -> Result<Vec<u8>, ImageIoError> {
    let mut cursor = Cursor::new(Vec::new());
    save_to_writer(buffer, &mut cursor, format)?;
    Ok(cursor.into_inner())
}

/// Encodes a `PixelBuffer` directly to a `Write + Seek` stream.
pub fn save_to_writer<W: Write + Seek>(
    buffer: &PixelBuffer,
    writer: &mut W,
    format: ImageFileFormat,
) -> Result<(), ImageIoError> {
    let rgba_img = RgbaImage::from_raw(buffer.width(), buffer.height(), buffer.as_raw().to_vec())
        .ok_or(ImageIoError::InvalidPixelBuffer)?;

    let dynamic_img = match format {
        ImageFileFormat::Jpeg => {
            // For JPEG, convert to RGB8 with white background for transparency
            let mut rgb_img = image::RgbImage::new(buffer.width(), buffer.height());
            for y in 0..buffer.height() {
                for x in 0..buffer.width() {
                    let p = buffer
                        .get_pixel(x, y)
                        .unwrap_or(ColorRgba::rgb(255, 255, 255));
                    // Blend onto white if alpha < 255
                    let a = p.a as f32 / 255.0;
                    let r = ((p.r as f32 * a) + 255.0 * (1.0 - a)).round() as u8;
                    let g = ((p.g as f32 * a) + 255.0 * (1.0 - a)).round() as u8;
                    let b = ((p.b as f32 * a) + 255.0 * (1.0 - a)).round() as u8;
                    rgb_img.put_pixel(x, y, image::Rgb([r, g, b]));
                }
            }
            DynamicImage::ImageRgb8(rgb_img)
        }
        _ => DynamicImage::ImageRgba8(rgba_img),
    };

    dynamic_img
        .write_to(writer, format.to_image_format())
        .map_err(ImageIoError::Codec)?;
    Ok(())
}

/// Encodes and saves a `PixelBuffer` to a filesystem path.
pub fn save_to_path<P: AsRef<Path>>(
    buffer: &PixelBuffer,
    path: P,
    format: ImageFileFormat,
) -> Result<(), ImageIoError> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    save_to_writer(buffer, &mut writer, format)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::RED;

    #[test]
    fn test_format_codecs_roundtrip() {
        let mut original = PixelBuffer::new(32, 32, ColorRgba::rgb(255, 255, 255));
        // Draw some red patterns
        for i in 0..16 {
            original.set_pixel(i, i, RED);
            original.set_pixel(31 - i, i, ColorRgba::rgb(0, 128, 255));
        }

        // Test PNG
        let png_bytes = save_to_memory(&original, ImageFileFormat::Png).expect("save png");
        let loaded_png = load_from_memory(&png_bytes).expect("load png");
        assert_eq!(loaded_png.width(), 32);
        assert_eq!(loaded_png.height(), 32);
        assert_eq!(loaded_png.get_pixel(5, 5), Some(RED));

        // Test BMP
        let bmp_bytes = save_to_memory(&original, ImageFileFormat::Bmp).expect("save bmp");
        let loaded_bmp = load_from_memory(&bmp_bytes).expect("load bmp");
        assert_eq!(loaded_bmp.width(), 32);
        assert_eq!(loaded_bmp.height(), 32);
        assert_eq!(loaded_bmp.get_pixel(5, 5), Some(RED));

        // Test GIF
        let gif_bytes = save_to_memory(&original, ImageFileFormat::Gif).expect("save gif");
        let loaded_gif = load_from_memory(&gif_bytes).expect("load gif");
        assert_eq!(loaded_gif.width(), 32);
        assert_eq!(loaded_gif.height(), 32);

        // Test TIFF
        let tiff_bytes = save_to_memory(&original, ImageFileFormat::Tiff).expect("save tiff");
        let loaded_tiff = load_from_memory(&tiff_bytes).expect("load tiff");
        assert_eq!(loaded_tiff.width(), 32);
        assert_eq!(loaded_tiff.height(), 32);
        assert_eq!(loaded_tiff.get_pixel(5, 5), Some(RED));

        // Test JPEG (lossy, just check dimension & loadability)
        let jpeg_bytes = save_to_memory(&original, ImageFileFormat::Jpeg).expect("save jpeg");
        let loaded_jpeg = load_from_memory(&jpeg_bytes).expect("load jpeg");
        assert_eq!(loaded_jpeg.width(), 32);
        assert_eq!(loaded_jpeg.height(), 32);
    }

    #[test]
    fn test_format_from_path() {
        assert_eq!(
            ImageFileFormat::from_path(Path::new("test.png")),
            Some(ImageFileFormat::Png)
        );
        assert_eq!(
            ImageFileFormat::from_path(Path::new("c:/path/image.bmp")),
            Some(ImageFileFormat::Bmp)
        );
        assert_eq!(
            ImageFileFormat::from_path(Path::new("photo.jpg")),
            Some(ImageFileFormat::Jpeg)
        );
        assert_eq!(
            ImageFileFormat::from_path(Path::new("anim.gif")),
            Some(ImageFileFormat::Gif)
        );
        assert_eq!(
            ImageFileFormat::from_path(Path::new("scan.tiff")),
            Some(ImageFileFormat::Tiff)
        );
        assert_eq!(ImageFileFormat::from_path(Path::new("unknown.xyz")), None);
    }
}

use paint_core::image::PixelBuffer;
use paint_core::palette::{ColorRgba, BLUE, GREEN, RED, WHITE};
use paint_core::tools::mixer::{apply_mixer_effect, MixerEffect};
use std::path::{Path, PathBuf};

fn get_golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("golden")
}

fn assert_or_save_golden(buffer: &PixelBuffer, golden_filename: &str) {
    let golden_dir = get_golden_dir();
    let golden_path = golden_dir.join(golden_filename);
    let current_image = buffer.to_rgba_image().expect("failed to get RgbaImage");

    if !golden_path.exists() {
        std::fs::create_dir_all(&golden_dir).ok();
        current_image
            .save(&golden_path)
            .expect("failed to save initial golden image");
    } else {
        let golden_image = image::open(&golden_path)
            .expect("failed to open golden image")
            .to_rgba8();

        assert_eq!(
            current_image.dimensions(),
            golden_image.dimensions(),
            "Dimensions mismatch for {}",
            golden_filename
        );

        let mut diff_count = 0;
        for (x, y, pixel) in current_image.enumerate_pixels() {
            let golden_pixel = golden_image.get_pixel(x, y);
            if pixel != golden_pixel {
                diff_count += 1;
            }
        }

        assert_eq!(
            diff_count, 0,
            "Pixel mismatch for {}: {} pixels differed",
            golden_filename, diff_count
        );
    }
}

/// Creates a standardized colorful test image with distinct shapes, gradients, and lines.
fn create_test_pattern(width: u32, height: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, WHITE);

    // Diagonal stripes
    for y in 0..height {
        for x in 0..width {
            if (x + y) % 16 < 8 {
                buf.set_pixel(x, y, ColorRgba::rgb(220, 240, 255));
            }
        }
    }

    // Colored boxes
    for y in 10..height.saturating_sub(10) {
        for x in 10..(width / 3).min(width) {
            buf.set_pixel(x, y, RED);
        }
        for x in (width / 3)..(2 * width / 3).min(width) {
            buf.set_pixel(x, y, GREEN);
        }
        for x in (2 * width / 3)..width.saturating_sub(10) {
            buf.set_pixel(x, y, BLUE);
        }
    }

    // Center circular yellow core
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let r = (width.min(height) / 6) as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    buf.set_pixel(px as u32, py as u32, ColorRgba::rgb(255, 220, 0));
                }
            }
        }
    }

    buf
}

#[test]
fn test_golden_electric_mixer_tray_1_all_14() {
    let cell_w = 64u32;
    let cell_h = 64u32;
    let cols = 7u32;
    let rows = 2u32; // 14 effects: 7x2
    let mut sheet = PixelBuffer::new(cols * cell_w, rows * cell_h, WHITE);

    let test_pat = create_test_pattern(cell_w, cell_h);

    for (idx, &effect) in MixerEffect::TRAY_1.iter().enumerate() {
        let col = (idx as u32) % cols;
        let row = (idx as u32) / cols;
        let transformed = apply_mixer_effect(&test_pat, effect, 42);

        // Blit into sheet
        let offset_x = col * cell_w;
        let offset_y = row * cell_h;
        for dy in 0..cell_h {
            for dx in 0..cell_w {
                if let Some(p) = transformed.get_pixel(dx, dy) {
                    sheet.set_pixel(offset_x + dx, offset_y + dy, p);
                }
            }
        }
    }

    assert_or_save_golden(&sheet, "golden_electric_mixer_tray_1.png");
}

#[test]
fn test_golden_electric_mixer_tray_2_all_14() {
    let cell_w = 64u32;
    let cell_h = 64u32;
    let cols = 7u32;
    let rows = 2u32; // 14 effects: 7x2
    let mut sheet = PixelBuffer::new(cols * cell_w, rows * cell_h, WHITE);

    let test_pat = create_test_pattern(cell_w, cell_h);

    for (idx, &effect) in MixerEffect::TRAY_2.iter().enumerate() {
        let col = (idx as u32) % cols;
        let row = (idx as u32) / cols;
        let transformed = apply_mixer_effect(&test_pat, effect, 1337);

        // Blit into sheet
        let offset_x = col * cell_w;
        let offset_y = row * cell_h;
        for dy in 0..cell_h {
            for dx in 0..cell_w {
                if let Some(p) = transformed.get_pixel(dx, dy) {
                    sheet.set_pixel(offset_x + dx, offset_y + dy, p);
                }
            }
        }
    }

    assert_or_save_golden(&sheet, "golden_electric_mixer_tray_2.png");
}

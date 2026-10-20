use paint_core::geometry::Point;
use paint_core::image::PixelBuffer;
use paint_core::palette::{ColorRgba, BLACK, BLUE, GREEN, RED, WHITE};
use paint_core::tools::wacky::{
    apply_stamp, draw_wacky_brush_stroke, pattern_flood_fill, FillPattern, StampShape,
    WackyBrushShape, WackyBrushState,
};
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

#[test]
fn test_golden_wacky_brushes_all_18_variants() {
    let mut buffer = PixelBuffer::new(360, 180, WHITE);

    for (i, &shape) in WackyBrushShape::ALL.iter().enumerate() {
        let x_start = 15 + (i as i32 % 6) * 58;
        let y_start = 15 + (i as i32 / 6) * 55;
        let mut state = WackyBrushState::default();

        let stroke_color = match i % 3 {
            0 => RED,
            1 => BLUE,
            _ => BLACK,
        };

        draw_wacky_brush_stroke(
            &mut buffer,
            Point::new(x_start, y_start),
            Point::new(x_start + 40, y_start + 30),
            shape,
            4,
            stroke_color,
            WHITE,
            &mut state,
        );
    }

    assert_or_save_golden(&buffer, "golden_wacky_brushes_all_18_variants.png");
}

#[test]
fn test_golden_rubber_stamps_all_16() {
    let mut buffer = PixelBuffer::new(160, 160, WHITE);

    for (i, &stamp) in StampShape::ALL.iter().enumerate() {
        let col = i as i32 % 4;
        let row = i as i32 / 4;
        let cx = 20 + col * 40;
        let cy = 20 + row * 40;

        let color = match i % 4 {
            0 => RED,
            1 => BLUE,
            2 => GREEN,
            _ => BLACK,
        };

        apply_stamp(&mut buffer, Point::new(cx, cy), stamp, 2, color);
    }

    assert_or_save_golden(&buffer, "golden_rubber_stamps_16.png");
}

#[test]
fn test_golden_pattern_fills_all_8() {
    let mut buffer = PixelBuffer::new(160, 80, WHITE);

    for (i, &pattern) in FillPattern::ALL.iter().enumerate() {
        let col = i as i32 % 4;
        let row = i as i32 / 4;
        let x_min = col * 40;
        let y_min = row * 40;

        // Draw bounding outline
        for x in x_min..x_min + 40 {
            buffer.set_pixel_i32(x, y_min, BLACK);
            buffer.set_pixel_i32(x, y_min + 39, BLACK);
        }
        for y in y_min..y_min + 40 {
            buffer.set_pixel_i32(x_min, y, BLACK);
            buffer.set_pixel_i32(x_min + 39, y, BLACK);
        }

        let fg = match i % 3 {
            0 => RED,
            1 => BLUE,
            _ => BLACK,
        };
        let bg = ColorRgba::rgb(240, 240, 240);

        pattern_flood_fill(
            &mut buffer,
            Point::new(x_min + 15, y_min + 15),
            pattern,
            0,
            fg,
            bg,
        );
    }

    assert_or_save_golden(&buffer, "golden_pattern_fills_8.png");
}

#[test]
fn test_wacky_brush_options_deterministic_seed() {
    let shape = WackyBrushShape::Splatter;
    let options = paint_core::tools::wacky::WackyBrushOptions {
        seed_mode: paint_core::tools::wacky::SeedMode::Locked(133742),
        scatter_radius: 8,
        ..Default::default()
    };

    let mut buf1 = PixelBuffer::new(80, 80, WHITE);
    let mut state1 = WackyBrushState::default();
    paint_core::tools::wacky::draw_wacky_brush_stroke_with_options(
        &mut buf1,
        Point::new(10, 10),
        Point::new(70, 70),
        shape,
        4,
        RED,
        WHITE,
        &mut state1,
        &options,
    );

    let mut buf2 = PixelBuffer::new(80, 80, WHITE);
    let mut state2 = WackyBrushState::default();
    paint_core::tools::wacky::draw_wacky_brush_stroke_with_options(
        &mut buf2,
        Point::new(10, 10),
        Point::new(70, 70),
        shape,
        4,
        RED,
        WHITE,
        &mut state2,
        &options,
    );

    assert_eq!(
        buf1.to_rgba_image().unwrap().into_raw(),
        buf2.to_rgba_image().unwrap().into_raw(),
        "Locked seed must yield bit-identical pixel buffers"
    );
}

#[test]
fn test_wacky_brush_spacing_continuous_vs_custom() {
    let mut buf_continuous = PixelBuffer::new(100, 20, WHITE);
    let mut state_c = WackyBrushState::default();
    let opt_continuous = paint_core::tools::wacky::WackyBrushOptions {
        spacing: paint_core::tools::wacky::BrushSpacing::Continuous,
        ..Default::default()
    };
    paint_core::tools::wacky::draw_wacky_brush_stroke_with_options(
        &mut buf_continuous,
        Point::new(10, 10),
        Point::new(90, 10),
        WackyBrushShape::PineTree,
        2,
        BLACK,
        WHITE,
        &mut state_c,
        &opt_continuous,
    );

    let mut buf_custom = PixelBuffer::new(100, 20, WHITE);
    let mut state_cust = WackyBrushState::default();
    let opt_custom = paint_core::tools::wacky::WackyBrushOptions {
        spacing: paint_core::tools::wacky::BrushSpacing::Custom(40),
        ..Default::default()
    };
    paint_core::tools::wacky::draw_wacky_brush_stroke_with_options(
        &mut buf_custom,
        Point::new(10, 10),
        Point::new(90, 10),
        WackyBrushShape::PineTree,
        2,
        BLACK,
        WHITE,
        &mut state_cust,
        &opt_custom,
    );

    let count_c = buf_continuous
        .to_rgba_image()
        .unwrap()
        .pixels()
        .filter(|p| p[0] != 255)
        .count();
    let count_cust = buf_custom
        .to_rgba_image()
        .unwrap()
        .pixels()
        .filter(|p| p[0] != 255)
        .count();

    assert!(
        count_c > count_cust,
        "Continuous spacing should stamp many more points than Custom(40)"
    );
}

#[test]
fn test_stamp_sub_quadrant_rotation_and_outline() {
    let mut buffer = PixelBuffer::new(60, 60, WHITE);
    let options = paint_core::tools::wacky::StampOptions {
        scale: 2,
        rotation_deg: 45.0,
        spacing: paint_core::tools::wacky::StampSpacing::Auto,
        tint: paint_core::tools::wacky::StampTint::Outlined {
            fill: RED,
            outline: BLACK,
        },
    };

    paint_core::tools::wacky::apply_stamp_with_options(
        &mut buffer,
        Point::new(30, 30),
        StampShape::Star,
        &options,
    );

    let has_red = buffer
        .to_rgba_image()
        .unwrap()
        .pixels()
        .any(|p| p[0] == 255 && p[1] == 0 && p[2] == 0);
    let has_black = buffer
        .to_rgba_image()
        .unwrap()
        .pixels()
        .any(|p| p[0] == 0 && p[1] == 0 && p[2] == 0);

    assert!(has_red, "Outlined stamp must have red interior");
    assert!(has_black, "Outlined stamp must have black outline");
}

#[test]
fn test_pattern_fill_scoped_clip_rect() {
    let mut buffer = PixelBuffer::new(100, 100, WHITE);
    let clip = paint_core::geometry::Rect::from_coords(20, 20, 50, 50);
    let options = paint_core::tools::wacky::PatternFillOptions {
        scale: 2,
        origin: Point::new(5, 5),
        clip_rect: Some(clip),
    };

    paint_core::tools::wacky::pattern_flood_fill_with_options(
        &mut buffer,
        Point::new(30, 30),
        FillPattern::Checkerboard,
        0,
        RED,
        WHITE,
        &options,
    );

    // Outside the clip rect, pixels must remain purely WHITE
    assert_eq!(buffer.get_pixel_i32(10, 10), Some(WHITE));
    assert_eq!(buffer.get_pixel_i32(80, 80), Some(WHITE));

    // Inside the clip rect, some pixels should be RED
    let inside_reds = (20..=50)
        .flat_map(|y| (20..=50).map(move |x| (x, y)))
        .filter(|&(x, y)| buffer.get_pixel_i32(x, y) == Some(RED))
        .count();
    assert!(
        inside_reds > 10,
        "Checkerboard pattern should fill inside clip rect"
    );
}

#[test]
fn test_mixer_effect_scoped_bounding_box() {
    let mut buffer = PixelBuffer::new(100, 100, WHITE);
    // Draw horizontal band across entire width
    for y in 40..60 {
        for x in 0..100 {
            buffer.set_pixel_i32(x, y, RED);
        }
    }

    let clip = paint_core::geometry::Rect::from_coords(20, 20, 50, 80);
    let options = paint_core::tools::mixer::MixerOptions {
        seed: 42,
        scope: paint_core::tools::mixer::MixerScope::BoundingBox(clip),
        params: paint_core::tools::mixer::MixerParams::default(),
    };

    let processed = paint_core::tools::mixer::apply_mixer_effect_with_options(
        &buffer,
        paint_core::tools::mixer::MixerEffect::Wave,
        &options,
    );

    // Outside the bounding box, pixels must match original EXACTLY
    for y in 0..100 {
        for x in 0..100 {
            if x < 20 || x > 50 || y < 20 || y > 80 {
                assert_eq!(
                    buffer.get_pixel_i32(x, y),
                    processed.get_pixel_i32(x, y),
                    "Pixel outside scoped box at ({}, {}) was modified!",
                    x,
                    y
                );
            }
        }
    }
}

use paint_core::geometry::Point;
use paint_core::image::PixelBuffer;
use paint_core::palette::{
    BLACK, BLUE, DARK_RED, GREEN, LIGHT_BLUE, LIGHT_GREEN, LIGHT_RED, LIGHT_YELLOW, PURPLE, RED,
    WHITE, XP_PALETTE_28,
};
use paint_core::tools::airbrush::{apply_airbrush_spray, AirbrushSize};
use paint_core::tools::brush::{draw_brush_stroke, BrushShape};
use paint_core::tools::eraser::{draw_color_eraser_stroke, draw_eraser_stroke, EraserSize};
use paint_core::tools::fill::flood_fill;
use paint_core::tools::pencil::draw_pencil_stroke;
use paint_core::tools::selection::{
    lift_freeform_selection, lift_rectangular_selection, stamp_floating_selection, SelectionMode,
};
use paint_core::tools::shapes::{
    draw_curve_cubic, draw_curve_quadratic, draw_ellipse, draw_polygon, draw_rectangle,
    draw_rounded_rectangle, FillStyle,
};
use paint_core::tools::text::draw_text_bitmap;
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
        assert_eq!(
            current_image.into_raw(),
            golden_image.into_raw(),
            "Pixel mismatch with golden image: {}",
            golden_filename
        );
    }
}

#[test]
fn test_golden_eraser_and_color_eraser() {
    let mut buffer = PixelBuffer::new(80, 80, WHITE);

    // Draw background color bands
    for y in 10..70 {
        for x in 10..70 {
            let color = if x < 30 {
                RED
            } else if x < 50 {
                GREEN
            } else {
                BLUE
            };
            buffer.set_pixel_i32(x, y, color);
        }
    }

    // Normal eraser stroke across all bands
    draw_eraser_stroke(
        &mut buffer,
        Point::new(5, 20),
        Point::new(75, 20),
        EraserSize::Small,
        WHITE,
    );
    draw_eraser_stroke(
        &mut buffer,
        Point::new(5, 35),
        Point::new(75, 35),
        EraserSize::Large,
        LIGHT_YELLOW,
    );

    // Color eraser replacing ONLY GREEN with PURPLE
    draw_color_eraser_stroke(
        &mut buffer,
        Point::new(5, 55),
        Point::new(75, 55),
        EraserSize::XLarge,
        GREEN,
        PURPLE,
    );

    assert_or_save_golden(&buffer, "golden_eraser_and_color_eraser.png");
}

#[test]
fn test_golden_flood_fill() {
    let mut buffer = PixelBuffer::new(64, 64, WHITE);

    // Draw nested shapes
    draw_rectangle(
        &mut buffer,
        Point::new(4, 4),
        Point::new(59, 59),
        FillStyle::OutlineOnly,
        1,
        BLACK,
        WHITE,
    );
    draw_ellipse(
        &mut buffer,
        Point::new(10, 10),
        Point::new(53, 53),
        FillStyle::OutlineOnly,
        1,
        BLACK,
        WHITE,
    );
    draw_rectangle(
        &mut buffer,
        Point::new(20, 20),
        Point::new(43, 43),
        FillStyle::OutlineOnly,
        1,
        BLACK,
        WHITE,
    );

    // Fill regions with different colors
    flood_fill(&mut buffer, Point::new(6, 6), LIGHT_BLUE);
    flood_fill(&mut buffer, Point::new(15, 15), LIGHT_GREEN);
    flood_fill(&mut buffer, Point::new(32, 32), LIGHT_RED);

    assert_or_save_golden(&buffer, "golden_flood_fill.png");
}

#[test]
fn test_golden_brush_12_tips() {
    let mut buffer = PixelBuffer::new(100, 80, WHITE);

    for (i, &shape) in BrushShape::ALL.iter().enumerate() {
        let col = i % 4;
        let row = i / 4;
        let start_x = 10 + (col as i32) * 22;
        let start_y = 10 + (row as i32) * 24;
        let color = XP_PALETTE_28[i + 2];

        draw_brush_stroke(
            &mut buffer,
            Point::new(start_x, start_y),
            Point::new(start_x + 14, start_y + 12),
            shape,
            color,
        );
    }

    assert_or_save_golden(&buffer, "golden_brush_12_tips.png");
}

#[test]
fn test_golden_airbrush_patterns() {
    let mut buffer = PixelBuffer::new(80, 40, WHITE);

    apply_airbrush_spray(
        &mut buffer,
        Point::new(15, 20),
        AirbrushSize::Small,
        RED,
        42,
    );
    apply_airbrush_spray(
        &mut buffer,
        Point::new(40, 20),
        AirbrushSize::Medium,
        GREEN,
        42,
    );
    apply_airbrush_spray(
        &mut buffer,
        Point::new(65, 20),
        AirbrushSize::Large,
        BLUE,
        42,
    );

    assert_or_save_golden(&buffer, "golden_airbrush_patterns.png");
}

#[test]
fn test_golden_shapes_all_styles() {
    let mut buffer = PixelBuffer::new(120, 90, WHITE);

    // 1. Rectangles (Outline, FilledWithOutline, FillOnly)
    draw_rectangle(
        &mut buffer,
        Point::new(4, 4),
        Point::new(26, 26),
        FillStyle::OutlineOnly,
        1,
        BLACK,
        WHITE,
    );
    draw_rectangle(
        &mut buffer,
        Point::new(32, 4),
        Point::new(54, 26),
        FillStyle::FilledWithOutline,
        2,
        DARK_RED,
        LIGHT_RED,
    );
    draw_rectangle(
        &mut buffer,
        Point::new(60, 4),
        Point::new(82, 26),
        FillStyle::FillOnly,
        1,
        BLUE,
        WHITE,
    );

    // 2. Rounded Rectangles
    draw_rounded_rectangle(
        &mut buffer,
        Point::new(90, 4),
        Point::new(114, 26),
        4,
        FillStyle::FilledWithOutline,
        1,
        BLACK,
        LIGHT_GREEN,
    );

    // 3. Ellipses
    draw_ellipse(
        &mut buffer,
        Point::new(4, 34),
        Point::new(26, 56),
        FillStyle::FilledWithOutline,
        2,
        PURPLE,
        LIGHT_YELLOW,
    );
    draw_ellipse(
        &mut buffer,
        Point::new(32, 34),
        Point::new(54, 56),
        FillStyle::FillOnly,
        1,
        GREEN,
        WHITE,
    );

    // 4. Polygon
    let poly_pts = vec![
        Point::new(75, 34),
        Point::new(85, 42),
        Point::new(80, 56),
        Point::new(65, 54),
        Point::new(60, 40),
    ];
    draw_polygon(
        &mut buffer,
        &poly_pts,
        FillStyle::FilledWithOutline,
        1,
        BLACK,
        LIGHT_BLUE,
    );

    // 5. Curves
    draw_curve_quadratic(
        &mut buffer,
        Point::new(4, 65),
        Point::new(30, 85),
        Point::new(55, 65),
        2,
        RED,
    );
    draw_curve_cubic(
        &mut buffer,
        Point::new(65, 80),
        Point::new(80, 60),
        Point::new(95, 90),
        Point::new(115, 65),
        2,
        BLUE,
    );

    assert_or_save_golden(&buffer, "golden_shapes_all_styles.png");
}

#[test]
fn test_golden_selection_move_copy_trail() {
    let mut buffer = PixelBuffer::new(80, 60, WHITE);

    // Draw initial sprite to select
    draw_rectangle(
        &mut buffer,
        Point::new(4, 4),
        Point::new(16, 16),
        FillStyle::FilledWithOutline,
        1,
        BLACK,
        RED,
    );
    draw_pencil_stroke(&mut buffer, Point::new(6, 6), Point::new(14, 14), WHITE);

    // Rectangular selection lift
    let sel = lift_rectangular_selection(&mut buffer, Point::new(4, 4), Point::new(16, 16), WHITE)
        .expect("lift failed");

    // 1. Normal copy at (25, 4)
    stamp_floating_selection(
        &mut buffer,
        &sel,
        Point::new(25, 4),
        SelectionMode::Opaque,
        WHITE,
    );

    // 2. Transparent mode copy at (45, 4) over blue background
    draw_rectangle(
        &mut buffer,
        Point::new(45, 4),
        Point::new(65, 20),
        FillStyle::FillOnly,
        1,
        BLUE,
        WHITE,
    );
    stamp_floating_selection(
        &mut buffer,
        &sel,
        Point::new(48, 5),
        SelectionMode::Transparent,
        WHITE,
    );

    // 3. Shift+drag trail from (4, 30) to (50, 45)
    for step in 0..6 {
        let x = 4 + step * 8;
        let y = 30 + step * 3;
        stamp_floating_selection(
            &mut buffer,
            &sel,
            Point::new(x, y),
            SelectionMode::Opaque,
            WHITE,
        );
    }

    // 4. Free-form lasso selection test
    let lasso_pts = vec![
        Point::new(45, 4),
        Point::new(60, 4),
        Point::new(65, 15),
        Point::new(50, 20),
    ];
    if let Some(lasso_sel) = lift_freeform_selection(&mut buffer, &lasso_pts, WHITE) {
        stamp_floating_selection(
            &mut buffer,
            &lasso_sel,
            Point::new(10, 42),
            SelectionMode::Opaque,
            WHITE,
        );
    }

    assert_or_save_golden(&buffer, "golden_selection_move_copy_trail.png");
}

#[test]
fn test_golden_text_render() {
    let mut buffer = PixelBuffer::new(120, 50, WHITE);

    // Opaque text
    draw_text_bitmap(
        &mut buffer,
        Point::new(4, 4),
        "XP Paint 5.1",
        1,
        SelectionMode::Opaque,
        BLACK,
        LIGHT_BLUE,
    );

    // Scaled text 2x
    draw_text_bitmap(
        &mut buffer,
        Point::new(4, 20),
        "RUST DX",
        2,
        SelectionMode::Transparent,
        DARK_RED,
        WHITE,
    );

    assert_or_save_golden(&buffer, "golden_text_render.png");
}

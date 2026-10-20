use paint_core::geometry::Point;
use paint_core::image::PixelBuffer;
use paint_core::palette::{BLACK, WHITE, XP_PALETTE_28};
use paint_core::tools::line::draw_line;
use paint_core::tools::pencil::draw_pencil_stroke;
use std::path::Path;

#[test]
fn test_golden_pencil_and_line_render() {
    let mut buffer = PixelBuffer::new(64, 64, WHITE);

    // Draw pencil strokes
    draw_pencil_stroke(
        &mut buffer,
        Point::new(2, 2),
        Point::new(30, 10),
        XP_PALETTE_28[2], // Dark Red
    );
    draw_pencil_stroke(
        &mut buffer,
        Point::new(30, 10),
        Point::new(60, 2),
        XP_PALETTE_28[4], // Dark Green
    );

    // Draw lines with different widths 1 to 5
    for w in 1..=5 {
        let y0 = 15 + (w as i32 - 1) * 9;
        let color = XP_PALETTE_28[w as usize + 5];
        draw_line(
            &mut buffer,
            Point::new(4, y0),
            Point::new(58, y0 + 4),
            w,
            color,
        );
    }

    // Draw black border
    draw_line(&mut buffer, Point::new(0, 0), Point::new(63, 0), 1, BLACK);
    draw_line(&mut buffer, Point::new(63, 0), Point::new(63, 63), 1, BLACK);
    draw_line(&mut buffer, Point::new(63, 63), Point::new(0, 63), 1, BLACK);
    draw_line(&mut buffer, Point::new(0, 63), Point::new(0, 0), 1, BLACK);

    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("golden");

    let golden_path = golden_dir.join("pencil_line_test.png");

    let current_image = buffer.to_rgba_image().expect("failed to get RgbaImage");

    if !golden_path.exists() {
        // Save initial golden file if it doesn't exist
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
            "Dimensions mismatch"
        );
        assert_eq!(
            current_image.into_raw(),
            golden_image.into_raw(),
            "Pixel mismatch with golden image!"
        );
    }
}

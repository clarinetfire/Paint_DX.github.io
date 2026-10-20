use paint_core::geometry::{Point, Rect};
use paint_core::image::PixelBuffer;
use paint_core::layers::LayerStack;
use paint_core::palette::{
    BLACK, BLUE, CYAN, DARK_BLUE, DARK_GREEN, DARK_RED, GREEN, LIGHT_BLUE, MAGENTA, PURPLE, RED,
    WHITE, YELLOW,
};
use paint_core::tools::brush::{draw_brush_stroke, BrushShape};
use paint_core::tools::eraser::{draw_eraser_stroke, EraserSize};
use paint_core::tools::line::draw_line;
use paint_core::tools::pencil::draw_pencil_stroke;
use paint_core::tools::selection::SelectionMode;
use paint_core::tools::shapes::{draw_ellipse, draw_polygon, draw_rectangle, FillStyle};
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
fn test_golden_5_layers_composite() {
    let width = 120;
    let height = 100;
    let mut stack = LayerStack::new(width, height, WHITE);

    // Layer 1: Background with decorative border and grid
    {
        let bg_layer = stack.active_layer_mut().unwrap();
        bg_layer.name = "Background".to_string();
        // Draw grid
        for x in (10..width - 10).step_by(10) {
            draw_line(
                &mut bg_layer.buffer,
                Point::new(x as i32, 5),
                Point::new(x as i32, height as i32 - 5),
                1,
                LIGHT_BLUE,
            );
        }
        for y in (10..height - 10).step_by(10) {
            draw_line(
                &mut bg_layer.buffer,
                Point::new(5, y as i32),
                Point::new(width as i32 - 5, y as i32),
                1,
                LIGHT_BLUE,
            );
        }
        // Border
        draw_rectangle(
            &mut bg_layer.buffer,
            Point::new(2, 2),
            Point::new(width as i32 - 3, height as i32 - 3),
            FillStyle::OutlineOnly,
            2,
            DARK_BLUE,
            WHITE,
        );
    }

    // Layer 2: Geometric shapes with 80% opacity
    let _l2_id = stack.add_layer("Shapes");
    {
        let l2 = stack.active_layer_mut().unwrap();
        l2.opacity = 80;
        // Red filled rectangle
        draw_rectangle(
            &mut l2.buffer,
            Point::new(15, 15),
            Point::new(55, 45),
            FillStyle::FilledWithOutline,
            2,
            DARK_RED,
            RED,
        );
        // Green ellipse
        draw_ellipse(
            &mut l2.buffer,
            Point::new(60, 20),
            Point::new(105, 55),
            FillStyle::FilledWithOutline,
            1,
            DARK_GREEN,
            GREEN,
        );
        // Yellow polygon
        let poly = vec![Point::new(30, 60), Point::new(50, 90), Point::new(10, 85)];
        draw_polygon(
            &mut l2.buffer,
            &poly,
            FillStyle::FilledWithOutline,
            1,
            DARK_RED,
            YELLOW,
        );
    }

    // Layer 3: Brush strokes & semi-transparent blending with 50% opacity
    let _l3_id = stack.add_layer("Brushwork");
    {
        let l3 = stack.active_layer_mut().unwrap();
        l3.opacity = 50;
        draw_brush_stroke(
            &mut l3.buffer,
            Point::new(10, 30),
            Point::new(110, 70),
            BrushShape::RoundLarge,
            PURPLE,
        );
        draw_brush_stroke(
            &mut l3.buffer,
            Point::new(110, 30),
            Point::new(10, 70),
            BrushShape::SquareLarge,
            MAGENTA,
        );
    }

    // Layer 4: Fine details and Text with 100% opacity
    let _l4_id = stack.add_layer("Text & Lines");
    {
        let l4 = stack.active_layer_mut().unwrap();
        l4.opacity = 100;
        draw_text_bitmap(
            &mut l4.buffer,
            Point::new(28, 42),
            "PAINT DX",
            1,
            SelectionMode::Transparent,
            BLACK,
            WHITE,
        );
        draw_pencil_stroke(
            &mut l4.buffer,
            Point::new(15, 80),
            Point::new(105, 80),
            BLACK,
        );
    }

    // Layer 5: Floating overlay with offset and transparent eraser cutouts (70% opacity)
    let _l5_id = stack.add_layer("Overlay");
    {
        let l5 = stack.active_layer_mut().unwrap();
        l5.opacity = 70;
        l5.offset_x = 5;
        l5.offset_y = 5;
        // Draw cyan circle
        draw_ellipse(
            &mut l5.buffer,
            Point::new(60, 50),
            Point::new(100, 90),
            FillStyle::FillOnly,
            1,
            CYAN,
            CYAN,
        );
        // Erase transparent cross through circle
        draw_eraser_stroke(
            &mut l5.buffer,
            Point::new(60, 70),
            Point::new(100, 70),
            EraserSize::Medium,
            paint_core::palette::TRANSPARENT,
        );
        draw_eraser_stroke(
            &mut l5.buffer,
            Point::new(80, 50),
            Point::new(80, 90),
            EraserSize::Medium,
            paint_core::palette::TRANSPARENT,
        );
    }

    assert_eq!(stack.layers.len(), 5);

    // Composite full image and verify against golden reference
    let composite = stack.composite();
    assert_or_save_golden(&composite, "golden_5_layers_composite.png");

    // Verify dirty rectangle compositing matches full composite
    let mut dirty_comp = PixelBuffer::new(width, height, paint_core::palette::TRANSPARENT);
    stack.composite_rect(
        &mut dirty_comp,
        Some(Rect::from_coords(0, 0, width as i32 - 1, height as i32 - 1)),
    );
    assert_eq!(composite, dirty_comp);

    // Verify dirty rectangle on a sub-quadrant
    let quadrant = Rect::from_coords(20, 20, 80, 80);
    let mut quadrant_target = composite.clone();
    stack.composite_rect(&mut quadrant_target, Some(quadrant));
    assert_eq!(composite, quadrant_target);
}

#[test]
fn test_layer_stack_operations_and_merge() {
    let width = 60;
    let height = 60;
    let mut stack = LayerStack::new(width, height, WHITE);

    let _l2 = stack.add_layer("Layer 2");
    {
        let l = stack.active_layer_mut().unwrap();
        draw_rectangle(
            &mut l.buffer,
            Point::new(10, 10),
            Point::new(40, 40),
            FillStyle::FillOnly,
            1,
            RED,
            RED,
        );
    }

    let l3 = stack.add_layer("Layer 3");
    {
        let l = stack.active_layer_mut().unwrap();
        draw_ellipse(
            &mut l.buffer,
            Point::new(20, 20),
            Point::new(50, 50),
            FillStyle::FillOnly,
            1,
            BLUE,
            BLUE,
        );
    }

    let before_merge = stack.composite();

    // Merge Layer 3 down onto Layer 2
    assert!(stack.merge_down(l3));
    assert_eq!(stack.layers.len(), 2);
    let after_merge = stack.composite();
    assert_eq!(before_merge, after_merge);

    // Flatten image
    stack.flatten(WHITE);
    assert_eq!(stack.layers.len(), 1);
    let after_flatten = stack.composite();
    assert_eq!(before_merge, after_flatten);
}

#[test]
fn test_large_canvas_stress_and_dirty_rect() {
    let width = 8000;
    let height = 8000;
    let mut stack = LayerStack::new(width, height, WHITE);

    let l2 = stack.add_layer("Large Overlay");
    {
        let l = stack.active_layer_mut().unwrap();
        // Draw in top-left corner
        draw_rectangle(
            &mut l.buffer,
            Point::new(100, 100),
            Point::new(300, 300),
            FillStyle::FillOnly,
            1,
            RED,
            RED,
        );
        // Draw in bottom-right corner
        draw_rectangle(
            &mut l.buffer,
            Point::new(7700, 7700),
            Point::new(7900, 7900),
            FillStyle::FillOnly,
            1,
            BLUE,
            BLUE,
        );
    }

    // Verify localized dirty rect compositing on large canvas
    let mut cached_comp = PixelBuffer::new(width, height, WHITE);
    stack.composite_rect(&mut cached_comp, Some(Rect::from_coords(90, 90, 310, 310)));
    stack.composite_rect(
        &mut cached_comp,
        Some(Rect::from_coords(7690, 7690, 7910, 7910)),
    );

    assert_eq!(cached_comp.width(), 8000);
    assert_eq!(cached_comp.height(), 8000);
    assert_eq!(cached_comp.get_pixel(200, 200), Some(RED));
    assert_eq!(cached_comp.get_pixel(7800, 7800), Some(BLUE));
    assert_eq!(cached_comp.get_pixel(4000, 4000), Some(WHITE));

    // Test layer toggle and re-composite
    stack.get_layer_mut(l2).unwrap().visible = false;
    let mut comp_hidden = PixelBuffer::new(width, height, WHITE);
    stack.composite_rect(&mut comp_hidden, Some(Rect::from_coords(90, 90, 310, 310)));
    assert_eq!(comp_hidden.get_pixel(200, 200), Some(WHITE));
    assert_eq!(comp_hidden.get_pixel(7800, 7800), Some(WHITE));
}

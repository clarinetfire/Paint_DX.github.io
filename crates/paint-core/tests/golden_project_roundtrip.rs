use paint_core::geometry::Point;
use paint_core::image::PixelBuffer;
use paint_core::layers::LayerStack;
use paint_core::palette::{
    ColorRgba, BLACK, BLUE, CYAN, DARK_BLUE, DARK_GREEN, DARK_RED, GREEN, LIGHT_BLUE, MAGENTA,
    PURPLE, RED, WHITE, XP_PALETTE_28, YELLOW,
};
use paint_core::project::{
    load_project_from_bytes, load_project_from_path, save_project_to_bytes, save_project_to_path,
    ProjectData, ProjectError,
};
use paint_core::tools::brush::{draw_brush_stroke, BrushShape};
use paint_core::tools::line::draw_line;
use paint_core::tools::pencil::draw_pencil_stroke;
use paint_core::tools::selection::SelectionMode;
use paint_core::tools::shapes::{draw_ellipse, draw_rectangle, FillStyle};
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
fn test_golden_5_layers_project_save_and_load_roundtrip() {
    let width = 120;
    let height = 100;

    // 1. Base Layer (Background) - White background with dark red border and grid lines
    let mut stack = LayerStack::new(width, height, WHITE);
    if let Some(layer) = stack.active_layer_mut() {
        layer.name = "Background Base".to_string();
        draw_rectangle(
            &mut layer.buffer,
            Point::new(2, 2),
            Point::new(117, 97),
            FillStyle::OutlineOnly,
            2,
            DARK_RED,
            WHITE,
        );
        draw_line(
            &mut layer.buffer,
            Point::new(10, 10),
            Point::new(110, 10),
            1,
            DARK_GREEN,
        );
        draw_line(
            &mut layer.buffer,
            Point::new(10, 90),
            Point::new(110, 90),
            1,
            DARK_GREEN,
        );
    }

    // 2. Layer 2: Shaded geometric shapes with 80% opacity
    let id2 = stack.add_layer("Geometric Shapes");
    if let Some(layer) = stack.get_layer_mut(id2) {
        layer.opacity = 80;
        draw_ellipse(
            &mut layer.buffer,
            Point::new(15, 20),
            Point::new(55, 60),
            FillStyle::FilledWithOutline,
            2,
            BLUE,
            CYAN,
        );
        draw_ellipse(
            &mut layer.buffer,
            Point::new(65, 20),
            Point::new(105, 60),
            FillStyle::FilledWithOutline,
            2,
            MAGENTA,
            YELLOW,
        );
    }

    // 3. Layer 3: Brush & Pencil art with offset
    let id3 = stack.add_layer("Organic Strokes");
    if let Some(layer) = stack.get_layer_mut(id3) {
        layer.opacity = 90;
        layer.offset_x = 2;
        layer.offset_y = -1;
        draw_brush_stroke(
            &mut layer.buffer,
            Point::new(20, 70),
            Point::new(100, 70),
            BrushShape::RoundLarge,
            PURPLE,
        );
        draw_pencil_stroke(
            &mut layer.buffer,
            Point::new(10, 30),
            Point::new(110, 45),
            RED,
        );
    }

    // 4. Layer 4: Text and details with 75% opacity
    let id4 = stack.add_layer("Text Overlay");
    if let Some(layer) = stack.get_layer_mut(id4) {
        layer.opacity = 75;
        draw_text_bitmap(
            &mut layer.buffer,
            Point::new(24, 40),
            "PAINT DX",
            1,
            SelectionMode::Transparent,
            BLACK,
            WHITE,
        );
        draw_rectangle(
            &mut layer.buffer,
            Point::new(35, 30),
            Point::new(95, 65),
            FillStyle::OutlineOnly,
            1,
            LIGHT_BLUE,
            GREEN,
        );
    }

    // 5. Layer 5: Top layer with locked status and crossing lines
    let id5 = stack.add_layer("Top Overlay");
    if let Some(layer) = stack.get_layer_mut(id5) {
        layer.opacity = 100;
        layer.locked = true;
        draw_line(
            &mut layer.buffer,
            Point::new(0, 0),
            Point::new(119, 99),
            2,
            DARK_RED,
        );
        draw_line(
            &mut layer.buffer,
            Point::new(0, 99),
            Point::new(119, 0),
            2,
            DARK_BLUE,
        );
    }

    // Set active layer to Organic Strokes
    stack.set_active_layer(id3);

    // Custom palette customization
    let mut custom_palette = XP_PALETTE_28;
    custom_palette[0] = ColorRgba::rgb(15, 30, 45);
    custom_palette[1] = ColorRgba::rgb(240, 128, 64);
    custom_palette[27] = ColorRgba::rgb(99, 199, 255);

    let original_project = ProjectData::from_layer_stack(stack.clone(), Some(custom_palette));
    let original_composite = stack.composite();

    // Verify golden image of original composite
    assert_or_save_golden(&original_composite, "project_5_layers_roundtrip.png");

    // Save project to memory
    let bytes =
        save_project_to_bytes(&original_project).expect("Failed to serialize project to bytes");
    assert!(!bytes.is_empty());

    // Save project to disk in temporary file
    let temp_dir = std::env::temp_dir();
    let temp_project_file = temp_dir.join("paint_dx_milestone6_test.paintproj");
    save_project_to_path(&original_project, &temp_project_file)
        .expect("Failed to save project to path");

    // Load project from bytes
    let loaded_from_memory =
        load_project_from_bytes(&bytes).expect("Failed to load project from bytes");

    // Load project from file
    let loaded_from_disk =
        load_project_from_path(&temp_project_file).expect("Failed to load project from disk");

    // Cleanup temp file
    let _ = std::fs::remove_file(temp_project_file);

    // Compare memory & disk loaded projects
    assert_eq!(loaded_from_memory.width, width);
    assert_eq!(loaded_from_memory.height, height);
    assert_eq!(loaded_from_memory.fps, 12);
    assert_eq!(loaded_from_memory.palette, Some(custom_palette));
    assert_eq!(loaded_from_memory.layer_stack.layers.len(), 5);
    assert_eq!(loaded_from_memory.layer_stack.active_layer_id, id3);

    assert_eq!(loaded_from_disk.width, width);
    assert_eq!(loaded_from_disk.height, height);
    assert_eq!(loaded_from_disk.palette, Some(custom_palette));
    assert_eq!(loaded_from_disk.layer_stack.layers.len(), 5);
    assert_eq!(loaded_from_disk.layer_stack.active_layer_id, id3);

    // Verify each layer's attributes and raw pixel data
    for i in 0..5 {
        let orig_l = &original_project.layer_stack.layers[i];
        let mem_l = &loaded_from_memory.layer_stack.layers[i];
        let disk_l = &loaded_from_disk.layer_stack.layers[i];

        assert_eq!(orig_l.id, mem_l.id);
        assert_eq!(orig_l.name, mem_l.name);
        assert_eq!(orig_l.opacity, mem_l.opacity);
        assert_eq!(orig_l.visible, mem_l.visible);
        assert_eq!(orig_l.locked, mem_l.locked);
        assert_eq!(orig_l.blend, mem_l.blend);
        assert_eq!(orig_l.offset_x, mem_l.offset_x);
        assert_eq!(orig_l.offset_y, mem_l.offset_y);
        assert_eq!(orig_l.buffer.as_raw(), mem_l.buffer.as_raw());

        assert_eq!(orig_l.id, disk_l.id);
        assert_eq!(orig_l.name, disk_l.name);
        assert_eq!(orig_l.opacity, disk_l.opacity);
        assert_eq!(orig_l.visible, disk_l.visible);
        assert_eq!(orig_l.locked, disk_l.locked);
        assert_eq!(orig_l.blend, disk_l.blend);
        assert_eq!(orig_l.offset_x, disk_l.offset_x);
        assert_eq!(orig_l.offset_y, disk_l.offset_y);
        assert_eq!(orig_l.buffer.as_raw(), disk_l.buffer.as_raw());
    }

    // Verify byte-identical composite rendering
    let loaded_mem_composite = loaded_from_memory.layer_stack.composite();
    let loaded_disk_composite = loaded_from_disk.layer_stack.composite();

    assert_eq!(
        original_composite.as_raw(),
        loaded_mem_composite.as_raw(),
        "Memory loaded project composite must be byte-identical to original composite!"
    );
    assert_eq!(
        original_composite.as_raw(),
        loaded_disk_composite.as_raw(),
        "Disk loaded project composite must be byte-identical to original composite!"
    );
}

#[test]
fn test_project_error_cases() {
    let empty_bytes = b"";
    assert!(matches!(
        load_project_from_bytes(empty_bytes),
        Err(ProjectError::Zip(_))
    ));

    let invalid_magic = b"PK\x03\x04\x00\x00\x00\x00corrupt";
    assert!(load_project_from_bytes(invalid_magic).is_err());
}

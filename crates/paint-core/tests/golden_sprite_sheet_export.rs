//! Golden integration tests for Milestone 9: Sprite Sheet Export.
//!
//! Validates Horizontal Strip, Vertical Strip, Grid, Packed (MaxRects),
//! Edge Extrusion, Power-of-Two (POT) rounding, and JSON metadata pixel coordinate matching.

use std::fs;
use std::path::PathBuf;

use paint_core::export::png_seq::{
    FrameRangeChoice, LayerExportScope, TransparencyOption, TrimMode,
};
use paint_core::export::sheet::{
    export_sprite_sheet_to_disk, generate_sprite_sheet, FrameOrdering, JsonSheetDoc,
    MetadataFormat, SheetLayout, SheetSizeMode, SpriteSheetOptions,
};
use paint_core::frames::{AnimTag, AnimationDocument, Cel, Frame, LayerMetadata};
use paint_core::io::load_from_path;
use paint_core::layers::LayerId;
use paint_core::palette::{BLACK, BLUE, GREEN, RED, WHITE};

fn setup_multi_frame_anim_doc() -> AnimationDocument {
    let mut doc = AnimationDocument::new_transparent(16, 16);
    let layer1 = LayerId(1);
    let layer2 = LayerId(2);
    doc.layers.push(LayerMetadata::new(layer2, "Foreground"));

    // Frame 0: Red 6x6 square on Layer 1 at (2, 2)
    let mut cel0_l1 = Cel::new(16, 16);
    for y in 2..8 {
        for x in 2..8 {
            cel0_l1.buffer.set_pixel(x, y, RED);
        }
    }
    doc.frames[0].set_cel(layer1, cel0_l1);

    // Frame 1: Blue 6x6 square on Layer 1 at (6, 6) + Green 2x2 dot on Layer 2 at (10, 10)
    let mut frame1 = Frame::new(100);
    let mut cel1_l1 = Cel::new(16, 16);
    for y in 6..12 {
        for x in 6..12 {
            cel1_l1.buffer.set_pixel(x, y, BLUE);
        }
    }
    frame1.set_cel(layer1, cel1_l1);

    let mut cel1_l2 = Cel::new(16, 16);
    for y in 10..12 {
        for x in 10..12 {
            cel1_l2.buffer.set_pixel(x, y, GREEN);
        }
    }
    frame1.set_cel(layer2, cel1_l2);
    doc.frames.push(frame1);

    // Frame 2: Black 4x4 square at (8, 8)
    let mut frame2 = Frame::new(140);
    let mut cel2 = Cel::new(16, 16);
    for y in 8..12 {
        for x in 8..12 {
            cel2.buffer.set_pixel(x, y, BLACK);
        }
    }
    frame2.set_cel(layer1, cel2);
    doc.frames.push(frame2);

    // Frame 3: White 2x2 dot at (0, 0)
    let mut frame3 = Frame::new(80);
    let mut cel3 = Cel::new(16, 16);
    for y in 0..2 {
        for x in 0..2 {
            cel3.buffer.set_pixel(x, y, WHITE);
        }
    }
    frame3.set_cel(layer1, cel3);
    doc.frames.push(frame3);

    doc.tags.push(AnimTag::new("walk", 0, 1));
    doc.tags.push(AnimTag::new("jump", 2, 3));
    doc
}

#[test]
fn test_sprite_sheet_metadata_coordinates_match_pixels() {
    let doc = setup_multi_frame_anim_doc();
    let out_dir = PathBuf::from("target/test_exports/sheet_meta_test");
    let _ = fs::remove_dir_all(&out_dir);

    let png_path = out_dir.join("hero_sheet.png");
    let json_path = out_dir.join("hero_sheet.json");

    let opts = SpriteSheetOptions {
        layout: SheetLayout::Grid,
        columns: 2,
        spacing: 4,
        border: 2,
        extrusion: 1, // 1px extrusion to test inner coordinate positioning
        size_mode: SheetSizeMode::Auto,
        frame_range: FrameRangeChoice::All,
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::KeepTransparency,
        scale: 1,
        trim: TrimMode::None,
        padding: 0,
        metadata_format: MetadataFormat::Json,
        ordering: FrameOrdering::ByFrameIndex,
    };

    let _result = export_sprite_sheet_to_disk(&doc, &opts, &png_path).expect("Export failed");
    assert!(png_path.exists());
    assert!(json_path.exists());

    let json_content = fs::read_to_string(&json_path).expect("Failed to read JSON");
    let sheet_doc: JsonSheetDoc = serde_json::from_str(&json_content).expect("Parse JSON");
    assert_eq!(sheet_doc.frames.len(), 4);

    let disk_img = load_from_path(&png_path).expect("Failed to load PNG");

    // CRITICAL MILESTONE 9 CRITERION:
    // Verify that every pixel inside the metadata rectangle matches the composited frame's pixels exactly!
    for f in &sheet_doc.frames {
        let expected_frame = doc.composite_frame(f.index);
        assert_eq!(f.w, expected_frame.width());
        assert_eq!(f.h, expected_frame.height());

        for dy in 0..f.h {
            for dx in 0..f.w {
                let sheet_pixel = disk_img
                    .get_pixel(f.x + dx, f.y + dy)
                    .expect("Sheet pixel must be in bounds");
                let expected_pixel = expected_frame
                    .get_pixel(dx, dy)
                    .expect("Frame pixel must be in bounds");
                assert_eq!(
                    sheet_pixel,
                    expected_pixel,
                    "Pixel mismatch at frame {} offset ({}, {}) -> sheet pos ({}, {})",
                    f.index,
                    dx,
                    dy,
                    f.x + dx,
                    f.y + dy
                );
            }
        }
    }

    // Also verify extrusion: the pixel at (f.x - 1, f.y - 1) must match (f.x, f.y)
    let f0 = &sheet_doc.frames[0];
    let top_left_inner = disk_img.get_pixel(f0.x, f0.y).unwrap();
    let top_left_extrude = disk_img.get_pixel(f0.x - 1, f0.y - 1).unwrap();
    assert_eq!(top_left_inner, top_left_extrude);

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn test_sprite_sheet_horizontal_and_vertical_strips() {
    let doc = setup_multi_frame_anim_doc();

    // 1. Horizontal strip
    let h_opts = SpriteSheetOptions {
        layout: SheetLayout::HorizontalStrip,
        spacing: 2,
        border: 1,
        extrusion: 0,
        size_mode: SheetSizeMode::Auto,
        ..Default::default()
    };
    let h_res = generate_sprite_sheet(&doc, &h_opts, "strip_h.png").unwrap();
    // 4 frames of 16px: width = 1 + 16 + 2 + 16 + 2 + 16 + 2 + 16 + 1 = 72; height = 16 + 2*1 = 18
    assert_eq!(h_res.sheet_width, 72);
    assert_eq!(h_res.sheet_height, 18);
    assert_eq!(h_res.frames[0].x, 1);
    assert_eq!(h_res.frames[1].x, 19);
    assert_eq!(h_res.frames[2].x, 37);
    assert_eq!(h_res.frames[3].x, 55);

    // 2. Vertical strip
    let v_opts = SpriteSheetOptions {
        layout: SheetLayout::VerticalStrip,
        spacing: 3,
        border: 2,
        extrusion: 0,
        size_mode: SheetSizeMode::Auto,
        ..Default::default()
    };
    let v_res = generate_sprite_sheet(&doc, &v_opts, "strip_v.png").unwrap();
    // 4 frames: width = 16 + 2*2 = 20; height = 2 + 16 + 3 + 16 + 3 + 16 + 3 + 16 + 2 = 77
    assert_eq!(v_res.sheet_width, 20);
    assert_eq!(v_res.sheet_height, 77);
    assert_eq!(v_res.frames[0].y, 2);
    assert_eq!(v_res.frames[1].y, 21);
    assert_eq!(v_res.frames[2].y, 40);
    assert_eq!(v_res.frames[3].y, 59);
}

#[test]
fn test_sprite_sheet_packed_layout_and_trim() {
    let doc = setup_multi_frame_anim_doc();

    let packed_opts = SpriteSheetOptions {
        layout: SheetLayout::Packed,
        spacing: 1,
        border: 1,
        extrusion: 0,
        trim: TrimMode::PerFrame, // Each frame trimmed to minimal content box
        size_mode: SheetSizeMode::Auto,
        ..Default::default()
    };

    let res = generate_sprite_sheet(&doc, &packed_opts, "packed.png").unwrap();
    assert_eq!(res.frames.len(), 4);

    // Verify all frames are within sheet bounds
    for f in &res.frames {
        assert!(f.x + f.w <= res.sheet_width);
        assert!(f.y + f.h <= res.sheet_height);
    }
}

#[test]
fn test_sprite_sheet_power_of_two_and_solid_background() {
    let doc = setup_multi_frame_anim_doc();

    let pot_opts = SpriteSheetOptions {
        layout: SheetLayout::Grid,
        columns: 2,
        size_mode: SheetSizeMode::PowerOfTwo,
        transparency: TransparencyOption::Flatten(WHITE),
        ..Default::default()
    };

    let res = generate_sprite_sheet(&doc, &pot_opts, "pot.png").unwrap();
    // 2x2 grid of 16x16 is 32x32 (already a power of two)
    assert_eq!(res.sheet_width, 32);
    assert_eq!(res.sheet_height, 32);
    assert!(res.sheet_width.is_power_of_two());
    assert!(res.sheet_height.is_power_of_two());

    // Background must be solid white (alpha 255) everywhere outside content
    let p_empty = res.sheet_buffer.get_pixel(0, 0).unwrap();
    assert_eq!(p_empty, WHITE);
    assert_eq!(p_empty.a, 255);
}

#[test]
fn test_sprite_sheet_csv_export() {
    let doc = setup_multi_frame_anim_doc();
    let out_dir = PathBuf::from("target/test_exports/sheet_csv_test");
    let _ = fs::remove_dir_all(&out_dir);

    let png_path = out_dir.join("anim.png");
    let csv_path = out_dir.join("anim.csv");

    let opts = SpriteSheetOptions {
        layout: SheetLayout::HorizontalStrip,
        metadata_format: MetadataFormat::Csv,
        ..Default::default()
    };

    let _res = export_sprite_sheet_to_disk(&doc, &opts, &png_path).expect("Export failed");
    assert!(csv_path.exists());

    let csv_content = fs::read_to_string(&csv_path).unwrap();
    assert!(csv_content
        .starts_with("index,x,y,w,h,duration_ms,offset_x,offset_y,source_w,source_h,tag\n"));
    let lines: Vec<&str> = csv_content.trim().lines().collect();
    assert_eq!(lines.len(), 5); // 1 header + 4 frames

    let _ = fs::remove_dir_all(&out_dir);
}

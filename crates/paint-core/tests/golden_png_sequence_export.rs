//! Integration and Golden Tests for Milestone 8: PNG Sequence Export.

use std::fs;
use std::path::PathBuf;

use paint_core::export::png_seq::{
    export_png_sequence, ExistingFilesPolicy, FrameRangeChoice, LayerExportScope,
    PngSequenceExportOptions, TransparencyOption, TrimMode,
};
use paint_core::frames::{AnimTag, AnimationDocument, Cel, Frame, LayerMetadata};
use paint_core::io::load_from_path;
use paint_core::layers::LayerId;
use paint_core::palette::{ColorRgba, BLACK, BLUE, GREEN, RED};

fn get_golden_dir() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("golden")
}

fn setup_test_doc() -> AnimationDocument {
    let mut doc = AnimationDocument::new_transparent(32, 32);
    let layer1 = LayerId(1);
    let layer2 = LayerId(2);
    doc.layers.push(LayerMetadata::new(layer2, "Foreground"));

    // Frame 0: Red 8x8 block on Layer 1 at (4, 4)
    let mut cel0_l1 = Cel::new(32, 32);
    for y in 4..12 {
        for x in 4..12 {
            cel0_l1.buffer.set_pixel(x, y, RED);
        }
    }
    doc.frames[0].set_cel(layer1, cel0_l1);

    // Frame 1: Blue 8x8 block on Layer 1 at (12, 12) + Green 4x4 dot on Layer 2 at (20, 20)
    let mut frame1 = Frame::new(120);
    let mut cel1_l1 = Cel::new(32, 32);
    for y in 12..20 {
        for x in 12..20 {
            cel1_l1.buffer.set_pixel(x, y, BLUE);
        }
    }
    frame1.set_cel(layer1, cel1_l1);

    let mut cel1_l2 = Cel::new(32, 32);
    for y in 20..24 {
        for x in 20..24 {
            cel1_l2.buffer.set_pixel(x, y, GREEN);
        }
    }
    frame1.set_cel(layer2, cel1_l2);
    doc.frames.push(frame1);

    // Frame 2: Black 4x4 block at (24, 24)
    let mut frame2 = Frame::new(150);
    let mut cel2_l1 = Cel::new(32, 32);
    for y in 24..28 {
        for x in 24..28 {
            cel2_l1.buffer.set_pixel(x, y, BLACK);
        }
    }
    frame2.set_cel(layer1, cel2_l1);
    doc.frames.push(frame2);

    doc.tags.push(AnimTag::new("intro", 0, 1));
    doc
}

#[test]
fn test_png_sequence_transparent_export_alpha_zero() {
    let doc = setup_test_doc();
    let out_dir = PathBuf::from("target/test_exports/png_seq_transparent");
    let _ = fs::remove_dir_all(&out_dir);

    let opts = PngSequenceExportOptions {
        output_dir: out_dir.clone(),
        pattern: "walk_".to_string(),
        start_index: 0,
        padding_digits: 4,
        frame_range: FrameRangeChoice::All,
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::KeepTransparency,
        scale: 1,
        trim: TrimMode::None,
        padding: 0,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    let result = export_png_sequence(&doc, &opts, None, |_, _| true).expect("Export failed");
    assert_eq!(result.exported_files.len(), 3);

    // 1. Check frame 0
    let f0_path = out_dir.join("walk_0000.png");
    assert!(f0_path.exists());
    let img0 = load_from_path(&f0_path).expect("Failed to load exported PNG");
    assert_eq!(img0.width(), 32);
    assert_eq!(img0.height(), 32);

    // Empty pixel must have alpha 0
    assert_eq!(img0.get_pixel(0, 0), Some(ColorRgba::TRANSPARENT));
    assert_eq!(img0.get_pixel(0, 0).unwrap().a, 0);
    // Drawn pixel must have alpha 255 and color RED
    assert_eq!(img0.get_pixel(6, 6), Some(RED));
    assert_eq!(img0.get_pixel(6, 6).unwrap().a, 255);

    // 2. Check frame 1
    let f1_path = out_dir.join("walk_0001.png");
    assert!(f1_path.exists());
    let img1 = load_from_path(&f1_path).expect("Failed to load exported PNG");
    assert_eq!(img1.get_pixel(0, 0), Some(ColorRgba::TRANSPARENT));
    assert_eq!(img1.get_pixel(15, 15), Some(BLUE));
    assert_eq!(img1.get_pixel(22, 22), Some(GREEN));

    // 3. Check frame 2
    let f2_path = out_dir.join("walk_0002.png");
    assert!(f2_path.exists());
    let img2 = load_from_path(&f2_path).expect("Failed to load exported PNG");
    assert_eq!(img2.get_pixel(0, 0), Some(ColorRgba::TRANSPARENT));
    assert_eq!(img2.get_pixel(25, 25), Some(BLACK));

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn test_png_sequence_flattened_export_opaque_alpha() {
    let doc = setup_test_doc();
    let out_dir = PathBuf::from("target/test_exports/png_seq_flattened");
    let _ = fs::remove_dir_all(&out_dir);

    let yellow_bg = ColorRgba::rgb(255, 255, 0);

    let opts = PngSequenceExportOptions {
        output_dir: out_dir.clone(),
        pattern: "flat_".to_string(),
        start_index: 1,
        padding_digits: 2,
        frame_range: FrameRangeChoice::All,
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::Flatten(yellow_bg),
        scale: 1,
        trim: TrimMode::None,
        padding: 0,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    let result = export_png_sequence(&doc, &opts, None, |_, _| true).expect("Export failed");
    assert_eq!(result.exported_files.len(), 3);

    let f0_path = out_dir.join("flat_01.png");
    let img0 = load_from_path(&f0_path).expect("Failed to load exported PNG");

    // Check every single pixel to ensure alpha is 255 everywhere
    for y in 0..img0.height() {
        for x in 0..img0.width() {
            let px = img0.get_pixel(x, y).unwrap();
            assert_eq!(px.a, 255, "Pixel at ({}, {}) has alpha != 255", x, y);
        }
    }

    // Empty pixel should now be the yellow background
    assert_eq!(img0.get_pixel(0, 0), Some(yellow_bg));
    // Red square preserved
    assert_eq!(img0.get_pixel(6, 6), Some(RED));

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn test_png_sequence_nearest_neighbor_scaling() {
    let doc = setup_test_doc();
    let out_dir = PathBuf::from("target/test_exports/png_seq_scale");
    let _ = fs::remove_dir_all(&out_dir);

    let opts = PngSequenceExportOptions {
        output_dir: out_dir.clone(),
        pattern: "scale4x_".to_string(),
        start_index: 0,
        padding_digits: 3,
        frame_range: FrameRangeChoice::ActiveFrame,
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::KeepTransparency,
        scale: 4,
        trim: TrimMode::None,
        padding: 0,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    let result = export_png_sequence(&doc, &opts, None, |_, _| true).expect("Export failed");
    assert_eq!(result.exported_files.len(), 1);
    assert_eq!(result.exported_files[0].width, 128);
    assert_eq!(result.exported_files[0].height, 128);

    let img = load_from_path(&result.exported_files[0].file_path).unwrap();
    assert_eq!(img.width(), 128);
    assert_eq!(img.height(), 128);

    // In 4x scaled image, the red block at (4..12) becomes (16..48)
    assert_eq!(img.get_pixel(15, 15), Some(ColorRgba::TRANSPARENT));
    assert_eq!(img.get_pixel(16, 16), Some(RED));
    assert_eq!(img.get_pixel(47, 47), Some(RED));
    assert_eq!(img.get_pixel(48, 48), Some(ColorRgba::TRANSPARENT));

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn test_png_sequence_trim_modes_and_offsets() {
    let doc = setup_test_doc();
    let out_dir = PathBuf::from("target/test_exports/png_seq_trims");
    let _ = fs::remove_dir_all(&out_dir);

    // 1. Per-frame trim
    let opts_per_frame = PngSequenceExportOptions {
        output_dir: out_dir.join("per_frame"),
        pattern: "pf_".to_string(),
        start_index: 0,
        padding_digits: 2,
        frame_range: FrameRangeChoice::All,
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::KeepTransparency,
        scale: 1,
        trim: TrimMode::PerFrame,
        padding: 2,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    let pf_res = export_png_sequence(&doc, &opts_per_frame, None, |_, _| true).unwrap();
    // Frame 0 had 8x8 block + 2px padding each side -> 12x12
    assert_eq!(pf_res.exported_files[0].width, 12);
    assert_eq!(pf_res.exported_files[0].height, 12);
    assert_eq!(pf_res.exported_files[0].offset_x, 4);
    assert_eq!(pf_res.exported_files[0].offset_y, 4);

    // 2. Union trim
    let opts_union = PngSequenceExportOptions {
        output_dir: out_dir.join("union"),
        pattern: "union_".to_string(),
        start_index: 0,
        padding_digits: 2,
        frame_range: FrameRangeChoice::All,
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::KeepTransparency,
        scale: 1,
        trim: TrimMode::Union,
        padding: 0,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    let u_res = export_png_sequence(&doc, &opts_union, None, |_, _| true).unwrap();
    // Union covers (4, 4) to (27, 27) -> size 24x24
    for f in &u_res.exported_files {
        assert_eq!(f.width, 24);
        assert_eq!(f.height, 24);
        assert_eq!(f.offset_x, 4);
        assert_eq!(f.offset_y, 4);
    }

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn test_png_sequence_layer_scope_and_tags() {
    let doc = setup_test_doc();
    let out_dir = PathBuf::from("target/test_exports/png_seq_layers_tags");
    let _ = fs::remove_dir_all(&out_dir);

    // 1. Tag selection ("intro" = frames 0 and 1)
    let opts_tag = PngSequenceExportOptions {
        output_dir: out_dir.join("tag"),
        pattern: "intro_".to_string(),
        start_index: 0,
        padding_digits: 2,
        frame_range: FrameRangeChoice::Tag("intro".to_string()),
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::KeepTransparency,
        scale: 1,
        trim: TrimMode::None,
        padding: 0,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    let tag_res = export_png_sequence(&doc, &opts_tag, None, |_, _| true).unwrap();
    assert_eq!(tag_res.exported_files.len(), 2);

    // 2. One sequence per layer
    let opts_per_layer = PngSequenceExportOptions {
        output_dir: out_dir.join("per_layer"),
        pattern: "seq_".to_string(),
        start_index: 0,
        padding_digits: 2,
        frame_range: FrameRangeChoice::All,
        layers: LayerExportScope::OneSequencePerLayer,
        transparency: TransparencyOption::KeepTransparency,
        scale: 1,
        trim: TrimMode::None,
        padding: 0,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    let pl_res = export_png_sequence(&doc, &opts_per_layer, None, |_, _| true).unwrap();
    // 3 frames * 2 layers = 6 files
    assert_eq!(pl_res.exported_files.len(), 6);

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn test_png_sequence_collision_policies() {
    let doc = setup_test_doc();
    let out_dir = PathBuf::from("target/test_exports/png_seq_collisions");
    let _ = fs::remove_dir_all(&out_dir);

    let mut opts = PngSequenceExportOptions {
        output_dir: out_dir.clone(),
        pattern: "col_".to_string(),
        start_index: 0,
        padding_digits: 2,
        frame_range: FrameRangeChoice::ActiveFrame,
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::KeepTransparency,
        scale: 1,
        trim: TrimMode::None,
        padding: 0,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    // First export creates col_00.png
    let res1 = export_png_sequence(&doc, &opts, None, |_, _| true).unwrap();
    assert_eq!(res1.exported_files.len(), 1);
    assert!(out_dir.join("col_00.png").exists());

    // Skip policy
    opts.existing_files = ExistingFilesPolicy::Skip;
    let res_skip = export_png_sequence(&doc, &opts, None, |_, _| true).unwrap();
    assert_eq!(res_skip.exported_files.len(), 0);
    assert_eq!(res_skip.skipped_files.len(), 1);

    // AutoRename policy
    opts.existing_files = ExistingFilesPolicy::AutoRename;
    let res_rename = export_png_sequence(&doc, &opts, None, |_, _| true).unwrap();
    assert_eq!(res_rename.exported_files.len(), 1);
    assert!(out_dir.join("col_00_1.png").exists());

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn test_golden_png_sequence_non_regression() {
    let doc = setup_test_doc();
    let out_dir = PathBuf::from("target/test_exports/png_seq_golden");
    let _ = fs::remove_dir_all(&out_dir);

    let opts = PngSequenceExportOptions {
        output_dir: out_dir.clone(),
        pattern: "golden_seq_".to_string(),
        start_index: 0,
        padding_digits: 4,
        frame_range: FrameRangeChoice::All,
        layers: LayerExportScope::VisibleComposite,
        transparency: TransparencyOption::KeepTransparency,
        scale: 1,
        trim: TrimMode::None,
        padding: 0,
        existing_files: ExistingFilesPolicy::Overwrite,
    };

    let result = export_png_sequence(&doc, &opts, None, |_, _| true).expect("Export failed");
    assert_eq!(result.exported_files.len(), 3);

    let golden_path = get_golden_dir().join("golden_png_sequence_frame_0.png");
    let frame0_buf = load_from_path(&result.exported_files[0].file_path).unwrap();

    if !golden_path.exists() {
        if let Some(parent) = golden_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        paint_core::io::save_to_path(
            &frame0_buf,
            &golden_path,
            paint_core::io::ImageFileFormat::Png,
        )
        .expect("Failed to save initial golden PNG");
    }

    let golden_buf = load_from_path(&golden_path).expect("Failed to load golden PNG");
    assert_eq!(
        frame0_buf, golden_buf,
        "Exported PNG sequence frame 0 did not match golden reference!"
    );

    let _ = fs::remove_dir_all(&out_dir);
}

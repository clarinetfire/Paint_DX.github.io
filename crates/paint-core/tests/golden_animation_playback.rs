use paint_core::frames::{AnimTag, AnimationDocument};
use paint_core::geometry::Point;
use paint_core::image::PixelBuffer;
use paint_core::onion_skin::{composite_with_onion_skin, OnionSkinConfig, OnionSkinTint};
use paint_core::palette::{BLACK, BLUE, DARK_BLUE, GREEN, RED, WHITE, YELLOW};
use paint_core::project::{load_project_from_bytes, save_project_to_bytes, ProjectData};
use paint_core::tools::shapes::{draw_ellipse, draw_rectangle, FillStyle};
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
fn test_4_frame_animation_playback_and_durations() {
    let width = 100;
    let height = 60;

    // Create 4-frame animation document with custom durations
    let mut doc = AnimationDocument::new(width, height, WHITE);
    let l_anim = doc.add_layer("Bouncing Ball");
    let l_overlay = doc.add_layer("Static Frame Overlay");

    // Frame 0: 100ms
    doc.set_frame_duration(0, 100);

    // Frame 1: 150ms
    let f1 = doc.add_frame(150, false);
    assert_eq!(f1, 1);

    // Frame 2: 200ms
    let f2 = doc.add_frame(200, false);
    assert_eq!(f2, 2);

    // Frame 3: 250ms
    let f3 = doc.add_frame(250, false);
    assert_eq!(f3, 3);

    assert_eq!(doc.frames.len(), 4);
    assert_eq!(doc.total_duration_ms(), 700);

    // Draw static overlay on all frames
    for f in 0..4 {
        if let Some(cel) = doc.get_cel_mut(f, l_overlay) {
            draw_rectangle(
                &mut cel.buffer,
                Point::new(1, 1),
                Point::new(98, 58),
                FillStyle::OutlineOnly,
                1,
                DARK_BLUE,
                WHITE,
            );
        }
    }

    // Draw moving ball on l_anim layer across 4 frames
    let ball_positions = [
        Point::new(15, 30),
        Point::new(35, 15),
        Point::new(60, 15),
        Point::new(80, 30),
    ];
    let ball_colors = [RED, YELLOW, GREEN, BLUE];

    for (f, (&pos, &color)) in ball_positions.iter().zip(ball_colors.iter()).enumerate() {
        if let Some(cel) = doc.get_cel_mut(f, l_anim) {
            draw_ellipse(
                &mut cel.buffer,
                Point::new(pos.x - 8, pos.y - 8),
                Point::new(pos.x + 8, pos.y + 8),
                FillStyle::FillOnly,
                1,
                BLACK,
                color,
            );
        }
    }

    // 1. Verify playback timing calculation
    assert_eq!(doc.frame_index_at_time(0, true), 0);
    assert_eq!(doc.frame_index_at_time(50, true), 0);
    assert_eq!(doc.frame_index_at_time(100, true), 1);
    assert_eq!(doc.frame_index_at_time(200, true), 1);
    assert_eq!(doc.frame_index_at_time(250, true), 2);
    assert_eq!(doc.frame_index_at_time(400, true), 2);
    assert_eq!(doc.frame_index_at_time(450, true), 3);
    assert_eq!(doc.frame_index_at_time(699, true), 3);
    assert_eq!(doc.frame_index_at_time(700, true), 0); // Looped back to frame 0
    assert_eq!(doc.frame_index_at_time(850, true), 1); // 850 % 700 = 150 -> frame 1

    // 2. Verify Animation Tags
    let walk_tag = AnimTag::new("Bounce", 0, 3).with_color(255, 128, 0);
    let apex_tag = AnimTag::new("Apex", 1, 2).with_color(0, 200, 255);
    doc.add_tag(walk_tag);
    doc.add_tag(apex_tag);

    assert_eq!(doc.tags.len(), 2);
    assert!(doc.tags[0].contains_frame(0));
    assert!(doc.tags[0].contains_frame(3));
    assert!(!doc.tags[1].contains_frame(0));
    assert!(doc.tags[1].contains_frame(1));
    assert!(doc.tags[1].contains_frame(2));

    // 3. Verify frame compositing
    let comp0 = doc.composite_frame(0);
    let comp2 = doc.composite_frame(2);
    assert_or_save_golden(&comp0, "anim_4_frames_frame_0.png");
    assert_or_save_golden(&comp2, "anim_4_frames_frame_2.png");

    // 4. Verify Onion Skinning on Frame 2 (with prev=1, next=1, RedBlue tint)
    let onion_cfg = OnionSkinConfig {
        enabled: true,
        prev_frames: 1,
        next_frames: 1,
        opacity: 50,
        tint_mode: OnionSkinTint::RedBlue,
        loop_bounds: false,
    };
    let onion_comp = composite_with_onion_skin(&doc, 2, &onion_cfg);
    assert_or_save_golden(&onion_comp, "anim_4_frames_onion_skin.png");

    // 5. Verify Project Serialization & Roundtrip
    let project = ProjectData::from_doc(doc.clone(), None);
    let bytes = save_project_to_bytes(&project).expect("Failed to serialize 4-frame project");
    assert!(!bytes.is_empty());

    let loaded = load_project_from_bytes(&bytes).expect("Failed to deserialize 4-frame project");
    assert_eq!(loaded.doc.frames.len(), 4);
    assert_eq!(loaded.doc.frames[0].duration_ms, 100);
    assert_eq!(loaded.doc.frames[1].duration_ms, 150);
    assert_eq!(loaded.doc.frames[2].duration_ms, 200);
    assert_eq!(loaded.doc.frames[3].duration_ms, 250);
    assert_eq!(loaded.doc.layers.len(), 3);
    assert_eq!(loaded.doc.tags.len(), 2);
    assert_eq!(loaded.doc.tags[0].name, "Bounce");
    assert_eq!(loaded.doc.tags[1].name, "Apex");

    let loaded_comp0 = loaded.doc.composite_frame(0);
    assert_eq!(
        comp0.as_raw(),
        loaded_comp0.as_raw(),
        "Loaded frame 0 composite must be byte-identical!"
    );

    let loaded_comp2 = loaded.doc.composite_frame(2);
    assert_eq!(
        comp2.as_raw(),
        loaded_comp2.as_raw(),
        "Loaded frame 2 composite must be byte-identical!"
    );
}

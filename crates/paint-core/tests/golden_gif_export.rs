use paint_core::export::gif::{export_gif_to_memory, GifExportOptions, GifRepeatOption};
use paint_core::export::png_seq::{FrameRangeChoice, TransparencyOption};
use paint_core::frames::AnimationDocument;
use paint_core::geometry::Point;
use paint_core::palette::{BLACK, BLUE, GREEN, RED, WHITE, YELLOW};
use paint_core::tools::shapes::{draw_ellipse, FillStyle};
use std::io::Cursor;

#[test]
fn test_golden_gif_export_animation() {
    let width = 64;
    let height = 64;

    let mut doc = AnimationDocument::new(width, height, WHITE);
    let ball_layer = doc.add_layer("Ball");

    // Create 4 frames with ball moving in circle
    let positions = [
        Point::new(16, 16),
        Point::new(48, 16),
        Point::new(48, 48),
        Point::new(16, 48),
    ];
    let colors = [RED, YELLOW, GREEN, BLUE];

    // Frame 0
    if let Some(cel) = doc.get_cel_mut(0, ball_layer) {
        draw_ellipse(
            &mut cel.buffer,
            Point::new(positions[0].x - 10, positions[0].y - 10),
            Point::new(positions[0].x + 10, positions[0].y + 10),
            FillStyle::FillOnly,
            1,
            BLACK,
            colors[0],
        );
    }

    // Frames 1..4
    for i in 1..4 {
        doc.add_frame(100, false);
        if let Some(cel) = doc.get_cel_mut(i, ball_layer) {
            draw_ellipse(
                &mut cel.buffer,
                Point::new(positions[i].x - 10, positions[i].y - 10),
                Point::new(positions[i].x + 10, positions[i].y + 10),
                FillStyle::FillOnly,
                1,
                BLACK,
                colors[i],
            );
        }
    }

    let opts = GifExportOptions {
        frame_range: FrameRangeChoice::All,
        transparency: TransparencyOption::Flatten(WHITE),
        fps_override: Some(10),
        scale: 1,
        repeat: GifRepeatOption::Infinite,
        ..Default::default()
    };

    let gif_bytes = export_gif_to_memory(&doc, &opts).expect("export gif bytes");
    assert!(!gif_bytes.is_empty());
    assert_eq!(&gif_bytes[0..3], b"GIF");

    // Decode and verify frames
    let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(&gif_bytes)).expect("decode gif");
    use image::AnimationDecoder;
    let frames = decoder
        .into_frames()
        .collect_frames()
        .expect("collect frames");
    assert_eq!(frames.len(), 4);
    assert_eq!(frames[0].buffer().width(), 64);
    assert_eq!(frames[0].buffer().height(), 64);
}

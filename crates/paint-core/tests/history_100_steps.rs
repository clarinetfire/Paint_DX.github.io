use paint_core::geometry::Point;
use paint_core::history::History;
use paint_core::image::PixelBuffer;
use paint_core::palette::{ColorRgba, BLACK, BLUE, GREEN, RED, WHITE};
use paint_core::tools::fill::flood_fill;
use paint_core::tools::line::draw_line;
use paint_core::tools::pencil::draw_pencil_stroke;
use paint_core::tools::shapes::{draw_ellipse, draw_rectangle, FillStyle};

#[test]
fn test_100_undo_redo_steps_round_trip_without_pixel_drift() {
    let mut history = History::new(150);
    let mut current = PixelBuffer::new(64, 64, WHITE);

    let mut snapshots = Vec::with_capacity(101);
    snapshots.push(current.clone()); // snapshot 0

    // Perform 100 diverse modifying drawing and editing operations
    let mut step_count = 0;
    let mut i = 1;
    while step_count < 100 {
        let before = current.clone();
        let color = match (i + step_count) % 5 {
            0 => RED,
            1 => GREEN,
            2 => BLUE,
            3 => BLACK,
            _ => ColorRgba::rgba(255, 128, 0, 200),
        };

        match step_count % 5 {
            0 => {
                // Pencil stroke
                let p0 = Point::new((i * 3 % 60) as i32, (i * 7 % 60) as i32);
                let p1 = Point::new(((i * 3 + 10) % 60) as i32, ((i * 7 + 10) % 60) as i32);
                draw_pencil_stroke(&mut current, p0, p1, color);
            }
            1 => {
                // Line
                let p0 = Point::new((i % 50) as i32, ((i * 2) % 50) as i32);
                let p1 = Point::new(((i + 20) % 60) as i32, ((i * 2 + 15) % 60) as i32);
                draw_line(&mut current, p0, p1, (i % 3 + 1) as u32, color);
            }
            2 => {
                // Rectangle
                let p0 = Point::new((i % 40) as i32, ((i * 3) % 40) as i32);
                let p1 = Point::new(((i % 40) + 12) as i32, (((i * 3) % 40) + 12) as i32);
                draw_rectangle(
                    &mut current,
                    p0,
                    p1,
                    FillStyle::FilledWithOutline,
                    1,
                    color,
                    WHITE,
                );
            }
            3 => {
                // Ellipse
                let p0 = Point::new((i % 35) as i32, ((i * 4) % 35) as i32);
                let p1 = Point::new(((i % 35) + 15) as i32, (((i * 4) % 35) + 15) as i32);
                draw_ellipse(
                    &mut current,
                    p0,
                    p1,
                    FillStyle::OutlineOnly,
                    2,
                    color,
                    WHITE,
                );
            }
            _ => {
                // Eraser or fill with distinct color
                let pt = Point::new((i % 60) as i32, ((i * 5) % 60) as i32);
                flood_fill(&mut current, pt, color);
            }
        }

        if history.record_change(&before, &current) {
            snapshots.push(current.clone());
            step_count += 1;
        }
        i += 1;
    }

    assert_eq!(snapshots.len(), 101);
    assert_eq!(history.undo_count(), 100);

    // Verify Undo from step 100 back to 0
    for step in (1..=100).rev() {
        assert!(history.undo(&mut current), "Failed undo at step {}", step);
        let expected = &snapshots[step - 1];
        assert_eq!(
            current.as_raw(),
            expected.as_raw(),
            "Pixel drift detected at undo step {}",
            step
        );
    }

    // Now current should be identical to the initial canvas (snapshot 0)
    assert_eq!(current.as_raw(), snapshots[0].as_raw());
    assert!(!history.can_undo());
    assert_eq!(history.redo_count(), 100);

    // Verify Redo from step 1 to 100
    for step in 1..=100 {
        assert!(history.redo(&mut current), "Failed redo at step {}", step);
        let expected = &snapshots[step];
        assert_eq!(
            current.as_raw(),
            expected.as_raw(),
            "Pixel drift detected at redo step {}",
            step
        );
    }

    // Now current should be identical to the final canvas (snapshot 100)
    assert_eq!(current.as_raw(), snapshots[100].as_raw());
    assert!(!history.can_redo());
}

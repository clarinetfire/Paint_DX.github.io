//! Geometric primitives and raster algorithms for pixel-exact canvas operations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn from_points(p0: Point, p1: Point) -> Self {
        Self {
            min: Point::new(p0.x.min(p1.x), p0.y.min(p1.y)),
            max: Point::new(p0.x.max(p1.x), p0.y.max(p1.y)),
        }
    }

    pub fn from_coords(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self::from_points(Point::new(x0, y0), Point::new(x1, y1))
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn width(&self) -> u32 {
        (self.max.x - self.min.x + 1).max(0) as u32
    }

    pub fn height(&self) -> u32 {
        (self.max.y - self.min.y + 1).max(0) as u32
    }
}

/// Computes all integer coordinates along a line from `p0` to `p1` using the classic Bresenham algorithm.
pub fn bresenham_line(p0: Point, p1: Point) -> Vec<Point> {
    let mut points = Vec::new();
    let mut x0 = p0.x;
    let mut y0 = p0.y;
    let x1 = p1.x;
    let y1 = p1.y;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push(Point::new(x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    points
}

/// Snaps `current` point relative to `start` to the nearest 45-degree angle (0°, 45°, 90°, 135°, etc.).
/// This matches Windows XP Paint's Shift-constrained line behavior.
pub fn snap_angle_45(start: Point, current: Point) -> Point {
    let dx = current.x - start.x;
    let dy = current.y - start.y;

    if dx == 0 && dy == 0 {
        return current;
    }

    let abs_dx = dx.abs();
    let abs_dy = dy.abs();

    if abs_dy * 1000 <= abs_dx * 414 {
        // Horizontal line
        Point::new(current.x, start.y)
    } else if abs_dx * 1000 <= abs_dy * 414 {
        // Vertical line
        Point::new(start.x, current.y)
    } else {
        // 45-degree diagonal
        let d = (abs_dx + abs_dy) / 2;
        let sign_x = if dx >= 0 { 1 } else { -1 };
        let sign_y = if dy >= 0 { 1 } else { -1 };
        Point::new(start.x + sign_x * d, start.y + sign_y * d)
    }
}

/// Snaps `current` point relative to `start` to form a 1:1 aspect ratio square/circle.
/// Used when holding Shift with Rectangle, Rounded Rectangle, and Ellipse tools.
pub fn snap_to_square(start: Point, current: Point) -> Point {
    let dx = current.x - start.x;
    let dy = current.y - start.y;
    let side = dx.abs().max(dy.abs());
    let sign_x = if dx >= 0 { 1 } else { -1 };
    let sign_y = if dy >= 0 { 1 } else { -1 };
    Point::new(start.x + sign_x * side, start.y + sign_y * side)
}

/// Rasterizes an axis-aligned ellipse bounded by `p0` and `p1`.
/// Returns `(outline_points, fill_spans)` where each span is `(y, x_min, x_max)`.
pub fn rasterize_ellipse(p0: Point, p1: Point) -> (Vec<Point>, Vec<(i32, i32, i32)>) {
    let min_x = p0.x.min(p1.x);
    let max_x = p0.x.max(p1.x);
    let min_y = p0.y.min(p1.y);
    let max_y = p0.y.max(p1.y);

    let width = max_x - min_x;
    let height = max_y - min_y;

    if width == 0 || height == 0 {
        let pts = bresenham_line(Point::new(min_x, min_y), Point::new(max_x, max_y));
        let mut spans = Vec::new();
        for p in &pts {
            spans.push((p.y, p.x, p.x));
        }
        return (pts, spans);
    }

    let a = width as f64 / 2.0;
    let b = height as f64 / 2.0;
    let xc = min_x as f64 + a;
    let yc = min_y as f64 + b;

    let mut spans_map: std::collections::BTreeMap<i32, (i32, i32)> =
        std::collections::BTreeMap::new();
    let mut outline = Vec::new();

    // Step around parameter theta with high enough resolution
    let steps = ((width + height) * 4).max(32);
    let mut last_pt: Option<Point> = None;

    for i in 0..=steps {
        let theta = 2.0 * std::f64::consts::PI * (i as f64) / (steps as f64);
        let px = (xc + a * theta.cos()).round() as i32;
        let py = (yc + b * theta.sin()).round() as i32;
        let pt = Point::new(px, py);

        if let Some(prev) = last_pt {
            for line_pt in bresenham_line(prev, pt) {
                outline.push(line_pt);
                let entry = spans_map.entry(line_pt.y).or_insert((line_pt.x, line_pt.x));
                entry.0 = entry.0.min(line_pt.x);
                entry.1 = entry.1.max(line_pt.x);
            }
        }
        last_pt = Some(pt);
    }

    // Deduplicate outline
    let mut dedup_outline = Vec::with_capacity(outline.len());
    let mut seen = std::collections::HashSet::new();
    for p in outline {
        if seen.insert(p) {
            dedup_outline.push(p);
        }
    }

    let spans = spans_map
        .into_iter()
        .map(|(y, (x0, x1))| (y, x0, x1))
        .collect();

    (dedup_outline, spans)
}

/// Rasterizes a quadratic Bezier curve with 1 control point (p0 = start, p1 = control, p2 = end).
pub fn rasterize_bezier_quadratic(p0: Point, p1: Point, p2: Point) -> Vec<Point> {
    let approx_len =
        ((p1.x - p0.x).abs() + (p1.y - p0.y).abs() + (p2.x - p1.x).abs() + (p2.y - p1.y).abs())
            as usize;
    let steps = approx_len.clamp(16, 500);

    let mut points = Vec::new();
    let mut prev = p0;

    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let u = 1.0 - t;
        let x =
            (u * u * p0.x as f64 + 2.0 * u * t * p1.x as f64 + t * t * p2.x as f64).round() as i32;
        let y =
            (u * u * p0.y as f64 + 2.0 * u * t * p1.y as f64 + t * t * p2.y as f64).round() as i32;
        let curr = Point::new(x, y);

        for pt in bresenham_line(prev, curr) {
            if points.last() != Some(&pt) {
                points.push(pt);
            }
        }
        prev = curr;
    }

    points
}

/// Rasterizes a cubic Bezier curve with 2 control points.
pub fn rasterize_bezier_cubic(p0: Point, p1: Point, p2: Point, p3: Point) -> Vec<Point> {
    let approx_len = ((p1.x - p0.x).abs()
        + (p1.y - p0.y).abs()
        + (p2.x - p1.x).abs()
        + (p2.y - p1.y).abs()
        + (p3.x - p2.x).abs()
        + (p3.y - p2.y).abs()) as usize;
    let steps = approx_len.clamp(20, 1000);

    let mut points = Vec::new();
    let mut prev = p0;

    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        let x = (uuu * p0.x as f64
            + 3.0 * uu * t * p1.x as f64
            + 3.0 * u * tt * p2.x as f64
            + ttt * p3.x as f64)
            .round() as i32;
        let y = (uuu * p0.y as f64
            + 3.0 * uu * t * p1.y as f64
            + 3.0 * u * tt * p2.y as f64
            + ttt * p3.y as f64)
            .round() as i32;
        let curr = Point::new(x, y);

        for pt in bresenham_line(prev, curr) {
            if points.last() != Some(&pt) {
                points.push(pt);
            }
        }
        prev = curr;
    }

    points
}

/// Rasterizes a closed polygon into horizontal fill spans using the scanline algorithm.
pub fn rasterize_polygon_scanline(vertices: &[Point]) -> Vec<(i32, i32, i32)> {
    if vertices.len() < 3 {
        return Vec::new();
    }

    let min_y = vertices.iter().map(|p| p.y).min().unwrap_or(0);
    let max_y = vertices.iter().map(|p| p.y).max().unwrap_or(0);

    let mut spans = Vec::new();

    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        for i in 0..vertices.len() {
            let p1 = vertices[i];
            let p2 = vertices[(i + 1) % vertices.len()];

            if (p1.y <= y && p2.y > y) || (p2.y <= y && p1.y > y) {
                let dy = p2.y - p1.y;
                let dx = p2.x - p1.x;
                let x = p1.x as f64 + ((y - p1.y) as f64 / dy as f64) * dx as f64;
                intersections.push(x.round() as i32);
            }
        }

        intersections.sort_unstable();

        for chunk in intersections.chunks(2) {
            if chunk.len() == 2 {
                spans.push((y, chunk[0], chunk[1]));
            }
        }
    }

    spans
}

/// Rasterizes a rounded rectangle. Returns `(outline_points, fill_spans)`.
pub fn rasterize_rounded_rectangle(
    p0: Point,
    p1: Point,
    radius: i32,
) -> (Vec<Point>, Vec<(i32, i32, i32)>) {
    let min_x = p0.x.min(p1.x);
    let max_x = p0.x.max(p1.x);
    let min_y = p0.y.min(p1.y);
    let max_y = p0.y.max(p1.y);

    let w = max_x - min_x;
    let h = max_y - min_y;

    let r = radius.clamp(0, (w.min(h) / 2).max(0));
    if r == 0 {
        let mut outline = Vec::new();
        for pt in bresenham_line(Point::new(min_x, min_y), Point::new(max_x, min_y)) {
            outline.push(pt);
        }
        for pt in bresenham_line(Point::new(max_x, min_y), Point::new(max_x, max_y)) {
            outline.push(pt);
        }
        for pt in bresenham_line(Point::new(max_x, max_y), Point::new(min_x, max_y)) {
            outline.push(pt);
        }
        for pt in bresenham_line(Point::new(min_x, max_y), Point::new(min_x, min_y)) {
            outline.push(pt);
        }
        let mut spans = Vec::new();
        for y in min_y..=max_y {
            spans.push((y, min_x, max_x));
        }
        return (outline, spans);
    }

    let mut spans_map: std::collections::BTreeMap<i32, (i32, i32)> =
        std::collections::BTreeMap::new();
    let mut outline = Vec::new();

    // Helper to add lines to outline and spans
    let mut add_seg = |start: Point, end: Point| {
        for pt in bresenham_line(start, end) {
            outline.push(pt);
            let entry = spans_map.entry(pt.y).or_insert((pt.x, pt.x));
            entry.0 = entry.0.min(pt.x);
            entry.1 = entry.1.max(pt.x);
        }
    };

    // Straight edges
    add_seg(Point::new(min_x + r, min_y), Point::new(max_x - r, min_y));
    add_seg(Point::new(max_x, min_y + r), Point::new(max_x, max_y - r));
    add_seg(Point::new(max_x - r, max_y), Point::new(min_x + r, max_y));
    add_seg(Point::new(min_x, max_y - r), Point::new(min_x, min_y + r));

    // 4 rounded corners using arc points
    let corners = [
        (
            min_x + r,
            min_y + r,
            std::f64::consts::PI,
            1.5 * std::f64::consts::PI,
        ), // top-left
        (
            max_x - r,
            min_y + r,
            1.5 * std::f64::consts::PI,
            2.0 * std::f64::consts::PI,
        ), // top-right
        (max_x - r, max_y - r, 0.0, 0.5 * std::f64::consts::PI), // bottom-right
        (
            min_x + r,
            max_y - r,
            0.5 * std::f64::consts::PI,
            std::f64::consts::PI,
        ), // bottom-left
    ];

    let arc_steps = (r * 4).max(8);
    for &(cx, cy, a_start, a_end) in &corners {
        let mut prev_pt: Option<Point> = None;
        for i in 0..=arc_steps {
            let angle = a_start + (a_end - a_start) * (i as f64 / arc_steps as f64);
            let px = (cx as f64 + r as f64 * angle.cos()).round() as i32;
            let py = (cy as f64 + r as f64 * angle.sin()).round() as i32;
            let curr = Point::new(px, py);
            if let Some(prev) = prev_pt {
                add_seg(prev, curr);
            }
            prev_pt = Some(curr);
        }
    }

    // Deduplicate outline
    let mut dedup_outline = Vec::with_capacity(outline.len());
    let mut seen = std::collections::HashSet::new();
    for p in outline {
        if seen.insert(p) {
            dedup_outline.push(p);
        }
    }

    // Ensure all internal scanlines between min_y and max_y are populated
    for y in (min_y + r)..=(max_y - r) {
        let entry = spans_map.entry(y).or_insert((min_x, max_x));
        entry.0 = entry.0.min(min_x);
        entry.1 = entry.1.max(max_x);
    }

    let spans = spans_map
        .into_iter()
        .map(|(y, (x0, x1))| (y, x0, x1))
        .collect();

    (dedup_outline, spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bresenham_horizontal_and_vertical() {
        let h = bresenham_line(Point::new(0, 0), Point::new(3, 0));
        assert_eq!(
            h,
            vec![
                Point::new(0, 0),
                Point::new(1, 0),
                Point::new(2, 0),
                Point::new(3, 0)
            ]
        );

        let v = bresenham_line(Point::new(1, 1), Point::new(1, 3));
        assert_eq!(
            v,
            vec![Point::new(1, 1), Point::new(1, 2), Point::new(1, 3)]
        );
    }

    #[test]
    fn test_bresenham_diagonal() {
        let d = bresenham_line(Point::new(0, 0), Point::new(2, 2));
        assert_eq!(
            d,
            vec![Point::new(0, 0), Point::new(1, 1), Point::new(2, 2)]
        );
    }

    #[test]
    fn test_snap_angle_45() {
        let start = Point::new(10, 10);
        assert_eq!(snap_angle_45(start, Point::new(20, 11)), Point::new(20, 10));
        assert_eq!(snap_angle_45(start, Point::new(11, 20)), Point::new(10, 20));
        assert_eq!(snap_angle_45(start, Point::new(20, 19)), Point::new(19, 19));
    }

    #[test]
    fn test_rasterize_ellipse() {
        let (outline, spans) = rasterize_ellipse(Point::new(0, 0), Point::new(10, 10));
        assert!(!outline.is_empty());
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_rasterize_bezier() {
        let pts =
            rasterize_bezier_quadratic(Point::new(0, 0), Point::new(10, 20), Point::new(20, 0));
        assert!(!pts.is_empty());
        assert_eq!(pts.first(), Some(&Point::new(0, 0)));
        assert_eq!(pts.last(), Some(&Point::new(20, 0)));
    }

    #[test]
    fn test_rasterize_polygon_scanline() {
        let verts = vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ];
        let spans = rasterize_polygon_scanline(&verts);
        assert_eq!(spans.len(), 10);
    }
}

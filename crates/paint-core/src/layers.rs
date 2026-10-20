//! Layer data models, layer stack management, and compositing.

use crate::geometry::Rect;
use crate::image::PixelBuffer;
use crate::palette::ColorRgba;
use crate::transform::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: u8, // 0..=100
    pub blend: BlendMode,
    pub buffer: PixelBuffer,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl Layer {
    pub fn new(
        id: LayerId,
        name: impl Into<String>,
        width: u32,
        height: u32,
        fill: ColorRgba,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            locked: false,
            opacity: 100,
            blend: BlendMode::Normal,
            buffer: PixelBuffer::new(width, height, fill),
            offset_x: 0,
            offset_y: 0,
        }
    }

    pub fn new_transparent(id: LayerId, name: impl Into<String>, width: u32, height: u32) -> Self {
        Self::new(id, name, width, height, ColorRgba::TRANSPARENT)
    }

    pub fn width(&self) -> u32 {
        self.buffer.width()
    }

    pub fn height(&self) -> u32 {
        self.buffer.height()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerStack {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>, // Ordered bottom (index 0) to top (index len - 1)
    pub active_layer_id: LayerId,
    pub next_layer_id: u64,
}

impl LayerStack {
    pub fn new(width: u32, height: u32, background_color: ColorRgba) -> Self {
        let initial_layer = Layer::new(LayerId(1), "Background", width, height, background_color);
        Self {
            width,
            height,
            layers: vec![initial_layer],
            active_layer_id: LayerId(1),
            next_layer_id: 2,
        }
    }

    pub fn new_transparent(width: u32, height: u32) -> Self {
        Self::new(width, height, ColorRgba::TRANSPARENT)
    }

    pub fn from_raw_layers(
        width: u32,
        height: u32,
        layers: Vec<Layer>,
        active_layer_id: LayerId,
    ) -> Self {
        let max_id = layers.iter().map(|l| l.id.0).max().unwrap_or(0);
        let actual_active = if layers.iter().any(|l| l.id == active_layer_id) {
            active_layer_id
        } else {
            layers.first().map(|l| l.id).unwrap_or(LayerId(1))
        };
        Self {
            width,
            height,
            layers,
            active_layer_id: actual_active,
            next_layer_id: max_id + 1,
        }
    }

    pub fn active_layer(&self) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == self.active_layer_id)
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        let id = self.active_layer_id;
        self.layers.iter_mut().find(|l| l.id == id)
    }

    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    pub fn active_index(&self) -> Option<usize> {
        self.layers
            .iter()
            .position(|l| l.id == self.active_layer_id)
    }

    pub fn set_active_layer(&mut self, id: LayerId) -> bool {
        if self.layers.iter().any(|l| l.id == id) {
            self.active_layer_id = id;
            true
        } else {
            false
        }
    }

    pub fn set_active_index(&mut self, index: usize) -> bool {
        if index < self.layers.len() {
            self.active_layer_id = self.layers[index].id;
            true
        } else {
            false
        }
    }

    pub fn add_layer(&mut self, name: impl Into<String>) -> LayerId {
        let id = LayerId(self.next_layer_id);
        self.next_layer_id += 1;
        let new_layer = Layer::new_transparent(id, name, self.width, self.height);

        let insert_idx = match self.active_index() {
            Some(idx) => idx + 1,
            None => self.layers.len(),
        };
        self.layers.insert(insert_idx, new_layer);
        self.active_layer_id = id;
        id
    }

    pub fn insert_layer(&mut self, index: usize, layer: Layer) {
        let clamped_idx = index.min(self.layers.len());
        if layer.id.0 >= self.next_layer_id {
            self.next_layer_id = layer.id.0 + 1;
        }
        self.active_layer_id = layer.id;
        self.layers.insert(clamped_idx, layer);
    }

    pub fn delete_layer(&mut self, id: LayerId) -> bool {
        if self.layers.len() <= 1 {
            return false;
        }
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            self.layers.remove(pos);
            if self.active_layer_id == id {
                let new_pos = pos.min(self.layers.len() - 1);
                self.active_layer_id = self.layers[new_pos].id;
            }
            true
        } else {
            false
        }
    }

    pub fn delete_active_layer(&mut self) -> bool {
        self.delete_layer(self.active_layer_id)
    }

    pub fn duplicate_layer(&mut self, id: LayerId) -> Option<LayerId> {
        let pos = self.layers.iter().position(|l| l.id == id)?;
        let src = &self.layers[pos];
        let new_id = LayerId(self.next_layer_id);
        self.next_layer_id += 1;
        let mut dup = src.clone();
        dup.id = new_id;
        dup.name = format!("{} copy", src.name);
        self.layers.insert(pos + 1, dup);
        self.active_layer_id = new_id;
        Some(new_id)
    }

    pub fn move_up(&mut self, id: LayerId) -> bool {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            if pos + 1 < self.layers.len() {
                self.layers.swap(pos, pos + 1);
                return true;
            }
        }
        false
    }

    pub fn move_down(&mut self, id: LayerId) -> bool {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            if pos > 0 {
                self.layers.swap(pos, pos - 1);
                return true;
            }
        }
        false
    }

    pub fn reorder_layer(&mut self, from_idx: usize, to_idx: usize) -> bool {
        if from_idx < self.layers.len() && to_idx < self.layers.len() && from_idx != to_idx {
            let layer = self.layers.remove(from_idx);
            self.layers.insert(to_idx, layer);
            true
        } else {
            false
        }
    }

    pub fn merge_down(&mut self, id: LayerId) -> bool {
        let pos = match self.layers.iter().position(|l| l.id == id) {
            Some(p) => p,
            None => return false,
        };
        if pos == 0 {
            return false;
        }

        let upper = self.layers.remove(pos);
        let lower = &mut self.layers[pos - 1];

        // Blend upper layer onto lower layer
        blend_layer_into(&upper, &mut lower.buffer);
        self.active_layer_id = lower.id;
        true
    }

    pub fn merge_visible(&mut self) -> bool {
        if self.layers.len() <= 1 {
            return false;
        }

        let mut merged_buf = PixelBuffer::new(self.width, self.height, ColorRgba::TRANSPARENT);
        let mut visible_indices = Vec::new();
        for (i, layer) in self.layers.iter().enumerate() {
            if layer.visible {
                visible_indices.push(i);
                blend_layer_into(layer, &mut merged_buf);
            }
        }

        if visible_indices.is_empty() {
            return false;
        }

        let first_idx = visible_indices[0];
        // Remove from highest to lowest index
        for &idx in visible_indices.iter().rev() {
            self.layers.remove(idx);
        }

        let new_id = LayerId(self.next_layer_id);
        self.next_layer_id += 1;
        let merged_layer = Layer {
            id: new_id,
            name: "Merged".to_string(),
            visible: true,
            locked: false,
            opacity: 100,
            blend: BlendMode::Normal,
            buffer: merged_buf,
            offset_x: 0,
            offset_y: 0,
        };

        let insert_idx = first_idx.min(self.layers.len());
        self.layers.insert(insert_idx, merged_layer);
        self.active_layer_id = new_id;
        true
    }

    pub fn flatten(&mut self, background_color: ColorRgba) {
        let mut composite_buf = PixelBuffer::new(self.width, self.height, background_color);
        for layer in &self.layers {
            if layer.visible {
                blend_layer_into(layer, &mut composite_buf);
            }
        }

        let new_id = LayerId(self.next_layer_id);
        self.next_layer_id += 1;
        let flattened_layer = Layer {
            id: new_id,
            name: "Background".to_string(),
            visible: true,
            locked: false,
            opacity: 100,
            blend: BlendMode::Normal,
            buffer: composite_buf,
            offset_x: 0,
            offset_y: 0,
        };

        self.layers = vec![flattened_layer];
        self.active_layer_id = new_id;
    }

    pub fn resize(&mut self, new_w: u32, new_h: u32, fill: ColorRgba) {
        self.width = new_w;
        self.height = new_h;
        for (i, layer) in self.layers.iter_mut().enumerate() {
            let layer_fill = if i == 0 { fill } else { ColorRgba::TRANSPARENT };
            layer.buffer = resize_canvas(&layer.buffer, new_w, new_h, layer_fill);
        }
    }

    pub fn flip_horizontal(&mut self, all_layers: bool) {
        if all_layers {
            for layer in &mut self.layers {
                layer.buffer = flip_horizontal(&layer.buffer);
            }
        } else if let Some(layer) = self.active_layer_mut() {
            layer.buffer = flip_horizontal(&layer.buffer);
        }
    }

    pub fn flip_vertical(&mut self, all_layers: bool) {
        if all_layers {
            for layer in &mut self.layers {
                layer.buffer = flip_vertical(&layer.buffer);
            }
        } else if let Some(layer) = self.active_layer_mut() {
            layer.buffer = flip_vertical(&layer.buffer);
        }
    }

    pub fn rotate_90(&mut self, all_layers: bool) {
        let new_w = self.height;
        let new_h = self.width;
        if all_layers {
            self.width = new_w;
            self.height = new_h;
            for layer in &mut self.layers {
                layer.buffer = rotate_90(&layer.buffer);
            }
        } else if let Some(layer) = self.active_layer_mut() {
            layer.buffer = rotate_90(&layer.buffer);
        }
    }

    pub fn rotate_180(&mut self, all_layers: bool) {
        if all_layers {
            for layer in &mut self.layers {
                layer.buffer = rotate_180(&layer.buffer);
            }
        } else if let Some(layer) = self.active_layer_mut() {
            layer.buffer = rotate_180(&layer.buffer);
        }
    }

    pub fn rotate_270(&mut self, all_layers: bool) {
        let new_w = self.height;
        let new_h = self.width;
        if all_layers {
            self.width = new_w;
            self.height = new_h;
            for layer in &mut self.layers {
                layer.buffer = rotate_270(&layer.buffer);
            }
        } else if let Some(layer) = self.active_layer_mut() {
            layer.buffer = rotate_270(&layer.buffer);
        }
    }

    pub fn invert_colors(&mut self, all_layers: bool) {
        if all_layers {
            for layer in &mut self.layers {
                invert_colors(&mut layer.buffer);
            }
        } else if let Some(layer) = self.active_layer_mut() {
            invert_colors(&mut layer.buffer);
        }
    }

    pub fn stretch(&mut self, pct_x: u32, pct_y: u32, all_layers: bool) {
        if all_layers {
            let new_w = ((self.width as u64 * pct_x as u64) / 100).max(1) as u32;
            let new_h = ((self.height as u64 * pct_y as u64) / 100).max(1) as u32;
            self.width = new_w;
            self.height = new_h;
            for layer in &mut self.layers {
                layer.buffer = stretch(&layer.buffer, pct_x, pct_y);
            }
        } else if let Some(layer) = self.active_layer_mut() {
            layer.buffer = stretch(&layer.buffer, pct_x, pct_y);
        }
    }

    pub fn skew(&mut self, deg_x: i32, deg_y: i32, bg: ColorRgba, all_layers: bool) {
        if all_layers {
            for layer in &mut self.layers {
                layer.buffer = skew(&layer.buffer, deg_x, deg_y, bg);
            }
            if let Some(first) = self.layers.first() {
                self.width = first.buffer.width();
                self.height = first.buffer.height();
            }
        } else if let Some(layer) = self.active_layer_mut() {
            layer.buffer = skew(&layer.buffer, deg_x, deg_y, bg);
        }
    }

    /// Full compositing of all visible layers into a new PixelBuffer.
    pub fn composite(&self) -> PixelBuffer {
        let mut target = PixelBuffer::new(self.width, self.height, ColorRgba::TRANSPARENT);
        self.composite_rect(&mut target, None);
        target
    }

    /// Composites all visible layers into `target`, optionally restricted to `dirty_rect`.
    pub fn composite_rect(&self, target: &mut PixelBuffer, dirty_rect: Option<Rect>) {
        let (min_x, min_y, max_x, max_y) = match dirty_rect {
            Some(r) => {
                let rect = Rect::from_points(r.min, r.max);
                let x0 = rect.min.x.clamp(0, self.width as i32) as u32;
                let y0 = rect.min.y.clamp(0, self.height as i32) as u32;
                let x1 = (rect.max.x + 1).clamp(0, self.width as i32) as u32;
                let y1 = (rect.max.y + 1).clamp(0, self.height as i32) as u32;
                (x0, y0, x1, y1)
            }
            None => (0, 0, self.width, self.height),
        };

        if min_x >= max_x || min_y >= max_y {
            return;
        }

        // Clear dirty sub-region in target to fully transparent
        for y in min_y..max_y {
            for x in min_x..max_x {
                target.set_pixel(x, y, ColorRgba::TRANSPARENT);
            }
        }

        // Blend each visible layer within the dirty region
        for layer in &self.layers {
            if !layer.visible || layer.opacity == 0 {
                continue;
            }
            blend_layer_subregion(layer, target, min_x, min_y, max_x, max_y);
        }
    }
}

/// Blends an entire layer into `dst_buf`.
fn blend_layer_into(layer: &Layer, dst_buf: &mut PixelBuffer) {
    let w = dst_buf.width();
    let h = dst_buf.height();
    blend_layer_subregion(layer, dst_buf, 0, 0, w, h);
}

/// Blends `layer` into `dst_buf` within target rectangle `[min_x..max_x, min_y..max_y]`.
fn blend_layer_subregion(
    layer: &Layer,
    dst_buf: &mut PixelBuffer,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
) {
    let opacity = layer.opacity.min(100) as u32;
    if opacity == 0 {
        return;
    }
    let src_buf = &layer.buffer;
    let ox = layer.offset_x;
    let oy = layer.offset_y;

    let dst_y_start = (oy.max(min_y as i32)).max(0) as u32;
    let dst_y_end = ((oy + src_buf.height() as i32).min(max_y as i32)).max(0) as u32;
    let dst_x_start = (ox.max(min_x as i32)).max(0) as u32;
    let dst_x_end = ((ox + src_buf.width() as i32).min(max_x as i32)).max(0) as u32;

    if dst_x_start >= dst_x_end || dst_y_start >= dst_y_end {
        return;
    }

    for y in dst_y_start..dst_y_end {
        let src_y = (y as i32 - oy) as u32;
        for x in dst_x_start..dst_x_end {
            let src_x = (x as i32 - ox) as u32;
            if let Some(src_px) = src_buf.get_pixel(src_x, src_y) {
                if src_px.a == 0 {
                    continue;
                }

                // Scale source alpha by layer opacity
                let effective_a = ((src_px.a as u32 * opacity + 50) / 100).min(255) as u8;
                if effective_a == 0 {
                    continue;
                }

                let scaled_src = ColorRgba::rgba(src_px.r, src_px.g, src_px.b, effective_a);
                dst_buf.blend_pixel(x, y, scaled_src);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{RED, WHITE};

    #[test]
    fn test_layer_stack_basic_ops() {
        let mut stack = LayerStack::new(10, 10, WHITE);
        assert_eq!(stack.layers.len(), 1);
        assert_eq!(stack.active_layer_id, LayerId(1));

        let id2 = stack.add_layer("Layer 2");
        assert_eq!(stack.layers.len(), 2);
        assert_eq!(stack.active_layer_id, id2);

        let id3 = stack.add_layer("Layer 3");
        assert_eq!(stack.layers.len(), 3);
        assert_eq!(stack.active_layer_id, id3);

        // Move down
        assert!(stack.move_down(id3));
        assert_eq!(stack.layers[1].id, id3);

        // Duplicate
        let dup_id = stack.duplicate_layer(id3).unwrap();
        assert_eq!(stack.layers.len(), 4);
        assert_eq!(stack.active_layer_id, dup_id);

        // Delete
        assert!(stack.delete_layer(dup_id));
        assert_eq!(stack.layers.len(), 3);

        // Merge down
        assert!(stack.merge_down(id3));
        assert_eq!(stack.layers.len(), 2);
    }

    #[test]
    fn test_compositing_with_opacity_and_dirty_rect() {
        let mut stack = LayerStack::new(4, 4, WHITE);
        let _id2 = stack.add_layer("Layer 2");

        // Draw red pixel on layer 2 with 50% opacity
        let layer2 = stack.active_layer_mut().unwrap();
        layer2.opacity = 50;
        layer2.buffer.set_pixel(0, 0, RED);

        let comp1 = stack.composite();
        let p = comp1.get_pixel(0, 0).unwrap();
        // White (255, 255, 255) + 50% Red (255, 0, 0, 128) -> (255, 128, 128)
        assert_eq!(p.r, 255);
        assert!((p.g as i32 - 128).abs() <= 2);
        assert!((p.b as i32 - 128).abs() <= 2);
        assert_eq!(p.a, 255);

        // Test dirty rectangle compositing produces identical result
        let mut comp2 = PixelBuffer::new(4, 4, ColorRgba::TRANSPARENT);
        stack.composite_rect(&mut comp2, Some(Rect::from_coords(0, 0, 3, 3)));
        assert_eq!(comp1, comp2);
    }
}

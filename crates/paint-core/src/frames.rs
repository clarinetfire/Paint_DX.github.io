//! Frame and Cel animation data models and document management.

use crate::image::PixelBuffer;
use crate::layers::{BlendMode, Layer, LayerId, LayerStack};
use crate::palette::ColorRgba;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A Cel represents pixel data for a specific layer on a specific frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cel {
    pub x: i32,
    pub y: i32,
    pub buffer: PixelBuffer,
}

impl Cel {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            buffer: PixelBuffer::new(width, height, ColorRgba::TRANSPARENT),
        }
    }

    pub fn new_with_color(width: u32, height: u32, color: ColorRgba) -> Self {
        Self {
            x: 0,
            y: 0,
            buffer: PixelBuffer::new(width, height, color),
        }
    }

    pub fn from_buffer(x: i32, y: i32, buffer: PixelBuffer) -> Self {
        Self { x, y, buffer }
    }

    pub fn is_empty(&self) -> bool {
        for chunk in self.buffer.as_raw().chunks_exact(4) {
            if chunk[3] > 0 {
                return false;
            }
        }
        true
    }

    pub fn clear(&mut self) {
        self.buffer.clear(ColorRgba::TRANSPARENT);
    }
}

/// A Frame in the animation sequence, containing duration in milliseconds and layer cels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub duration_ms: u32,
    pub cels: HashMap<LayerId, Cel>,
}

impl Frame {
    pub fn new(duration_ms: u32) -> Self {
        Self {
            duration_ms: duration_ms.max(1),
            cels: HashMap::new(),
        }
    }

    pub fn get_cel(&self, layer_id: LayerId) -> Option<&Cel> {
        self.cels.get(&layer_id)
    }

    pub fn get_cel_mut(&mut self, layer_id: LayerId) -> Option<&mut Cel> {
        self.cels.get_mut(&layer_id)
    }

    pub fn ensure_cel(&mut self, layer_id: LayerId, width: u32, height: u32) -> &mut Cel {
        self.cels
            .entry(layer_id)
            .or_insert_with(|| Cel::new(width, height))
    }

    pub fn set_cel(&mut self, layer_id: LayerId, cel: Cel) {
        self.cels.insert(layer_id, cel);
    }

    pub fn remove_cel(&mut self, layer_id: LayerId) -> Option<Cel> {
        self.cels.remove(&layer_id)
    }
}

/// An AnimTag represents a named range of animation frames (e.g. "walk", "idle", "attack").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimTag {
    pub name: String,
    pub from_frame: usize,
    pub to_frame: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 3]>,
}

impl AnimTag {
    pub fn new(name: impl Into<String>, from_frame: usize, to_frame: usize) -> Self {
        Self {
            name: name.into(),
            from_frame: from_frame.min(to_frame),
            to_frame: from_frame.max(to_frame),
            color: None,
        }
    }

    pub fn with_color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = Some([r, g, b]);
        self
    }

    pub fn contains_frame(&self, frame_idx: usize) -> bool {
        frame_idx >= self.from_frame && frame_idx <= self.to_frame
    }

    pub fn frame_count(&self) -> usize {
        self.to_frame.saturating_sub(self.from_frame) + 1
    }
}

/// Shared metadata properties of a layer across all frames.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: u8, // 0..=100
    pub blend: BlendMode,
}

impl LayerMetadata {
    pub fn new(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            locked: false,
            opacity: 100,
            blend: BlendMode::Normal,
        }
    }
}

/// Complete Animation Document containing layers metadata, frames, cels, tags, and playback configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimationDocument {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<LayerMetadata>, // Bottom to top
    pub frames: Vec<Frame>,
    pub tags: Vec<AnimTag>,
    pub fps: u32,
    pub active_frame_index: usize,
    pub active_layer_id: LayerId,
    pub next_layer_id: u64,
}

impl AnimationDocument {
    /// Creates a new document with 1 initial frame and 1 background layer.
    pub fn new(width: u32, height: u32, background_color: ColorRgba) -> Self {
        let bg_id = LayerId(1);
        let bg_meta = LayerMetadata::new(bg_id, "Background");
        let mut first_frame = Frame::new(1000 / 12); // Default ~12 fps (83ms)

        let initial_cel = if background_color == ColorRgba::TRANSPARENT {
            Cel::new(width, height)
        } else {
            Cel::new_with_color(width, height, background_color)
        };
        first_frame.set_cel(bg_id, initial_cel);

        Self {
            width,
            height,
            layers: vec![bg_meta],
            frames: vec![first_frame],
            tags: Vec::new(),
            fps: 12,
            active_frame_index: 0,
            active_layer_id: bg_id,
            next_layer_id: 2,
        }
    }

    pub fn new_transparent(width: u32, height: u32) -> Self {
        Self::new(width, height, ColorRgba::TRANSPARENT)
    }

    /// Creates an AnimationDocument from an existing single-frame LayerStack.
    pub fn from_layer_stack(stack: LayerStack, fps: u32) -> Self {
        let fps = fps.max(1);
        let duration_ms = 1000 / fps;
        let mut frame = Frame::new(duration_ms);
        let mut layers = Vec::with_capacity(stack.layers.len());

        for l in stack.layers {
            layers.push(LayerMetadata {
                id: l.id,
                name: l.name,
                visible: l.visible,
                locked: l.locked,
                opacity: l.opacity,
                blend: l.blend,
            });
            frame.set_cel(l.id, Cel::from_buffer(l.offset_x, l.offset_y, l.buffer));
        }

        Self {
            width: stack.width,
            height: stack.height,
            layers,
            frames: vec![frame],
            tags: Vec::new(),
            fps,
            active_frame_index: 0,
            active_layer_id: stack.active_layer_id,
            next_layer_id: stack.next_layer_id,
        }
    }

    /// Converts a specific frame into a full LayerStack for tool interaction and editing.
    pub fn to_layer_stack(&self, frame_idx: usize) -> LayerStack {
        let frame = self.frames.get(frame_idx).unwrap_or(&self.frames[0]);
        let mut stack_layers = Vec::with_capacity(self.layers.len());

        for meta in &self.layers {
            let (buffer, offset_x, offset_y) = if let Some(cel) = frame.get_cel(meta.id) {
                (cel.buffer.clone(), cel.x, cel.y)
            } else {
                (
                    PixelBuffer::new(self.width, self.height, ColorRgba::TRANSPARENT),
                    0,
                    0,
                )
            };

            stack_layers.push(Layer {
                id: meta.id,
                name: meta.name.clone(),
                visible: meta.visible,
                locked: meta.locked,
                opacity: meta.opacity,
                blend: meta.blend,
                buffer,
                offset_x,
                offset_y,
            });
        }

        LayerStack {
            width: self.width,
            height: self.height,
            layers: stack_layers,
            active_layer_id: self.active_layer_id,
            next_layer_id: self.next_layer_id,
        }
    }

    /// Updates this document's active frame and layer properties from a modified LayerStack.
    pub fn sync_from_layer_stack(&mut self, frame_idx: usize, stack: &LayerStack) {
        if frame_idx >= self.frames.len() {
            return;
        }

        self.width = stack.width;
        self.height = stack.height;
        self.active_layer_id = stack.active_layer_id;
        self.next_layer_id = stack.next_layer_id;

        // Update layer metadata list
        let mut new_layers = Vec::with_capacity(stack.layers.len());
        for l in &stack.layers {
            new_layers.push(LayerMetadata {
                id: l.id,
                name: l.name.clone(),
                visible: l.visible,
                locked: l.locked,
                opacity: l.opacity,
                blend: l.blend,
            });
        }
        self.layers = new_layers;

        // Update active frame cels
        let frame = &mut self.frames[frame_idx];
        for l in &stack.layers {
            frame.set_cel(
                l.id,
                Cel {
                    x: l.offset_x,
                    y: l.offset_y,
                    buffer: l.buffer.clone(),
                },
            );
        }

        // Clean up any cels in other frames for layers that were removed
        let valid_layer_ids: std::collections::HashSet<LayerId> =
            self.layers.iter().map(|l| l.id).collect();
        for f in &mut self.frames {
            f.cels.retain(|id, _| valid_layer_ids.contains(id));
        }
    }

    /// Composites all visible layers of a given frame into a single flattened PixelBuffer.
    pub fn composite_frame(&self, frame_idx: usize) -> PixelBuffer {
        let mut output = PixelBuffer::new(self.width, self.height, ColorRgba::TRANSPARENT);
        let Some(frame) = self.frames.get(frame_idx) else {
            return output;
        };

        for meta in &self.layers {
            if !meta.visible || meta.opacity == 0 {
                continue;
            }

            if let Some(cel) = frame.get_cel(meta.id) {
                if meta.opacity == 100 {
                    output.blend_buffer(&cel.buffer, cel.x, cel.y);
                } else {
                    let mut temp = cel.buffer.clone();
                    temp.apply_opacity(meta.opacity);
                    output.blend_buffer(&temp, cel.x, cel.y);
                }
            }
        }

        output
    }

    /// Adds a new frame at the end of the timeline.
    pub fn add_frame(&mut self, duration_ms: u32, copy_from_prev: bool) -> usize {
        let new_idx = self.frames.len();
        self.insert_frame(new_idx, duration_ms, copy_from_prev)
    }

    /// Inserts a new frame at `at_index`.
    pub fn insert_frame(
        &mut self,
        at_index: usize,
        duration_ms: u32,
        copy_from_prev: bool,
    ) -> usize {
        let at_index = at_index.min(self.frames.len());
        let mut frame = Frame::new(duration_ms);

        if copy_from_prev && !self.frames.is_empty() {
            let prev_idx = if at_index > 0 {
                at_index - 1
            } else {
                self.frames.len() - 1
            };
            if let Some(prev) = self.frames.get(prev_idx) {
                frame.cels = prev.cels.clone();
            }
        } else {
            for layer in &self.layers {
                frame.ensure_cel(layer.id, self.width, self.height);
            }
        }

        self.frames.insert(at_index, frame);

        // Adjust anim tags
        for tag in &mut self.tags {
            if tag.from_frame >= at_index {
                tag.from_frame += 1;
            }
            if tag.to_frame >= at_index {
                tag.to_frame += 1;
            }
        }

        at_index
    }

    /// Duplicates the frame at `frame_idx`.
    pub fn duplicate_frame(&mut self, frame_idx: usize) -> usize {
        if frame_idx >= self.frames.len() {
            return self.frames.len() - 1;
        }

        let new_frame = self.frames[frame_idx].clone();
        let insert_idx = frame_idx + 1;
        self.frames.insert(insert_idx, new_frame);

        // Adjust anim tags
        for tag in &mut self.tags {
            if tag.from_frame > frame_idx {
                tag.from_frame += 1;
            }
            if tag.to_frame >= frame_idx {
                tag.to_frame += 1;
            }
        }

        insert_idx
    }

    /// Removes the frame at `frame_idx` (refuses if only 1 frame remains).
    pub fn remove_frame(&mut self, frame_idx: usize) -> Result<(), &'static str> {
        if self.frames.len() <= 1 {
            return Err("Cannot remove the only remaining frame");
        }
        if frame_idx >= self.frames.len() {
            return Err("Frame index out of bounds");
        }

        self.frames.remove(frame_idx);

        if self.active_frame_index >= self.frames.len() {
            self.active_frame_index = self.frames.len() - 1;
        }

        // Adjust anim tags
        self.tags.retain_mut(|tag| {
            if tag.from_frame > frame_idx {
                tag.from_frame -= 1;
            }
            if tag.to_frame >= frame_idx && tag.to_frame > 0 {
                tag.to_frame -= 1;
            }
            tag.from_frame <= tag.to_frame && tag.from_frame < self.frames.len()
        });

        Ok(())
    }

    /// Moves a frame from `from_idx` to `to_idx`.
    pub fn reorder_frame(&mut self, from_idx: usize, to_idx: usize) -> Result<(), &'static str> {
        if from_idx >= self.frames.len() || to_idx >= self.frames.len() {
            return Err("Frame index out of bounds");
        }
        if from_idx == to_idx {
            return Ok(());
        }

        let frame = self.frames.remove(from_idx);
        self.frames.insert(to_idx, frame);

        if self.active_frame_index == from_idx {
            self.active_frame_index = to_idx;
        }

        Ok(())
    }

    /// Sets duration for a specific frame.
    pub fn set_frame_duration(&mut self, frame_idx: usize, duration_ms: u32) {
        if let Some(frame) = self.frames.get_mut(frame_idx) {
            frame.duration_ms = duration_ms.max(1);
        }
    }

    /// Sets duration for all frames in the document.
    pub fn set_all_durations(&mut self, duration_ms: u32) {
        let duration_ms = duration_ms.max(1);
        for frame in &mut self.frames {
            frame.duration_ms = duration_ms;
        }
    }

    /// Sets global FPS and updates frame durations if desired.
    pub fn set_fps(&mut self, fps: u32) {
        self.fps = fps.max(1);
    }

    /// Adds a new layer to the document stack across all frames.
    pub fn add_layer(&mut self, name: impl Into<String>) -> LayerId {
        let id = LayerId(self.next_layer_id);
        self.next_layer_id += 1;

        let meta = LayerMetadata::new(id, name);
        self.layers.push(meta);

        for frame in &mut self.frames {
            frame.ensure_cel(id, self.width, self.height);
        }

        self.active_layer_id = id;
        id
    }

    /// Removes a layer from the document across all frames.
    pub fn remove_layer(&mut self, layer_id: LayerId) -> Result<(), &'static str> {
        if self.layers.len() <= 1 {
            return Err("Cannot remove the last remaining layer");
        }

        let Some(pos) = self.layers.iter().position(|l| l.id == layer_id) else {
            return Err("Layer not found");
        };

        self.layers.remove(pos);

        for frame in &mut self.frames {
            frame.remove_cel(layer_id);
        }

        if self.active_layer_id == layer_id {
            self.active_layer_id = self.layers[pos.min(self.layers.len() - 1)].id;
        }

        Ok(())
    }

    /// Adds or replaces an animation tag.
    pub fn add_tag(&mut self, tag: AnimTag) {
        self.tags.retain(|t| t.name != tag.name);
        self.tags.push(tag);
    }

    /// Removes an animation tag by index.
    pub fn remove_tag(&mut self, idx: usize) -> Option<AnimTag> {
        if idx < self.tags.len() {
            Some(self.tags.remove(idx))
        } else {
            None
        }
    }

    /// Retrieves cel reference for a given frame and layer.
    pub fn get_cel(&self, frame_idx: usize, layer_id: LayerId) -> Option<&Cel> {
        self.frames.get(frame_idx).and_then(|f| f.get_cel(layer_id))
    }

    /// Retrieves mutable cel reference for a given frame and layer.
    pub fn get_cel_mut(&mut self, frame_idx: usize, layer_id: LayerId) -> Option<&mut Cel> {
        self.frames
            .get_mut(frame_idx)
            .and_then(|f| f.get_cel_mut(layer_id))
    }

    /// Ensures a cel exists and returns a mutable reference to it.
    pub fn ensure_cel(&mut self, frame_idx: usize, layer_id: LayerId) -> Option<&mut Cel> {
        let width = self.width;
        let height = self.height;
        self.frames
            .get_mut(frame_idx)
            .map(|f| f.ensure_cel(layer_id, width, height))
    }

    /// Calculates total playback duration in milliseconds.
    pub fn total_duration_ms(&self) -> u32 {
        self.frames.iter().map(|f| f.duration_ms).sum()
    }

    /// Finds the frame index corresponding to elapsed time `time_ms`.
    pub fn frame_index_at_time(&self, mut time_ms: u32, looping: bool) -> usize {
        let total = self.total_duration_ms();
        if total == 0 || self.frames.is_empty() {
            return 0;
        }

        if looping {
            time_ms %= total;
        } else if time_ms >= total {
            return self.frames.len() - 1;
        }

        let mut accum = 0;
        for (i, frame) in self.frames.iter().enumerate() {
            accum += frame.duration_ms;
            if time_ms < accum {
                return i;
            }
        }

        self.frames.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, BLUE, RED, WHITE};

    #[test]
    fn test_animation_document_initialization() {
        let doc = AnimationDocument::new(64, 64, WHITE);
        assert_eq!(doc.width, 64);
        assert_eq!(doc.height, 64);
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.frames.len(), 1);
        assert_eq!(doc.fps, 12);
        assert_eq!(doc.active_frame_index, 0);

        let composite = doc.composite_frame(0);
        assert_eq!(composite.get_pixel(0, 0), Some(WHITE));
    }

    #[test]
    fn test_frame_add_duplicate_remove_and_durations() {
        let mut doc = AnimationDocument::new(32, 32, ColorRgba::TRANSPARENT);
        assert_eq!(doc.frames.len(), 1);
        assert_eq!(doc.frames[0].duration_ms, 83);

        // Add frame
        let f1 = doc.add_frame(120, true);
        assert_eq!(f1, 1);
        assert_eq!(doc.frames.len(), 2);
        assert_eq!(doc.frames[1].duration_ms, 120);

        // Duplicate frame 1
        let f2 = doc.duplicate_frame(1);
        assert_eq!(f2, 2);
        assert_eq!(doc.frames.len(), 3);

        // Set duration
        doc.set_frame_duration(0, 100);
        doc.set_frame_duration(1, 200);
        doc.set_frame_duration(2, 300);
        assert_eq!(doc.total_duration_ms(), 600);

        // Frame at time test
        assert_eq!(doc.frame_index_at_time(50, true), 0);
        assert_eq!(doc.frame_index_at_time(150, true), 1);
        assert_eq!(doc.frame_index_at_time(350, true), 2);
        assert_eq!(doc.frame_index_at_time(650, true), 0); // Looped 650 % 600 = 50 -> frame 0

        // Remove frame
        assert!(doc.remove_frame(1).is_ok());
        assert_eq!(doc.frames.len(), 2);
    }

    #[test]
    fn test_multi_layer_multi_frame_composite() {
        let mut doc = AnimationDocument::new(10, 10, ColorRgba::TRANSPARENT);
        let l1 = LayerId(1);
        let l2 = doc.add_layer("Foreground");

        // Frame 0: l1 has Red at (0,0), l2 has Blue at (1,1)
        doc.get_cel_mut(0, l1).unwrap().buffer.set_pixel(0, 0, RED);
        doc.get_cel_mut(0, l2).unwrap().buffer.set_pixel(1, 1, BLUE);

        // Frame 1: l1 has Red at (2,2), l2 has Blue at (3,3)
        doc.add_frame(100, false);
        doc.get_cel_mut(1, l1).unwrap().buffer.set_pixel(2, 2, RED);
        doc.get_cel_mut(1, l2).unwrap().buffer.set_pixel(3, 3, BLUE);

        let comp0 = doc.composite_frame(0);
        assert_eq!(comp0.get_pixel(0, 0), Some(RED));
        assert_eq!(comp0.get_pixel(1, 1), Some(BLUE));
        assert_eq!(comp0.get_pixel(2, 2), Some(ColorRgba::TRANSPARENT));

        let comp1 = doc.composite_frame(1);
        assert_eq!(comp1.get_pixel(0, 0), Some(ColorRgba::TRANSPARENT));
        assert_eq!(comp1.get_pixel(2, 2), Some(RED));
        assert_eq!(comp1.get_pixel(3, 3), Some(BLUE));
    }

    #[test]
    fn test_anim_tag_management() {
        let mut doc = AnimationDocument::new(10, 10, WHITE);
        doc.add_frame(100, false);
        doc.add_frame(100, false);
        doc.add_frame(100, false);

        let tag = AnimTag::new("Walk", 1, 3).with_color(255, 128, 0);
        doc.add_tag(tag.clone());

        assert_eq!(doc.tags.len(), 1);
        assert!(doc.tags[0].contains_frame(1));
        assert!(doc.tags[0].contains_frame(2));
        assert!(doc.tags[0].contains_frame(3));
        assert!(!doc.tags[0].contains_frame(0));

        // Inserting frame at 0 shifts tag frames
        doc.insert_frame(0, 100, false);
        assert_eq!(doc.tags[0].from_frame, 2);
        assert_eq!(doc.tags[0].to_frame, 4);
    }

    #[test]
    fn test_layer_stack_roundtrip() {
        let mut doc = AnimationDocument::new(20, 20, WHITE);
        let l2 = doc.add_layer("Layer2");

        let mut stack = doc.to_layer_stack(0);
        assert_eq!(stack.layers.len(), 2);
        stack
            .get_layer_mut(l2)
            .unwrap()
            .buffer
            .set_pixel(5, 5, BLACK);

        doc.sync_from_layer_stack(0, &stack);
        let comp = doc.composite_frame(0);
        assert_eq!(comp.get_pixel(5, 5), Some(BLACK));
    }
}

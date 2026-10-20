//! History management for deep, diff-based Undo/Redo operations.

use crate::image::{compute_bounding_diff, PixelBuffer};
use crate::layers::{LayerId, LayerStack};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryEntry {
    PixelDiff {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        before: Vec<u8>,
        after: Vec<u8>,
    },
    LayerPixelDiff {
        layer_id: LayerId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        before: Vec<u8>,
        after: Vec<u8>,
    },
    LayerStackChange {
        before: LayerStack,
        after: LayerStack,
    },
    CanvasResize {
        old_width: u32,
        old_height: u32,
        new_width: u32,
        new_height: u32,
        before_pixels: Vec<u8>,
        after_pixels: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub struct History {
    pub max_steps: usize,
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
}

impl Default for History {
    fn default() -> Self {
        Self::new(100)
    }
}

impl History {
    pub fn new(max_steps: usize) -> Self {
        Self {
            max_steps: max_steps.max(100),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Records the difference between `before` and `after` single buffer states.
    pub fn record_change(&mut self, before: &PixelBuffer, after: &PixelBuffer) -> bool {
        if before.width() != after.width() || before.height() != after.height() {
            let entry = HistoryEntry::CanvasResize {
                old_width: before.width(),
                old_height: before.height(),
                new_width: after.width(),
                new_height: after.height(),
                before_pixels: before.as_raw().to_vec(),
                after_pixels: after.as_raw().to_vec(),
            };
            self.push_entry(entry);
            return true;
        }

        if let Some((x, y, width, height)) = compute_bounding_diff(before, after) {
            let before_data = before.extract_rect(x, y, width, height);
            let after_data = after.extract_rect(x, y, width, height);
            let entry = HistoryEntry::PixelDiff {
                x,
                y,
                width,
                height,
                before: before_data,
                after: after_data,
            };
            self.push_entry(entry);
            true
        } else {
            false
        }
    }

    /// Records pixel difference on a specific layer.
    pub fn record_layer_pixel_change(
        &mut self,
        layer_id: LayerId,
        before: &PixelBuffer,
        after: &PixelBuffer,
    ) -> bool {
        if let Some((x, y, width, height)) = compute_bounding_diff(before, after) {
            let before_data = before.extract_rect(x, y, width, height);
            let after_data = after.extract_rect(x, y, width, height);
            let entry = HistoryEntry::LayerPixelDiff {
                layer_id,
                x,
                y,
                width,
                height,
                before: before_data,
                after: after_data,
            };
            self.push_entry(entry);
            true
        } else {
            false
        }
    }

    /// Records a full layer stack change (e.g. adding, deleting, reordering, or merging layers).
    pub fn record_layer_stack_change(&mut self, before: &LayerStack, after: &LayerStack) -> bool {
        if before != after {
            let entry = HistoryEntry::LayerStackChange {
                before: before.clone(),
                after: after.clone(),
            };
            self.push_entry(entry);
            true
        } else {
            false
        }
    }

    /// Pushes a direct history entry and clears the redo stack.
    pub fn push_entry(&mut self, entry: HistoryEntry) {
        if self.undo_stack.len() >= self.max_steps {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(entry);
        self.redo_stack.clear();
    }

    /// Undoes on a single `PixelBuffer`.
    pub fn undo(&mut self, buffer: &mut PixelBuffer) -> bool {
        if let Some(entry) = self.undo_stack.pop() {
            match &entry {
                HistoryEntry::PixelDiff {
                    x,
                    y,
                    width,
                    height,
                    before,
                    ..
                } => {
                    buffer.apply_rect(*x, *y, *width, *height, before);
                }
                HistoryEntry::CanvasResize {
                    old_width,
                    old_height,
                    before_pixels,
                    ..
                } => {
                    buffer.replace_with(*old_width, *old_height, before_pixels.clone());
                }
                _ => {}
            }
            self.redo_stack.push(entry);
            true
        } else {
            false
        }
    }

    /// Redoes on a single `PixelBuffer`.
    pub fn redo(&mut self, buffer: &mut PixelBuffer) -> bool {
        if let Some(entry) = self.redo_stack.pop() {
            match &entry {
                HistoryEntry::PixelDiff {
                    x,
                    y,
                    width,
                    height,
                    after,
                    ..
                } => {
                    buffer.apply_rect(*x, *y, *width, *height, after);
                }
                HistoryEntry::CanvasResize {
                    new_width,
                    new_height,
                    after_pixels,
                    ..
                } => {
                    buffer.replace_with(*new_width, *new_height, after_pixels.clone());
                }
                _ => {}
            }
            self.undo_stack.push(entry);
            true
        } else {
            false
        }
    }

    /// Undoes the last recorded action on a `LayerStack`.
    pub fn undo_layer_stack(&mut self, stack: &mut LayerStack) -> bool {
        if let Some(entry) = self.undo_stack.pop() {
            match &entry {
                HistoryEntry::LayerPixelDiff {
                    layer_id,
                    x,
                    y,
                    width,
                    height,
                    before,
                    ..
                } => {
                    if let Some(layer) = stack.layers.iter_mut().find(|l| l.id == *layer_id) {
                        layer.buffer.apply_rect(*x, *y, *width, *height, before);
                    }
                }
                HistoryEntry::LayerStackChange { before, .. } => {
                    *stack = before.clone();
                }
                HistoryEntry::PixelDiff {
                    x,
                    y,
                    width,
                    height,
                    before,
                    ..
                } => {
                    if let Some(layer) = stack.active_layer_mut() {
                        layer.buffer.apply_rect(*x, *y, *width, *height, before);
                    }
                }
                HistoryEntry::CanvasResize {
                    old_width,
                    old_height,
                    before_pixels,
                    ..
                } => {
                    if let Some(layer) = stack.active_layer_mut() {
                        layer
                            .buffer
                            .replace_with(*old_width, *old_height, before_pixels.clone());
                    }
                }
            }
            self.redo_stack.push(entry);
            true
        } else {
            false
        }
    }

    /// Redoes the last undone action on a `LayerStack`.
    pub fn redo_layer_stack(&mut self, stack: &mut LayerStack) -> bool {
        if let Some(entry) = self.redo_stack.pop() {
            match &entry {
                HistoryEntry::LayerPixelDiff {
                    layer_id,
                    x,
                    y,
                    width,
                    height,
                    after,
                    ..
                } => {
                    if let Some(layer) = stack.layers.iter_mut().find(|l| l.id == *layer_id) {
                        layer.buffer.apply_rect(*x, *y, *width, *height, after);
                    }
                }
                HistoryEntry::LayerStackChange { after, .. } => {
                    *stack = after.clone();
                }
                HistoryEntry::PixelDiff {
                    x,
                    y,
                    width,
                    height,
                    after,
                    ..
                } => {
                    if let Some(layer) = stack.active_layer_mut() {
                        layer.buffer.apply_rect(*x, *y, *width, *height, after);
                    }
                }
                HistoryEntry::CanvasResize {
                    new_width,
                    new_height,
                    after_pixels,
                    ..
                } => {
                    if let Some(layer) = stack.active_layer_mut() {
                        layer
                            .buffer
                            .replace_with(*new_width, *new_height, after_pixels.clone());
                    }
                }
            }
            self.undo_stack.push(entry);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLACK, RED, WHITE};

    #[test]
    fn test_undo_redo_diff() {
        let mut history = History::new(100);
        let s0 = PixelBuffer::new(10, 10, WHITE);
        let mut current = s0.clone();

        // Step 1: Draw red dot at (0, 0)
        let mut s1 = current.clone();
        s1.set_pixel(0, 0, RED);
        history.record_change(&current, &s1);
        current = s1.clone();

        // Step 2: Draw black dot at (5, 5)
        let mut s2 = current.clone();
        s2.set_pixel(5, 5, BLACK);
        history.record_change(&current, &s2);
        current = s2.clone();

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 2);

        // Undo step 2
        assert!(history.undo(&mut current));
        assert_eq!(current, s1);
        assert_eq!(current.get_pixel(5, 5), Some(WHITE));
        assert_eq!(current.get_pixel(0, 0), Some(RED));
        assert!(history.can_redo());

        // Undo step 1
        assert!(history.undo(&mut current));
        assert_eq!(current, s0);
        assert_eq!(current.get_pixel(0, 0), Some(WHITE));

        // Redo step 1
        assert!(history.redo(&mut current));
        assert_eq!(current, s1);
        assert_eq!(current.get_pixel(0, 0), Some(RED));

        // Redo step 2
        assert!(history.redo(&mut current));
        assert_eq!(current, s2);
        assert_eq!(current.get_pixel(5, 5), Some(BLACK));
    }

    #[test]
    fn test_layer_stack_undo_redo() {
        let mut history = History::new(100);
        let mut stack = LayerStack::new(10, 10, WHITE);
        let before_stack = stack.clone();

        let l2_id = stack.add_layer("Layer 2");
        history.record_layer_stack_change(&before_stack, &stack);

        assert_eq!(stack.layers.len(), 2);
        assert!(history.undo_layer_stack(&mut stack));
        assert_eq!(stack.layers.len(), 1);
        assert!(history.redo_layer_stack(&mut stack));
        assert_eq!(stack.layers.len(), 2);
        assert_eq!(stack.active_layer_id, l2_id);
    }
}

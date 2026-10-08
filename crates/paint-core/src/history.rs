//! History management for deep Undo/Redo operations.

#[derive(Debug, Default)]
pub struct History {
    pub max_steps: usize,
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
}

#[derive(Debug)]
pub enum HistoryEntry {
    CanvasPixelDiff {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        before: Vec<u8>,
        after: Vec<u8>,
    },
}

impl History {
    pub fn new(max_steps: usize) -> Self {
        Self {
            max_steps,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

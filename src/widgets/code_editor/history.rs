use crate::widgets::code_editor::code::EditBatch;
use std::collections::VecDeque;

/// Bounded undo/redo stack for the code editor.
pub struct History {
    index: usize,
    max_items: usize,
    edits: VecDeque<EditBatch>,
}

impl History {
    /// Creates an empty `History` that retains at most `max_items` batches.
    pub fn new(max_items: usize) -> Self {
        Self {
            index: 0,
            max_items,
            edits: VecDeque::new(),
        }
    }

    /// Pushes a new `batch` onto the history stack.
    ///
    /// Any previously undone batches beyond the current position are discarded.
    /// If the stack is full, the oldest entry is dropped.
    pub fn push(&mut self, batch: EditBatch) {
        while self.edits.len() > self.index {
            self.edits.pop_back();
        }

        if self.edits.len() == self.max_items {
            self.edits.pop_front();
            self.index -= 1;
        }

        self.edits.push_back(batch);
        self.index += 1;
    }

    /// Steps one position back in history and returns the batch to reverse.
    ///
    /// Returns `None` if already at the beginning.
    pub fn undo(&mut self) -> Option<EditBatch> {
        if self.index == 0 {
            None
        } else {
            self.index -= 1;
            self.edits.get(self.index).cloned()
        }
    }

    /// Steps one position forward in history and returns the batch to re-apply.
    ///
    /// Returns `None` if already at the most recent position.
    pub fn redo(&mut self) -> Option<EditBatch> {
        if self.index >= self.edits.len() {
            None
        } else {
            let batch = self.edits.get(self.index).cloned();
            self.index += 1;
            batch
        }
    }
}

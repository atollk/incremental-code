#[derive(Debug, Clone, Copy)]
/// Controls how click-and-drag selection snaps to word or line boundaries.
pub enum SelectionSnap {
    None,
    Word { anchor: usize },
    Line { anchor: usize },
}

#[derive(Debug, Clone, Copy)]
/// A half-open char-index range `[start, end)` representing the current text selection.
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    /// Creates a `Selection` from two positions, normalising so `start <= end`.
    pub fn new(a: usize, b: usize) -> Self {
        Self {
            start: a.min(b),
            end: a.max(b),
        }
    }

    /// Creates a `Selection` from an anchor position and the current cursor position.
    pub fn from_anchor_and_cursor(anchor: usize, cursor: usize) -> Self {
        if anchor <= cursor {
            Selection {
                start: anchor,
                end: cursor,
            }
        } else {
            Selection {
                start: cursor,
                end: anchor,
            }
        }
    }

    /// Returns `true` if the selection spans at least one character.
    pub fn is_active(&self) -> bool {
        self.start != self.end
    }

    /// Returns `true` if the selection is empty (start equals end).
    pub fn is_empty(&self) -> bool {
        self.start.max(self.end) == self.start.min(self.end)
    }

    /// Returns `true` if `index` falls within `[start, end)`.
    pub fn contains(&self, index: usize) -> bool {
        index >= self.start && index < self.end
    }

    /// Returns `(min, max)` of the two endpoints in ascending order.
    pub fn sorted(&self) -> (usize, usize) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
}

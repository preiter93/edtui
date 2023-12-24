//! This module contains a rather brute-force implementation of undo and redo functionality
//! for a simple text editor. It stores the entire editor state at each action.
//!
//! While this approach works for basic undo/redo needs, it may not be efficient for more
//! complex usage. In the long run, this should be replaced with an action-based mechanism.
use crate::{EditorState, Index2, Lines};

#[derive(Debug, Clone)]
pub(crate) struct Stack {
    inner: Vec<UndoState>,
    max_size: usize,
}

impl Stack {
    pub(crate) fn new() -> Self {
        Self {
            inner: Vec::new(),
            max_size: 100,
        }
    }

    pub(crate) fn pop(&mut self) -> Option<UndoState> {
        self.inner.pop()
    }

    pub(crate) fn push(&mut self, value: UndoState) {
        self.inner.push(value);
        if self.len() > self.max_size {
            self.remove(0);
        }
    }

    fn len(&mut self) -> usize {
        self.inner.len()
    }

    fn remove(&mut self, index: usize) {
        self.inner.remove(index);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct UndoState {
    lines: Lines,
    cursor: Index2,
}

impl EditorState {
    pub(crate) fn capture(&mut self) {
        let editor_state = UndoState {
            lines: self.lines.clone(),
            cursor: self.cursor,
        };
        self.undo.push(editor_state);
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            let current = UndoState {
                lines: self.lines.clone(),
                cursor: self.cursor,
            };
            self.lines = prev.lines;
            self.cursor = prev.cursor;
            self.redo.push(current);
        }
    }

    pub fn redo(&mut self) {
        if let Some(prev) = self.redo.pop() {
            let current = UndoState {
                lines: self.lines.clone(),
                cursor: self.cursor,
            };
            self.lines = prev.lines;
            self.cursor = prev.cursor;
            self.undo.push(current);
        }
    }
}

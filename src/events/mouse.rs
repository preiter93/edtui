#![cfg(feature = "mouse")]
use jagged::Index2;
use ratatui::crossterm::event::{MouseEvent as CTMouseEvent, MouseEventKind};

use crate::{
    actions::{Execute, SwitchMode},
    helper::{is_out_of_bounds, set_selection},
    EditorMode, EditorState,
};

/// Handles a mouse event.
#[derive(Clone, Debug, Default)]
pub struct EditorMouse {}

impl EditorMouse {
    pub fn on_event<E>(event: E, state: &mut EditorState)
    where
        E: Into<MouseEvent>,
    {
        let event = event.into();
        if let MouseEvent::None = event {
            return;
        }

        let total_textarea_offset =
            state.view.editor_to_textarea_offset + state.view.window_to_editor_offset;

        match event {
            MouseEvent::Down(mouse) | MouseEvent::Up(mouse) | MouseEvent::Drag(mouse) => {
                let cursor = Index2::new(
                    mouse.row.saturating_sub(total_textarea_offset.x),
                    mouse.col.saturating_sub(total_textarea_offset.y),
                );
                if !is_out_of_bounds(&state.lines, &cursor) {
                    state.cursor = cursor;
                } else {
                    state.cursor = state.lines.last_index().unwrap_or(state.cursor);
                }
            }
            MouseEvent::None => return,
        };

        if let MouseEvent::Down(_) = event {
            state.selection = None;
            if state.mode == EditorMode::Visual {
                SwitchMode(EditorMode::Normal).execute(state);
            }
        }

        if let MouseEvent::Drag(_) = event {
            if state.mode != EditorMode::Visual {
                SwitchMode(EditorMode::Visual).execute(state);
            }
            set_selection(&mut state.selection, state.cursor);
        }
    }
}

/// Represents a mouse event.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MouseEvent {
    /// A mouse press event.
    Down(MousePosition),

    /// A mouse release event.
    Up(MousePosition),

    /// A mouse Drag event.
    Drag(MousePosition),

    /// A mouse event that is handled by the editor.
    None,
}

impl From<CTMouseEvent> for MouseEvent {
    fn from(event: CTMouseEvent) -> Self {
        match event.kind {
            MouseEventKind::Down(_) => Self::Down(MousePosition::new(event.row, event.column)),
            MouseEventKind::Up(_) => Self::Up(MousePosition::new(event.row, event.column)),
            MouseEventKind::Drag(_) => Self::Drag(MousePosition::new(event.row, event.column)),
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct MousePosition {
    /// The row that the event occurred on.
    pub(crate) row: usize,
    /// The column that the event occurred on.
    pub(crate) col: usize,
}

impl MousePosition {
    /// Creates a new `MousePosition` instance.
    fn new(row: u16, col: u16) -> Self {
        Self {
            row: row.into(),
            col: col.into(),
        }
    }
}

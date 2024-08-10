use jagged::Index2;
use ratatui::crossterm::event::{MouseEvent as CTMouseEvent, MouseEventKind};

use crate::{
    actions::{Execute, SwitchMode},
    helper::set_selection,
    EditorMode, EditorState,
};

/// Handles a mouse event.
#[derive(Clone, Debug, Default)]
pub struct EditorMouse {}

impl EditorMouse {
    pub fn on_event<E>(self, event: E, state: &mut EditorState)
    where
        E: Into<MouseEvent>,
    {
        let event = event.into();
        if event == MouseEvent::None {
            return;
        }

        let total_textarea_offset = state.view.editor_to_textarea_offset;

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

        match event {
            MouseEvent::Down(mouse) | MouseEvent::Up(mouse) | MouseEvent::Drag(mouse) => {
                let lines = &state.lines;
                let cursor = Index2::new(
                    mouse.row.saturating_sub(total_textarea_offset.y),
                    mouse.col.saturating_sub(total_textarea_offset.x),
                );
                let last_row = lines.len().saturating_sub(1);
                let last_col = lines.len_col(cursor.row).unwrap_or(0).saturating_sub(1);

                // row is out of bounds
                if last_row < cursor.row {
                    let last_col = lines.len_col(last_row).unwrap_or(0).saturating_sub(1);
                    state.cursor = Index2::new(last_row, last_col);
                // col is out of bounds
                } else if last_col < cursor.col {
                    state.cursor = Index2::new(cursor.row, last_col);
                } else {
                    state.cursor = cursor;
                }

                if let MouseEvent::Drag(_) = event {
                    set_selection(&mut state.selection, state.cursor);
                }
            }
            MouseEvent::None => (),
        };
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

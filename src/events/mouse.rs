use crossterm::event::{MouseEvent as CTMouseEvent, MouseEventKind};
use jagged::Index2;

use crate::{
    actions::{Execute, SwitchMode},
    helper::char_width,
    state::selection::set_selection,
    view::line_wrapper::LineWrapper,
    EditorMode, EditorState,
};

/// Handles a mouse event.
#[derive(Clone, Debug, Default)]
pub struct MouseEventHandler {}

impl MouseEventHandler {
    pub fn on_event<E>(event: E, state: &mut EditorState)
    where
        E: Into<MouseEvent>,
    {
        let event = event.into();
        if event == MouseEvent::None {
            return;
        }

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
                let cursor = mouse_position_to_cursor_position(state, &mouse, state.view.tab_width);
                let last_row = lines.last_row_index();
                let last_col = lines.last_col_index(cursor.row);

                // row is out of bounds
                if last_row < cursor.row {
                    let last_col = lines.last_col_index(last_row);
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

fn mouse_position_to_cursor_position(
    state: &EditorState,
    mouse: &MousePosition,
    tab_width: usize,
) -> Index2 {
    let mut row_index = state.view.viewport.y;
    let mut col_index = state.view.viewport.x;

    // Global -> editor coordinates
    let mut mouse = Index2::new(
        mouse.row.saturating_sub(state.view.screen_area.y.into()),
        mouse.col.saturating_sub(state.view.screen_area.x.into()),
    );

    if !state.view.wrap {
        return Index2::new(
            mouse.row.saturating_add(row_index),
            mouse.col.saturating_add(col_index),
        );
    }

    let mut row_screen_index = 0;
    for line in state.lines.iter_row().skip(row_index) {
        let wrapped_line = LineWrapper::wrap_line(
            line,
            state.view.screen_area.width.into(),
            state.view.tab_width,
        );
        let wrapped_line_len = wrapped_line.len().max(1);
        if row_screen_index + wrapped_line_len > mouse.row {
            mouse.row = mouse.row.saturating_sub(row_screen_index);
            col_index = find_cursor_column_in_wrapped_line(&wrapped_line, &mouse, tab_width);
            break;
        }
        row_screen_index += wrapped_line_len;
        row_index += 1;
    }

    Index2::new(row_index, col_index)
}

fn find_cursor_column_in_wrapped_line(
    line: &[Vec<char>],
    mouse: &Index2,
    tab_width: usize,
) -> usize {
    let Some(l) = line.get(mouse.row) else {
        return 0;
    };

    let col_offset: usize = line.iter().take(mouse.row).map(Vec::len).sum();
    let mut current_width = 0;
    let mut col_index = 0;

    for &ch in l {
        let char_width = char_width(ch, tab_width);

        if current_width + char_width > mouse.col {
            break;
        }

        current_width += char_width;
        col_index += 1;
    }

    col_offset + col_index
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

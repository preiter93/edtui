//! The editors state
pub mod status_line;
pub mod theme;
use self::theme::EditorTheme;
use crate::{helper::max_col, state::EditorState, EditorMode, Index2};
use ratatui::{prelude::*, widgets::Widget};
pub use status_line::EditorStatusLine;

pub struct EditorView<'a, 'b> {
    pub(crate) state: &'a mut EditorState,
    pub(crate) theme: EditorTheme<'b>,
}

impl<'a, 'b> EditorView<'a, 'b> {
    /// Creates a new instance of [`EditorView`].
    #[must_use]
    pub fn new(state: &'a mut EditorState) -> Self {
        Self {
            state,
            theme: EditorTheme::default(),
        }
    }

    /// Set the theme for the [`EditorView`]
    /// See [`EditorTheme`] for the customizable parameters.
    #[must_use]
    pub fn theme(mut self, theme: EditorTheme<'b>) -> Self {
        self.theme = theme;
        self
    }

    /// Returns a reference to the [`EditorState`].
    #[must_use]
    pub fn get_state(&'a self) -> &'a EditorState {
        self.state
    }

    /// Returns a mutable reference to the [`EditorState`].
    #[must_use]
    pub fn get_state_mut(&'a mut self) -> &'a mut EditorState {
        self.state
    }
}

impl Widget for EditorView<'_, '_> {
    // type State = ViewState;
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw the border.
        buf.set_style(area, self.theme.base);
        let area = match &self.theme.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.clone().render(area, buf);
                inner_area
            }
            None => area,
        };

        // Split into main section and status line
        let [main, status] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(u16::from(self.theme.status_line.is_some())),
        ])
        .areas(area);
        let width = main.width as usize;
        let height = main.height as usize;

        // Retrieve the displayed cursor position. The column of the displayed
        // cursor is clamped to the maximum line length.
        let cursor = displayed_cursor(self.state);

        // Store the offset from the current buffer to the textarea inside the state.
        // This is required to calculate mouse positions correctly.
        self.state.view.set_editor_to_textarea_offset(area);

        // Update the view offset. Requuires the screen size and the position
        // of the cursor. Updates the view offset only if the cursor is out
        // side of the view port. The state is stored in the `ViewOffset`.
        let size = (width, height);
        let offset = self.state.view.update_viewport_offset(size, cursor);

        // Rendering of the cursor. Cursor is not rendered in the loop below,
        // as the cursor may be outside the text in input mode.
        let x_cursor = (main.left() as usize) + width.min(cursor.col.saturating_sub(offset.x));
        let y_cursor = (main.top() as usize) + cursor.row.saturating_sub(offset.y);
        if let Some(cell) = buf.cell_mut(Position::new(x_cursor as u16, y_cursor as u16)) {
            cell.set_style(self.theme.cursor_style);
        }

        // Rendering the text and the selection.
        let lines = &self.state.lines;
        for (i, line) in lines.iter_row().skip(offset.y).take(height).enumerate() {
            let y = (main.top() as usize) as u16 + i as u16;
            for (j, char) in line.iter().skip(offset.x).take(width).enumerate() {
                let x = (main.left() as usize) as u16 + j as u16;

                // Text
                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    cell.set_symbol(&char.to_string());
                }

                // Selection
                if let Some(selection) = &self.state.selection {
                    let position = Index2::new(offset.y + i, offset.x + j);
                    if selection.contains(&position) {
                        if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                            cell.set_style(self.theme.selection_style);
                        }
                    }
                }
            }
        }

        // Render the status line.
        if let Some(s) = self.theme.status_line {
            s.mode(self.state.mode.name())
                .search(if self.state.mode == EditorMode::Search {
                    Some(self.state.search_pattern())
                } else {
                    None
                })
                .render(status, buf);
        }
    }
}

/// Retrieves the displayed cursor position based on the editor state.
///
/// Ensures that the displayed cursor position doesn't exceed the line length.
/// If the internal cursor position exceeds the maximum column, clamp it to
/// the maximum.
fn displayed_cursor(state: &EditorState) -> Index2 {
    let max_col = max_col(&state.lines, &state.cursor, state.mode);
    Index2::new(state.cursor.row, state.cursor.col.min(max_col))
}

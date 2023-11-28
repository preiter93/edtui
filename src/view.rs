pub mod status_line;
pub mod theme;
use ratatui::{prelude::*, widgets::Widget};
pub use status_line::StatusLine;

use crate::state::{position::Position, EditorState};

use self::theme::EditorTheme;

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
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(u16::from(self.theme.status_line.is_some())),
            ])
            .split(area);
        let (main, foot) = (area[0], area[1]);
        let width = main.width as usize;
        let height = main.height as usize;

        // Update the view offset. Requuires the screen size and the position
        // of the cursor. Updates the view offset only if the cursor is out
        // side of the view port. The state is stored in the `ViewOffset`.
        let size = (width, height);
        let cursor = (self.state.cursor.column, self.state.cursor.line);
        let (x_off, y_off) = self.state.view.update_offset(size, cursor);

        // Rendering of the cursor. Speficially not rendered in the loop below,
        // as the cursor may be outside the text in input mode.
        let cursor = &self.state.cursor;
        let x_cursor = (main.left() as usize) + width.min(cursor.column.saturating_sub(x_off));
        let y_cursor = (main.top() as usize) + cursor.line.saturating_sub(y_off);
        buf.get_mut(x_cursor as u16, y_cursor as u16)
            .set_style(self.theme.cursor_style);

        // Rendering the text and the selection.
        let lines = &self.state.lines;
        for (i, line) in lines.iter().skip(y_off).take(height).enumerate() {
            let y = (main.top() as usize) as u16 + i as u16;
            for (j, char) in line.iter().skip(x_off).take(width).enumerate() {
                let x = (main.left() as usize) as u16 + j as u16;

                // Text
                buf.get_mut(x, y).set_symbol(&char.to_string());

                // Selection
                if let Some(selection) = &self.state.selection {
                    let position = Position::new(x_off + i, y_off + j);
                    if selection.within(&position) {
                        buf.get_mut(x, y).set_style(self.theme.selection_style);
                    }
                }
            }
        }

        // Render the status line.
        if let Some(s) = self.theme.status_line {
            s.content(self.state.mode.name()).render(foot, buf);
        }
    }
}

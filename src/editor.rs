pub mod status_line;
pub mod theme;
pub mod view;
use ratatui::{prelude::*, widgets::Widget};
pub use status_line::StatusLine;

use crate::buffer::{position::Position, TextBuffer};

use self::{theme::EditorTheme, view::ViewState};

pub struct Editor<'a, 'b> {
    pub(crate) buffer: &'a TextBuffer,
    pub(crate) state: &'a mut ViewState,
    pub(crate) theme: EditorTheme<'b>,
}

impl<'a, 'b> Editor<'a, 'b> {
    /// Creates a new instance of [`Editor`].
    #[must_use]
    pub fn new(buffer: &'a TextBuffer, state: &'a mut ViewState) -> Self {
        Self {
            buffer,
            state,
            theme: EditorTheme::default(),
        }
    }

    /// This method allows you to pass a custome theme to the [`Editor`]
    /// See [`EditorTheme`] for the customizable parameters.
    #[must_use]
    pub fn theme(mut self, theme: EditorTheme<'b>) -> Self {
        self.theme = theme;
        self
    }

    /// Returns a reference to the text buffer
    #[must_use]
    pub fn get_buffer(&self) -> &'a TextBuffer {
        self.buffer
    }
}

impl Widget for Editor<'_, '_> {
    // type State = ViewState;
    fn render(
        self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        // state: &mut Self::State,
    ) {
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
                Constraint::Length(if self.theme.status_line.is_some() {
                    1
                } else {
                    0
                }),
            ])
            .split(area);
        let (main, foot) = (area[0], area[1]);
        let width = main.width as usize;
        let height = main.height as usize;

        // Update the view offset. Requuires the screen size and the position
        // of the cursor. Updates the view offset only if the cursor is out
        // side of the view port. The state is stored in the `ViewOffset`.
        let size = (width, height);
        let cursor = (self.buffer.cursor.column, self.buffer.cursor.line);
        let (x_off, y_off) = self.state.update_offset(size, cursor);

        // Rendering of the cursor. Speficially not rendered in the loop below,
        // as the cursor may be outside the text in input mode.
        let cursor = &self.buffer.cursor;
        let x_cursor = (main.left() as usize) + width.min(cursor.column.saturating_sub(x_off));
        let y_cursor = (main.top() as usize) + cursor.line.saturating_sub(y_off);
        buf.get_mut(x_cursor as u16, y_cursor as u16)
            .set_style(self.theme.cursor_style);

        // Rendering the text and the selection.
        let lines = &self.buffer.lines;
        for (i, line) in lines.iter().skip(y_off).take(height).enumerate() {
            let y = (main.top() as usize) as u16 + i as u16;
            for (j, char) in line.iter().skip(x_off).take(width).enumerate() {
                let x = (main.left() as usize) as u16 + j as u16;

                // Text
                buf.get_mut(x, y).set_symbol(&char.to_string());

                // Selection
                if let Some(selection) = &self.buffer.selection {
                    let position = Position::new(x_off + i, y_off + j);
                    if selection.within(&position) {
                        buf.get_mut(x, y).set_style(self.theme.selection_style);
                    }
                }
            }
        }

        // Render the status line.
        if let Some(s) = self.theme.status_line {
            s.content(self.buffer.mode.name()).render(foot, buf);
        }
    }
}

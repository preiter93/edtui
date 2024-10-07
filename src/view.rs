//! The editors state
pub(crate) mod line_wrapper;
pub mod status_line;
pub mod theme;

use std::cmp::min;

use self::theme::EditorTheme;
use crate::{
    helper::max_col,
    state::{selection::Selection, EditorState},
    EditorMode, Index2,
};
use line_wrapper::LineWrapper;
use ratatui::{prelude::*, widgets::Widget};
pub use status_line::EditorStatusLine;

pub struct EditorView<'a, 'b> {
    pub(crate) state: &'a mut EditorState,
    pub(crate) theme: EditorTheme<'b>,
    pub(crate) wrap: bool,
}

impl<'a, 'b> EditorView<'a, 'b> {
    /// Creates a new instance of [`EditorView`].
    #[must_use]
    pub fn new(state: &'a mut EditorState) -> Self {
        Self {
            state,
            theme: EditorTheme::default(),
            wrap: true,
        }
    }

    /// Set the theme for the [`EditorView`]
    /// See [`EditorTheme`] for the customizable parameters.
    #[must_use]
    pub fn theme(mut self, theme: EditorTheme<'b>) -> Self {
        self.theme = theme;
        self
    }

    /// Sets whether overflowing lines should wrap onto the next line.
    ///
    /// # Note
    /// Line wrapping currently has issues when used with mouse events.
    #[must_use]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
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
        let lines = &self.state.lines;

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
        let offset = self
            .state
            .view
            .update_viewport_offset(size, cursor, lines, self.wrap);

        // Predetermine search highlighted selections.
        let mut search_selection = None;
        if self.state.mode == EditorMode::Search {
            search_selection = self.state.search.selected_range();
        };
        let selections = [&self.state.selection, &search_selection];

        let mut y = (main.top() as usize) as u16;
        let mut num_rows = 0;
        for (i, line) in lines.iter_row().skip(offset.y).enumerate() {
            let row_index = offset.y + i;
            num_rows += 1;

            // Wrap lines
            let spans = InternalLine::new(
                line,
                self.theme.base,
                self.theme.selection_style,
                offset.y + i,
                offset.x,
            )
            .into_spans(&selections);

            let (line_widths, wrapped_spans) = if self.wrap {
                LineWrapper::wrap_spans(spans, main.width as usize)
            } else {
                let spans_width = spans_width(&spans);
                (vec![spans_width], vec![spans])
            };
            let line_count = wrapped_spans.len();

            // Rendering the content line by line.
            let mut y_line = y;
            for (i, span) in wrapped_spans.into_iter().enumerate() {
                let area = Rect::new(main.left(), y_line, main.width, main.height);
                span.into_iter().collect::<Line>().render(area, buf);

                // Increment the y position if there are more lines left to render.
                if i + 1 < line_count {
                    y_line += 1;
                }

                if y_line >= main.bottom() {
                    break;
                }
            }

            // Rendering of the the cursor. Must take line wrapping into account.
            if row_index == cursor.row {
                let relative_position = LineWrapper::find_position(&line_widths, cursor.col);
                let relative_position_col = relative_position.col.saturating_sub(offset.x);
                let x_cursor = main.left() + min(width, relative_position_col) as u16;
                let y_cursor = y + relative_position.row as u16;
                if let Some(cell) = buf.cell_mut(Position::new(x_cursor, y_cursor)) {
                    cell.set_style(self.theme.cursor_style);
                }
            }

            // Increment y after rendering the current visual line.
            y = y_line + 1;
            if y >= main.bottom() {
                break;
            }
        }

        // Save the total number of lines displayed in the viewport.
        // This is necessary to correctly handle scrolling.
        self.state.view.update_num_rows(num_rows);

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

fn spans_width(spans: &[Span]) -> usize {
    spans.iter().fold(0, |sum, span| sum + span.width())
}

struct InternalLine<'a> {
    line: &'a [char],
    base: Style,
    highlighted: Style,
    row_index: usize,
    col_offset: usize,
}

impl<'a> InternalLine<'a> {
    fn new(
        line: &'a [char],
        base: Style,
        highlighted: Style,
        row_index: usize,
        col_offset: usize,
    ) -> Self {
        Self {
            line,
            base,
            highlighted,
            row_index,
            col_offset,
        }
    }
}

impl<'a> InternalLine<'a> {
    fn get_style(&self, is_selected: bool) -> Style {
        if is_selected {
            self.highlighted
        } else {
            self.base
        }
    }

    /// Converts an `InternalLine` into a vector of `Span`s, applying styles based on the
    /// given selections.
    fn into_spans(self, selections: &[&Option<Selection>]) -> Vec<Span<'a>> {
        let mut spans = Vec::new();
        let mut current_span = String::new();
        let mut previous_is_selected = false;

        // Iterate over the line's characters, starting from the offset
        for (i, &ch) in self.line.iter().skip(self.col_offset).enumerate() {
            let position = Index2::new(self.row_index, self.col_offset + i);

            // Check if the current position is selected by any selection
            let current_is_selected = selections
                .iter()
                .filter_map(|selection| selection.as_ref())
                .any(|selection| selection.contains(&position));

            // If the selection state has changed, push the current span and start a new one
            if i != 0 && previous_is_selected != current_is_selected {
                spans.push(Span::styled(
                    current_span.clone(),
                    self.get_style(previous_is_selected),
                ));
                current_span.clear();
            }

            previous_is_selected = current_is_selected;
            current_span.push(ch);
        }

        // Push the final span
        spans.push(Span::styled(
            current_span,
            self.get_style(previous_is_selected),
        ));

        spans
    }
}

fn crop_first(s: &str, pos: usize) -> &str {
    match s.char_indices().nth(pos) {
        Some((pos, _)) => &s[pos..],
        None => "",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_line_into_spans() {
        // given
        let base = Style::default();
        let hightlighted = Style::default().red();
        let line = "Hello".chars().into_iter().collect::<Vec<char>>();

        let selection = Some(Selection::new(Index2::new(0, 0), Index2::new(0, 2)));
        let selections = vec![&selection];

        // when
        let spans = InternalLine::new(&line, base, hightlighted, 0, 0).into_spans(&selections);

        // then
        assert_eq!(spans[0], Span::styled("Hel", hightlighted));
        assert_eq!(spans[1], Span::styled("lo", base));
    }
}

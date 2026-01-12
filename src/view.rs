mod internal;
pub(crate) mod line_wrapper;
mod render_line;
pub mod status_line;
#[cfg(feature = "syntax-highlighting")]
pub mod syntax_higlighting;
pub mod theme;

use render_line::RenderLine;
#[cfg(feature = "syntax-highlighting")]
use syntax_higlighting::SyntaxHighlighter;

use crate::{
    helper::{max_col, rect_indent_y},
    state::{selection::Selection, EditorState},
    EditorMode, Index2,
};

#[cfg(feature = "syntax-highlighting")]
use internal::line_into_highlighted_spans_with_selections;
use internal::line_into_spans_with_selections;
use jagged::index::RowIndex;
use line_wrapper::LineWrapper;
use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::Style,
    text::Span,
    widgets::Widget,
};
pub use status_line::EditorStatusLine;
use theme::EditorTheme;

/// Configuration for line numbers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineNumbers {
    /// Line numbers are disabled (default).
    #[default]
    None,
    /// Display absolute line numbers.
    Absolute,
    /// Display relative line numbers.
    Relative,
}

/// Creates the view for the editor. [`EditorView`] and [`EditorState`] are
/// the core classes of edtui.
///
/// ## Example
///
/// ```rust
/// use edtui::EditorState;
/// use edtui::EditorTheme;
/// use edtui::EditorView;
///
/// let theme = EditorTheme::default();
/// let mut state = EditorState::default();
///
/// EditorView::new(&mut state)
///     .wrap(true)
///     .theme(theme);
/// ```
pub struct EditorView<'a, 'b> {
    /// The editor state.
    pub(crate) state: &'a mut EditorState,

    /// The editor theme.
    pub(crate) theme: EditorTheme<'b>,

    /// An optional syntax highlighter.
    #[cfg(feature = "syntax-highlighting")]
    pub(crate) syntax_highlighter: Option<SyntaxHighlighter>,
}

impl<'a, 'b> EditorView<'a, 'b> {
    /// Creates a new instance of [`EditorView`].
    #[must_use]
    pub fn new(state: &'a mut EditorState) -> Self {
        Self {
            state,
            theme: EditorTheme::default(),
            #[cfg(feature = "syntax-highlighting")]
            syntax_highlighter: None,
        }
    }

    /// Set the theme for the [`EditorView`]
    /// See [`EditorTheme`] for the customizable parameters.
    #[must_use]
    pub fn theme(mut self, theme: EditorTheme<'b>) -> Self {
        self.theme = theme;
        self
    }

    #[cfg(feature = "syntax-highlighting")]
    /// Set the syntax highlighter for the [`EditorView`]
    /// See [`SyntaxHighlighter`] for the more information.
    ///
    /// ```rust
    /// use edtui::EditorState;
    /// use edtui::EditorView;
    /// use edtui::SyntaxHighlighter;
    ///
    /// let mut state = EditorState::default();
    /// let syntax_highlighter = SyntaxHighlighter::new("dracula", "rs").unwrap();
    ///
    /// EditorView::new(&mut state).syntax_highlighter(Some(syntax_highlighter));
    /// ```
    #[must_use]
    pub fn syntax_highlighter(mut self, syntax_highlighter: Option<SyntaxHighlighter>) -> Self {
        self.syntax_highlighter = syntax_highlighter;
        self
    }

    /// Sets whether overflowing lines should wrap onto the next line.
    ///
    /// # Note
    /// Line wrapping currently has issues when used with mouse events.
    #[must_use]
    pub fn wrap(self, wrap: bool) -> Self {
        self.state.view.wrap = wrap;
        self
    }

    pub(super) fn get_wrap(&self) -> bool {
        self.state.view.wrap
    }

    /// Sets the number of spaces used for rendering tabs.
    #[must_use]
    pub fn tab_width(self, tab_width: usize) -> Self {
        self.state.view.tab_width = tab_width;
        self
    }

    /// Returns the tab width configuration.
    pub(super) fn get_tab_width(&self) -> usize {
        self.state.view.tab_width
    }

    /// Configures line numbers. Disabled by default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::{EditorState, EditorView, LineNumbers};
    ///
    /// let mut state = EditorState::default();
    ///
    /// // Enable absolute line numbers
    /// EditorView::new(&mut state).line_numbers(LineNumbers::Absolute);
    ///
    /// // Enable relative line numbers
    /// EditorView::new(&mut state).line_numbers(LineNumbers::Relative);
    /// ```
    #[must_use]
    pub fn line_numbers(self, line_numbers: LineNumbers) -> Self {
        self.state.view.line_numbers = line_numbers;
        self
    }

    /// Returns the line numbers configuration.
    pub(super) fn get_line_numbers(&self) -> LineNumbers {
        self.state.view.line_numbers
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

    /// Calculate the width needed for the line number gutter.
    fn line_number_width(&self) -> u16 {
        if self.state.view.line_numbers == LineNumbers::None {
            return 0;
        }

        let total_lines = self.state.lines.len().max(1);
        let digits = total_lines.to_string().len();
        (digits + 1) as u16
    }
}

impl Widget for EditorView<'_, '_> {
    #[allow(clippy::too_many_lines)]
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

        // Calculate line number gutter width and split area
        let line_number_width = self.line_number_width();
        let line_numbers_style = self.theme.line_numbers_style;
        let (gutter_area, content_main) = if line_number_width > 0 {
            let [gutter, content] =
                Layout::horizontal([Constraint::Length(line_number_width), Constraint::Min(0)])
                    .areas(main);
            // Fill the entire gutter with the line numbers style
            buf.set_style(gutter, line_numbers_style);
            (Some(gutter), content)
        } else {
            (None, main)
        };

        let width = content_main.width as usize;
        let height = content_main.height as usize;
        let wrap_lines = self.get_wrap();
        let tab_width = self.get_tab_width();
        let line_numbers = self.get_line_numbers();
        let lines = &self.state.lines;

        // Retrieve the displayed cursor position. The column of the displayed
        // cursor is clamped to the maximum line length.
        let max_col = max_col(&self.state.lines, &self.state.cursor, self.state.mode);
        let cursor = Index2::new(self.state.cursor.row, self.state.cursor.col.min(max_col));

        // Store the coordinates of the current editor.
        // Use content_main (not main) so mouse events are calculated relative to text area.
        self.state.view.set_screen_area(content_main);

        // Update the view offset. Requires the screen size and the position
        // of the cursor. Updates the view offset only if the cursor is out
        // side of the view port. The state is stored in the `ViewOffset`.
        let view_state = &mut self.state.view;
        let (offset_x, offset_y) = if wrap_lines {
            (
                0,
                view_state.update_viewport_vertical_wrap(width, height, cursor.row, lines),
            )
        } else {
            let line = lines.get(RowIndex::new(cursor.row));
            (
                view_state.update_viewport_horizontal(width, cursor.col, line),
                view_state.update_viewport_vertical(height, cursor.row),
            )
        };

        // Predetermine highlighted sections.
        let mut search_selection: Option<Selection> = None;
        if self.state.mode == EditorMode::Search {
            search_selection = (&self.state.search).into();
        };
        let selections = vec![&self.state.selection, &search_selection];

        let mut cursor_position: Option<Position> = None;
        let mut content_area = content_main;
        let mut gutter_row_area = gutter_area;
        let mut num_rendered_rows = 0;

        let line_numbers_enabled = line_numbers != LineNumbers::None;
        let is_relative = line_numbers == LineNumbers::Relative;

        let mut row_index = offset_y;
        for line in lines.iter_row().skip(row_index) {
            if content_area.height == 0 {
                break;
            }

            let col_skips = offset_x;
            num_rendered_rows += 1;

            let spans = generate_spans(
                line,
                &selections,
                row_index,
                col_skips,
                &self.theme.base,
                &self.theme.selection_style,
                #[cfg(feature = "syntax-highlighting")]
                self.syntax_highlighter.as_ref(),
            );

            let render_line = if wrap_lines {
                RenderLine::Wrapped(LineWrapper::wrap_spans(spans, width, tab_width))
            } else {
                RenderLine::Single(spans)
            };

            // Render line number in the gutter
            if line_numbers_enabled {
                if let Some(gutter) = gutter_row_area {
                    let is_cursor_line = row_index == cursor.row;
                    let line_num = if is_relative {
                        if is_cursor_line {
                            row_index + 1
                        } else {
                            row_index.abs_diff(cursor.row)
                        }
                    } else {
                        row_index + 1
                    };

                    // Right-align line numbers, but left-align current line
                    let num_str = if is_relative && is_cursor_line {
                        format!(
                            "{:<width$}",
                            line_num,
                            width = (line_number_width - 1) as usize
                        )
                    } else {
                        format!(
                            "{:>width$}",
                            line_num,
                            width = (line_number_width - 1) as usize
                        )
                    };
                    let num_span = Span::styled(num_str, line_numbers_style);

                    let line_num_area = Rect::new(gutter.x, gutter.y, gutter.width, 1);
                    buf.set_span(
                        line_num_area.x,
                        line_num_area.y,
                        &num_span,
                        line_num_area.width,
                    );

                    let num_lines = render_line.num_lines() as u16;
                    gutter_row_area = Some(Rect::new(
                        gutter.x,
                        gutter.y.saturating_add(num_lines),
                        gutter.width,
                        gutter.height.saturating_sub(num_lines),
                    ));
                }
            }

            // Determine the cursor position.
            if row_index == cursor.row {
                cursor_position = Some(render_line.data_coordinate_to_screen_coordinate(
                    cursor.col.saturating_sub(offset_x),
                    content_area,
                    tab_width,
                ));
            }

            // Render the current line.
            content_area = {
                let num_lines = render_line.num_lines();
                render_line.render(content_area, buf, tab_width);
                rect_indent_y(content_area, num_lines)
            };

            row_index += 1;
        }

        // Render the cursor on top.
        if let Some(cell) = buf.cell_mut(cursor_position.unwrap_or(Position::new(
            content_main.left(),
            content_main.top() + self.state.cursor.row as u16,
        ))) {
            cell.set_style(self.theme.cursor_style);
        }

        // Save the total number of lines that are currently displayed on the viewport.
        // Required to handle scrolling.
        self.state.view.update_num_rows(num_rendered_rows);

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

fn generate_spans<'a>(
    line: &[char],
    selections: &[&Option<Selection>],
    row_index: usize,
    col_skips: usize,
    base_style: &Style,
    highlight_style: &Style,
    #[cfg(feature = "syntax-highlighting")] syntax_highlighter: Option<&SyntaxHighlighter>,
) -> Vec<Span<'a>> {
    #[cfg(feature = "syntax-highlighting")]
    if let Some(syntax) = syntax_highlighter {
        return line_into_highlighted_spans_with_selections(
            line,
            selections,
            syntax,
            row_index,
            col_skips,
            base_style,
            highlight_style,
        );
    }
    line_into_spans_with_selections(
        line,
        selections,
        row_index,
        col_skips,
        base_style,
        highlight_style,
    )
}

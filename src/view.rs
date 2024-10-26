mod internal;
pub(crate) mod line_wrapper;
pub mod status_line;
#[cfg(feature = "syntax-highlighting")]
pub(crate) mod syntax_higlighting;
pub mod theme;

#[cfg(feature = "syntax-highlighting")]
use syntax_higlighting::SyntaxHighlighter;

use crate::{helper::max_col, state::EditorState, EditorMode, Index2};
use internal::{find_position_in_spans, find_position_in_wrapped_spans, InternalLine, RenderLine};
use jagged::index::RowIndex;
use line_wrapper::LineWrapper;
use ratatui::{prelude::*, widgets::Widget};
pub use status_line::EditorStatusLine;
use std::cmp::min;
use theme::EditorTheme;

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
    pub(crate) state: &'a mut EditorState,
    pub(crate) theme: EditorTheme<'b>,
    pub(crate) wrap: bool,
    #[cfg(feature = "syntax-highlighting")]
    pub(crate) syntax_highlighter: Option<SyntaxHighlighter>,
    pub(crate) tab_width: usize,
}

impl<'a, 'b> EditorView<'a, 'b> {
    /// Creates a new instance of [`EditorView`].
    #[must_use]
    pub fn new(state: &'a mut EditorState) -> Self {
        Self {
            state,
            theme: EditorTheme::default(),
            wrap: true,
            #[cfg(feature = "syntax-highlighting")]
            syntax_highlighter: None,
            tab_width: 2,
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
    /// #[cfg(feature = "syntax-highlighting")]
    /// {
    ///     use edtui::EditorState;
    ///     use edtui::EditorView;
    ///     use edtui::SyntaxHighlighter;
    ///
    ///     let syntax_highlighter = SyntaxHighlighter::new("dracula", "rs");
    ///     EditorView::new(&mut EditorState::default())
    ///         .syntax_highlighter(Some(syntax_highlighter));
    /// }
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
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Configures the number of spaces that are used to render at tab.
    #[must_use]
    pub fn tab_width(mut self, tab_width: usize) -> Self {
        self.tab_width = tab_width;
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
        let width = main.width as usize;
        let height = main.height as usize;
        let lines = &self.state.lines;

        // Retrieve the displayed cursor position. The column of the displayed
        // cursor is clamped to the maximum line length.
        let cursor = displayed_cursor(self.state);

        // Store the offset from the current buffer to the textarea inside the state.
        // This is required to calculate mouse positions correctly.
        self.state.view.set_editor_to_textarea_offset(area);

        // Set how many spaces are used to render a tab.
        self.state.view.tab_width = self.tab_width;

        // Update the view offset. Requuires the screen size and the position
        // of the cursor. Updates the view offset only if the cursor is out
        // side of the view port. The state is stored in the `ViewOffset`.
        let view_state = &mut self.state.view;
        let offset_y = if self.wrap {
            view_state.update_viewport_vertical_wrap(width, height, cursor.row, lines)
        } else {
            view_state.update_viewport_vertical(height, cursor.row)
        };
        let offset_x = if self.wrap {
            0
        } else {
            let line = lines.get(RowIndex::new(cursor.row));
            view_state.update_viewport_horizontal(width, cursor.col, line)
        };

        // Predetermine highlighted sections.
        let mut search_selection = None;
        if self.state.mode == EditorMode::Search {
            search_selection = self.state.search.selected_range();
        };
        let selections = vec![&self.state.selection, &search_selection];

        let mut y = main.top();
        let mut num_rows: usize = 0;
        for (i, line) in lines.iter_row().skip(offset_y).enumerate() {
            let row_index = offset_y + i;
            num_rows += 1;

            let internal_line = InternalLine::new(
                line,
                self.theme.base,
                self.theme.selection_style,
                offset_y + i,
                offset_x,
            );

            #[cfg(feature = "syntax-highlighting")]
            let spans = if let Some(syntax_highlighter) = &self.syntax_highlighter {
                internal_line.into_highlighted_spans(&selections, syntax_highlighter)
            } else {
                internal_line.into_spans(&selections)
            };
            #[cfg(not(feature = "syntax-highlighting"))]
            let spans = { internal_line.into_spans(&selections) };

            let display_line = if self.wrap {
                RenderLine::Wrapped(LineWrapper::wrap_spans(
                    spans,
                    main.width as usize,
                    self.tab_width,
                ))
            } else {
                RenderLine::Single(spans)
            };

            // Determine the cursor position.
            let cursor_position_on_screen = if row_index == cursor.row {
                let cursor_position = match display_line {
                    RenderLine::Wrapped(ref lines) => find_position_in_wrapped_spans(
                        lines,
                        cursor.col,
                        main.width as usize,
                        self.tab_width,
                    ),

                    RenderLine::Single(ref line) => Index2::new(
                        0,
                        find_position_in_spans(
                            line,
                            cursor.col.saturating_sub(offset_x),
                            self.tab_width,
                        ),
                    ),
                };
                Some(Position::new(
                    main.left() + min(width, cursor_position.col) as u16,
                    y + cursor_position.row as u16,
                ))
            } else {
                None
            };

            // Rendering the content line by line.
            match display_line {
                RenderLine::Wrapped(lines) if lines.is_empty() => {
                    y += 1;
                }
                RenderLine::Wrapped(lines) => {
                    for line in lines {
                        let area = Rect::new(main.left(), y, main.width, main.height);
                        render_line(area, buf, line, self.tab_width);
                        y += 1;

                        if y >= main.bottom() {
                            break;
                        }
                    }
                }
                RenderLine::Single(line) => {
                    let area = Rect::new(main.left(), y, main.width, main.height);
                    render_line(area, buf, line, self.tab_width);
                    y += 1;
                }
            }

            // Render the cursor on top.
            if let Some(cursor_position) = cursor_position_on_screen {
                if let Some(cell) = buf.cell_mut(cursor_position) {
                    cell.set_style(self.theme.cursor_style);
                }
            }

            // Increment y after rendering the current line.
            if y >= main.bottom() {
                break;
            }
        }

        // Render the cursor even if the editor has no content,
        if num_rows == 0 {
            if let Some(cell) = buf.cell_mut(Position::new(main.left(), main.top())) {
                cell.set_style(self.theme.cursor_style);
            }
        // Render the cursor if the cursor is out of bounds.
        } else if self.state.cursor.row + 1 > self.state.lines.len() {
            if let Some(cell) = buf.cell_mut(Position::new(
                main.left(),
                main.top() + self.state.cursor.row as u16,
            )) {
                cell.set_style(self.theme.cursor_style);
            }
        }

        // Save the total number of lines that are currentyl displayed on the viewport.
        // Needed to handle scrolling.
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

fn render_line(area: Rect, buf: &mut Buffer, spans: Vec<Span>, tab_width: usize) {
    let mut line: Line = spans.into_iter().collect();
    // Replace tabs
    for span in &mut line.spans {
        span.content = span.content.replace('\t', &" ".repeat(tab_width)).into();
    }
    line.render(area, buf);
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

mod internal;
pub(crate) mod line_wrapper;
mod render_line;
pub mod status_line;
#[cfg(feature = "syntax-highlighting")]
pub(crate) mod syntax_higlighting;
pub mod theme;

use render_line::RenderLine;
#[cfg(feature = "syntax-highlighting")]
use syntax_higlighting::SyntaxHighlighter;

use crate::{
    helper::{max_col, rect_indent_y},
    state::{selection::Selection, EditorState},
    EditorMode, Index2,
};

use internal::into_spans_with_selections;
#[cfg(feature = "syntax-highlighting")]
use internal::line_into_highlighted_spans_with_selections;
use jagged::index::RowIndex;
use line_wrapper::LineWrapper;
use ratatui::{prelude::*, widgets::Widget};
pub use status_line::EditorStatusLine;
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
    pub fn wrap(self, wrap: bool) -> Self {
        self.state.view.wrap = wrap;
        self
    }

    pub(super) fn get_wrap(&self) -> bool {
        self.state.view.wrap
    }

    /// Configures the number of spaces that are used to render at tab.
    #[must_use]
    pub fn tab_width(self, tab_width: usize) -> Self {
        self.state.view.tab_width = tab_width;
        self
    }

    pub(super) fn get_tab_width(&self) -> usize {
        self.state.view.tab_width
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
        let wrap_lines = self.get_wrap();
        let tab_width = self.get_tab_width();
        let lines = &self.state.lines;

        // Retrieve the displayed cursor position. The column of the displayed
        // cursor is clamped to the maximum line length.
        let max_col = max_col(&self.state.lines, &self.state.cursor, self.state.mode);
        let cursor = Index2::new(self.state.cursor.row, self.state.cursor.col.min(max_col));

        // Store the coordinats of the current editor.
        self.state.view.set_screen_area(area);

        // Update the view offset. Requuires the screen size and the position
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
        let mut content_area = main;
        let mut num_rendered_rows = 0;

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
            main.left(),
            main.top() + self.state.cursor.row as u16,
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
            highlight_style,
        );
    }
    into_spans_with_selections(
        line,
        selections,
        row_index,
        col_skips,
        base_style,
        highlight_style,
    )
}

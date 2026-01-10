use super::theme::{DARK_GRAY, WHITE};
use ratatui_core::layout::{Constraint, HorizontalAlignment, Layout};
use ratatui_core::{buffer::Buffer, layout::Rect, style::Style, text::Span, widgets::Widget};
use ratatui_widgets::block::Block;

/// An optional status line for Editor.
#[derive(Debug, Clone)]
pub struct EditorStatusLine {
    /// Displays the current editor mode in the status line.
    mode: String,
    /// The current search buffer. Shown only in search mode.
    search: Option<String>,
    /// The style for the mode of the status line
    style_mode: Option<Style>,
    /// The style for the search of the status line
    style_search: Option<Style>,
    /// The style for the line itself
    style_line: Style,
    /// Horizontal alignment of the status bar
    alignment: HorizontalAlignment,
}

impl Default for EditorStatusLine {
    /// Creates a new instance of [`EditorStatusLine`].
    ///
    /// This constructor initializes with default style.
    fn default() -> Self {
        Self {
            mode: String::new(),
            search: None,
            style_mode: Some(Style::default().fg(WHITE).bg(DARK_GRAY).bold()),
            style_search: Some(Style::default().fg(WHITE).bg(DARK_GRAY)),
            style_line: Style::default().fg(WHITE).bg(DARK_GRAY),
            alignment: HorizontalAlignment::Left,
        }
    }
}

impl EditorStatusLine {
    /// Overwrite the style for the status lines mode.
    ///
    /// This method allows you to customize the appearance of the
    /// status lines mode.
    #[must_use]
    pub fn style_mode(mut self, style: Option<Style>) -> Self {
        self.style_mode = style;
        self
    }

    /// Overwrite the style for the status lines search.
    ///
    /// This method allows you to customize the appearance of the
    /// status lines search.
    #[must_use]
    pub fn style_search(mut self, style: Option<Style>) -> Self {
        self.style_search = style;
        self
    }

    /// Overwrite the style for the status lines.
    ///
    /// This method allows you to customize the appearance of the
    /// status line.
    #[must_use]
    pub fn style_line(mut self, style: Style) -> Self {
        self.style_line = style;
        self
    }

    /// Overwrite the mode content for the status line.
    ///
    /// This method is used internally to dynamically set the editors mode.
    #[must_use]
    pub fn mode<S: Into<String>>(mut self, mode: S) -> Self {
        self.mode = mode.into();
        self
    }

    /// Overwrite the search content for the status line.
    ///
    /// This method is used internally to dynamically set the editors mode.
    #[must_use]
    pub fn search<S: Into<String>>(mut self, search: Option<S>) -> Self {
        self.search = search.map(Into::into);
        self
    }

    #[deprecated(
        since = "0.10.3",
        note = "Please use `alignment(HorizontalAlignment::Left)` or `alignment(HorizontalAlignment::Right)` instead"
    )]
    pub fn align_left(self, align_left: bool) -> Self {
        let alignment = match align_left {
            true => HorizontalAlignment::Left,
            false => HorizontalAlignment::Right,
        };
        self.alignment(alignment)
    }

    /// Set the alignment for the status line content.
    #[must_use]
    pub fn alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl Widget for EditorStatusLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let constraints = match self.alignment {
            HorizontalAlignment::Left => vec![Constraint::Length(10), Constraint::Min(1)],
            HorizontalAlignment::Center => vec![
                Constraint::Min(1),
                Constraint::Length(10),
                Constraint::Min(1),
            ],
            HorizontalAlignment::Right => vec![Constraint::Min(1), Constraint::Length(10)],
        };

        let layout = Layout::horizontal(constraints).split(area);

        let search_text = match self.search {
            None => String::new(),
            Some(search) => format!("/{search}"),
        };

        let mode_span = Span::raw(format!("{:^10}", self.mode))
            .style(self.style_mode.unwrap_or(self.style_line));
        let search_span =
            Span::raw(search_text).style(self.style_search.unwrap_or(self.style_line));

        let line_block = Block::new().style(self.style_line);

        line_block.render(area, buf);

        match self.alignment {
            HorizontalAlignment::Left => {
                mode_span.render(layout[0], buf);
                search_span.render(layout[1], buf);
            }
            HorizontalAlignment::Center => {
                mode_span.render(layout[1], buf);
                search_span.render(layout[2], buf);
            }
            HorizontalAlignment::Right => {
                mode_span.render(layout[1], buf);
                search_span.render(layout[0], buf);
            }
        }
    }
}

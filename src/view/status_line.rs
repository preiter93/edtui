use ratatui_core::layout::HorizontalAlignment;
use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};
use ratatui_widgets::paragraph::Paragraph;

use super::theme::{DARK_GRAY, WHITE};

/// An optional status line for Editor.
#[derive(Debug, Clone)]
pub struct EditorStatusLine {
    /// Displays the current editor mode in the status line.
    mode: String,
    /// The current search buffer. Shown only in search mode.
    search: Option<String>,
    /// The style for the mode of the status line
    style_mode: Style,
    /// The style for the search of the status line
    style_search: Style,
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
            style_mode: Style::default().fg(WHITE).bg(DARK_GRAY).bold(),
            style_search: Style::default().fg(WHITE).bg(DARK_GRAY),
            style_line: Style::default().fg(WHITE).bg(DARK_GRAY),
            alignment: HorizontalAlignment::Center,
        }
    }
}

impl EditorStatusLine {
    /// Overwrite the style for the status lines mode.
    ///
    /// This method allows you to customize the appearance of the
    /// status lines mode.
    #[must_use]
    pub fn style_mode(mut self, style: Style) -> Self {
        self.style_mode = style;
        self
    }

    /// Overwrite the style for the status lines search.
    ///
    /// This method allows you to customize the appearance of the
    /// status lines search.
    #[must_use]
    pub fn style_search(mut self, style: Style) -> Self {
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
        let search_text = match self.search {
            None => String::new(),
            Some(search) => format!("/{search}"),
        };

        let search_span = Span::raw(search_text).style(self.style_search);
        let mode_span = Span::raw(format!("{:^10}", self.mode)).style(self.style_mode);

        let mode_line = Line::from(vec![mode_span, search_span]).alignment(self.alignment);
        let mode_paragraph = Paragraph::new(mode_line).style(self.style_line);

        mode_paragraph.render(area, buf);
    }
}

use ratatui_core::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
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
    /// The style for the content of the sidebar
    style_text: Style,
    /// The style for the line itself
    style_line: Style,
    // Whether to align content to the left (true) or the right (false)
    align_left: bool,
}

impl Default for EditorStatusLine {
    /// Creates a new instance of [`EditorStatusLine`].
    ///
    /// This constructor initializes with default style.
    fn default() -> Self {
        Self {
            mode: String::new(),
            search: None,
            style_text: Style::default().fg(WHITE).bg(DARK_GRAY).bold(),
            style_line: Style::default().fg(WHITE).bg(DARK_GRAY),
            align_left: true,
        }
    }
}

impl EditorStatusLine {
    /// Overwrite the style for the status lines content.
    ///
    /// This method allows you to customize the appearance of the
    /// status lines content.
    #[must_use]
    pub fn style_text(mut self, style: Style) -> Self {
        self.style_text = style;
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

    /// Set the alignment for the status line content.
    ///
    /// Set to true to align content to the left, false to align to the right.
    #[must_use]
    pub fn align_left(mut self, align_left: bool) -> Self {
        self.align_left = align_left;
        self
    }
}

impl Widget for EditorStatusLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split the layout horizontally.
        let constraints = if self.align_left {
            [Constraint::Length(10), Constraint::Min(0)]
        } else {
            [Constraint::Min(0), Constraint::Length(10)]
        };
        let [left, right] = Layout::horizontal(constraints).areas(area);

        // Build the content and block widgets
        let mode_paragraph = Paragraph::new(Line::from(Span::from(self.mode)))
            .alignment(Alignment::Center)
            .style(self.style_text);
        let search_text = self.search.map_or(String::new(), |s| format!("/{s}"));
        let search_paragraph = Paragraph::new(Line::from(Span::from(search_text)))
            .alignment(Alignment::Left)
            .style(self.style_line);

        // Determine the alignment position
        if self.align_left {
            mode_paragraph.render(left, buf);
            search_paragraph.render(right, buf);
        } else {
            search_paragraph.render(left, buf);
            mode_paragraph.render(right, buf);
        };
    }
}

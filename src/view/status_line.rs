use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, Paragraph, Widget},
};

use super::theme::{DARK_PURPLE, LIGHT_PURPLE};

/// An optional status line for Editor.
#[derive(Debug, Clone)]
pub struct StatusLine {
    /// The displayed text in the status line. Used internally to
    /// display the editor mode.
    content: String,
    /// The style for the content of the sidebar
    style_text: Style,
    /// The style for the line itself
    style_line: Style,
    // Whether to align content to the left (true) or the right (false)
    align_left: bool,
}

impl Default for StatusLine {
    /// Creates a new instance of [`StatusLine`].
    ///
    /// This constructor initializes with default style.
    fn default() -> Self {
        Self {
            content: String::new(),
            style_text: Style::default().fg(Color::White).bg(DARK_PURPLE).bold(),
            style_line: Style::default().bg(LIGHT_PURPLE),
            align_left: true,
        }
    }
}

impl StatusLine {
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

    /// Overwrite the content for the status line.
    ///
    /// This method is used internally to dynamically set the editors mode.
    #[must_use]
    pub fn content<S: Into<String>>(mut self, content: S) -> Self {
        self.content = content.into();
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

impl Widget for StatusLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split the layout horizontally.
        let constraints = if self.align_left {
            [Constraint::Length(10), Constraint::Min(0)]
        } else {
            [Constraint::Min(0), Constraint::Length(10)]
        };
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Build the content and block widgets
        let text = Paragraph::new(Line::from(Span::styled(self.content, self.style_text)))
            .alignment(Alignment::Center)
            .style(self.style_text);
        let block = Block::default().style(self.style_line);

        // Determine the alignment position
        if self.align_left {
            text.render(area[0], buf);
            block.render(area[1], buf);
        } else {
            block.render(area[0], buf);
            text.render(area[1], buf);
        };
    }
}

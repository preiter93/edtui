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
    /// The style for the content of the sidebar
    style_text: Style,
    /// The style for the line itself
    style_line: Style,
    // Whether to align content to the left (true) or the right (false)
    alignement: HorizontalAlignment,
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
            alignement: HorizontalAlignment::Center,
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
    pub fn alignement(mut self, alignement: HorizontalAlignment) -> Self {
        self.alignement = alignement;
        self
    }
}

impl Widget for EditorStatusLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Build the content and block widgets
        let text = match self.search {
            None => self.mode,
            Some(search) => format!("{} /{}", self.mode, search),
        };

        let mode_line = Line::from(Span::from(text))
            .alignment(self.alignement)
            .style(self.style_text);

        let mode_paragraph = Paragraph::new(mode_line).style(self.style_line);

        mode_paragraph.render(area, buf);
    }
}

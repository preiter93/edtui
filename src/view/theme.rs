use super::StatusLine;
use ratatui::{
    style::{Color, Style},
    widgets::Block,
};

/// The theme data of the Editor.
pub struct EditorTheme<'a> {
    /// The base text style
    pub base: Style,
    /// The cursor style
    pub cursor_style: Style,
    /// The text style in visual mode when a text is selected
    pub selection_style: Style,
    /// The surrounding block
    pub block: Option<Block<'a>>,
    /// An optional [`StatusLine`] displaying the editor mode
    pub status_line: Option<StatusLine>,
}

impl Default for EditorTheme<'_> {
    /// Creates a new instance of [`EditorTheme`].
    ///
    /// This constructor initializes with default style.
    fn default() -> Self {
        Self {
            base: Style::default().bg(DARK_BLUE),
            block: None,
            cursor_style: Style::default()
                .bg(ratatui::style::Color::White)
                .fg(ratatui::style::Color::Black),
            selection_style: Style::default()
                .bg(ratatui::style::Color::Indexed(220))
                .fg(ratatui::style::Color::Black),
            status_line: Some(StatusLine::default()),
        }
    }
}

impl<'a> EditorTheme<'a> {
    /// This method allows you to customize the base appearance of the
    /// Editor.
    #[must_use]
    pub fn base(mut self, base: Style) -> Self {
        self.base = base;
        self
    }

    /// Returns the base style.
    #[must_use]
    pub fn base_style(&self) -> Style {
        self.base
    }

    /// This method allows you to customize the block surrrounding
    /// the Editor.
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// This method allows you to customize the style of the cursor of
    /// the Editor.
    #[must_use]
    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    /// This method allows you to customize the style of the selection of
    /// the Editor in visual mode.
    #[must_use]
    pub fn selection_style(mut self, style: Style) -> Self {
        self.selection_style = style;
        self
    }

    /// This method allows you to customize the style of the [`StatusLine`]
    /// of the Editor. See [`StatusLine`] on how to modify its appearance.
    /// If None, no status line is shown.
    #[must_use]
    pub fn status_line(mut self, status_line: Option<StatusLine>) -> Self {
        self.status_line = status_line;
        self
    }
}

pub(crate) const DARK_BLUE: Color = Color::Rgb(16, 24, 48);
pub(crate) const DARK_PURPLE: Color = Color::Indexed(55);
pub(crate) const LIGHT_PURPLE: Color = Color::Indexed(93);

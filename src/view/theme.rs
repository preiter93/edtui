use super::EditorStatusLine;
use ratatui_core::style::{Color, Style};
use ratatui_widgets::block::Block;

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
    pub status_line: Option<EditorStatusLine>,
}

impl Default for EditorTheme<'_> {
    /// Creates a new instance of [`EditorTheme`].
    ///
    /// This constructor initializes with default style.
    fn default() -> Self {
        Self {
            base: Style::default().bg(BLACK).fg(WHITE),
            block: None,
            cursor_style: Style::default().bg(WHITE).fg(BLACK),
            selection_style: Style::default().bg(YELLOW).fg(BLACK),
            status_line: Some(EditorStatusLine::default()),
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

    /// Hides the cursors.
    #[must_use]
    pub fn hide_cursor(mut self) -> Self {
        self.cursor_style = self.base;
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
    /// Use `hide_status_line` to hide the status line.
    #[must_use]
    pub fn status_line(mut self, status_line: EditorStatusLine) -> Self {
        self.status_line = Some(status_line);
        self
    }

    /// Hides the status lilne.
    #[must_use]
    pub fn hide_status_line(mut self) -> Self {
        self.status_line = None;
        self
    }
}

pub(crate) const WHITE: Color = Color::Rgb(255, 255, 255);
pub(crate) const BLACK: Color = Color::Rgb(0, 0, 0);
pub(crate) const DARK_GRAY: Color = Color::Rgb(16, 17, 22);
pub(crate) const YELLOW: Color = Color::Rgb(250, 204, 21);

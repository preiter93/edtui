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
    /// Style for line numbers (subdued by default)
    pub line_numbers_style: Style,
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
            line_numbers_style: Style::default().bg(BLACK).fg(GRAY),
        }
    }
}

impl<'a> EditorTheme<'a> {
    /// This method allows you to customize the base appearance of the
    /// Editor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::EditorTheme;
    /// use ratatui::style::{Style, Color};
    ///
    /// let theme = EditorTheme::default()
    ///     .base(Style::default().fg(Color::White).bg(Color::Black));
    /// ```
    #[must_use]
    pub fn base(mut self, base: Style) -> Self {
        self.base = base;
        self
    }

    /// Returns the base style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::EditorTheme;
    ///
    /// let theme = EditorTheme::default();
    /// let base_style = theme.base_style();
    /// ```
    #[must_use]
    pub fn base_style(&self) -> Style {
        self.base
    }

    /// This method allows you to customize the block surrounding
    /// the Editor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::EditorTheme;
    /// use ratatui::widgets::Block;
    ///
    /// let theme = EditorTheme::default()
    ///     .block(Block::bordered().title("Editor"));
    /// ```
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// This method allows you to customize the style of the cursor of
    /// the Editor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::EditorTheme;
    /// use ratatui::style::{Style, Color, Modifier};
    ///
    /// let theme = EditorTheme::default()
    ///     .cursor_style(
    ///         Style::default()
    ///             .bg(Color::White)
    ///             .fg(Color::Black)
    ///             .add_modifier(Modifier::REVERSED),
    ///     );
    /// ```
    #[must_use]
    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    /// Hides the cursor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::EditorTheme;
    ///
    /// let theme = EditorTheme::default()
    ///     .hide_cursor();
    /// ```
    #[must_use]
    pub fn hide_cursor(mut self) -> Self {
        self.cursor_style = self.base;
        self
    }

    /// This method allows you to customize the style of the selection of
    /// the Editor in visual mode.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::EditorTheme;
    /// use ratatui::style::{Style, Color};
    ///
    /// let theme = EditorTheme::default()
    ///     .selection_style(Style::default().bg(Color::Blue));
    /// ```
    #[must_use]
    pub fn selection_style(mut self, style: Style) -> Self {
        self.selection_style = style;
        self
    }

    /// This method allows you to customize the style of the [`StatusLine`]
    /// of the Editor. See [`StatusLine`] on how to modify its appearance.
    /// Use `hide_status_line` to hide the status line.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::{EditorTheme, EditorStatusLine};
    ///
    /// let status_line = EditorStatusLine::default();
    ///
    /// let theme = EditorTheme::default()
    ///     .status_line(status_line);
    /// ```
    #[must_use]
    pub fn status_line(mut self, status_line: EditorStatusLine) -> Self {
        self.status_line = Some(status_line);
        self
    }

    /// Hides the status line.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::EditorTheme;
    ///
    /// let theme = EditorTheme::default()
    ///     .hide_status_line();
    /// ```
    #[must_use]
    pub fn hide_status_line(mut self) -> Self {
        self.status_line = None;
        self
    }

    /// Customize the style of the line numbers.
    /// By default, line numbers are displayed in a subdued gray color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use edtui::EditorTheme;
    /// use ratatui::style::{Style, Color};
    ///
    /// let theme = EditorTheme::default()
    ///     .line_numbers_style(Style::default().fg(Color::DarkGray));
    /// ```
    #[must_use]
    pub fn line_numbers_style(mut self, style: Style) -> Self {
        self.line_numbers_style = style;
        self
    }
}

pub(crate) const WHITE: Color = Color::Rgb(255, 255, 255);
pub(crate) const BLACK: Color = Color::Rgb(0, 0, 0);
pub(crate) const DARK_GRAY: Color = Color::Rgb(16, 17, 22);
pub(crate) const YELLOW: Color = Color::Rgb(250, 204, 21);
pub(crate) const GRAY: Color = Color::Rgb(100, 100, 100);

use crate::syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};
use once_cell::sync::Lazy;
use ratatui_core::style::{Color, Style};
use syntect::dumps::from_binary;

use super::internal::InternalSpan;

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(load_defaults);

pub fn load_defaults() -> ThemeSet {
    from_binary(include_bytes!("../../assets/default.themedump"))
}

/// Syntax highlighter settings including theme and syntax.
pub struct SyntaxHighlighter {
    theme: Theme,
    syntax_ref: SyntaxReference,
}

impl SyntaxHighlighter {
    /// Creates a new [`SyntaxHighlighter`] with a given theme (e.g. "base16-ocean.dark")
    /// and an extension (e.g. "json").
    ///
    /// Syntax highlighting is currently highly experimental, and there might be breaking
    /// changes in the future.
    ///
    /// See [`Self::theme`] for a list of available themes.
    ///
    /// # Panics
    /// - Could not find `theme` in syntect.
    /// - Could not find `extension` in syntect.
    ///
    /// ## Example
    ///
    /// ```
    /// use edtui::SyntaxHighlighter;
    ///
    /// let syntax_highlighter = SyntaxHighlighter::new("dracula", "rs");
    /// ```
    #[must_use]
    pub fn new(theme: &str, extension: &str) -> Self {
        let theme = THEME_SET
            .themes
            .get(theme)
            .unwrap_or_else(|| panic!("Could not find theme {theme}"))
            .clone();
        let syntax_ref = SYNTAX_SET
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| panic!("Could not find extension {extension}"))
            .clone();

        Self { theme, syntax_ref }
    }

    /// Set a custom theme. If you would like to use a predefined
    /// theme use [`theme_by_name`].
    #[must_use]
    pub fn custom_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set a theme by name, e.g. , e.g. "base16-ocean.dark".
    ///
    /// # Panics
    /// - Could not find `theme` in syntect.
    ///
    /// # Available themes
    /// "`1337`"
    /// "`OneHalfDark`"
    /// "`OneHalfLight`"
    /// "`Tomorrow`"
    /// "`agola-dark`"
    /// "`ascetic-white`"
    /// "`axar`"
    /// "`ayu-dark`"
    /// "`ayu-light`"
    /// "`ayu-mirage`"
    /// "`base16-atelierdune-light`"
    /// "`base16-ocean-dark`"
    /// "`base16-ocean-light`"
    /// "`bbedit`"
    /// "`boron`"
    /// "`charcoal`"
    /// "`cheerfully-light`"
    /// "`classic-modified`"
    /// "`demain`"
    /// "`dimmed-fluid`"
    /// "`dracula`"
    /// "`gray-matter-dark`"
    /// "`green`"
    /// "`gruvbox-dark`"
    /// "`gruvbox-light`"
    /// "`idle`"
    /// "`inspired-github`"
    /// "`ir-white`"
    /// "`kronuz`"
    /// "`material-dark`"
    /// "`material-light`"
    /// "`monokai`"
    /// "`nord`"
    /// "`nyx-bold`"
    /// "`one-dark`"
    /// "`railsbase16-green-screen-dark`"
    /// "`solarized-dark`"
    /// "`solarized-light`"
    /// "`subway-madrid`"
    /// "`subway-moscow`"
    /// "`two-dark`"
    /// "`visual-studio-dark`"
    /// "`zenburn`"
    #[must_use]
    pub fn theme(mut self, theme: &str) -> Self {
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set
            .themes
            .get(theme)
            .unwrap_or_else(|| panic!("Could not find theme {theme}"));
        self.theme = theme.clone();
        self
    }

    /// Set the active extension for syntax highlighting, e.g. "json".
    ///
    /// # Panics
    /// - Could not find `theme` in syntect.
    /// - Could not find `extension` in syntect.
    #[must_use]
    pub(crate) fn extension(mut self, extension: &str) -> Self {
        let syntax_ref = SYNTAX_SET
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| panic!("Could not find extension {extension}"));
        self.syntax_ref = syntax_ref.clone();
        self
    }
}

impl SyntaxHighlighter {
    pub(super) fn highlight_line(&self, line: &str) -> Vec<InternalSpan> {
        // Highlight lines
        let mut highlight_lines = HighlightLines::new(&self.syntax_ref, &self.theme);
        let highlighted_line = highlight_lines.highlight_line(line, &SYNTAX_SET).unwrap();

        // Convert the highlighted lines into spans
        let mut spans = Vec::new();
        for &(style, text) in &highlighted_line {
            spans.push(InternalSpan::new(
                text.to_string(),
                &Style::default().fg(Color::Rgb(
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                )),
            ));
        }

        spans
    }
}

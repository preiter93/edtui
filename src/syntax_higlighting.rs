use crate::internal::InternalSpan;
use crate::syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};
use once_cell::sync::Lazy;
use ratatui::style::{Color, Style};
use syntect::dumps::from_binary;

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(load_defaults);

// Themes from
// https://git.data.coop/emelie/zola/src/commit/b0937fa5b78fa927febdb94f6f6b568249663623/sublime/themes
pub fn load_defaults() -> ThemeSet {
    from_binary(include_bytes!("../assets/edtui.themedump"))
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
            .expect(&format!("Could not find theme {theme}"))
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
    pub(crate) fn custom_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set a theme by name, e.g. , e.g. "base16-ocean.dark".
    ///
    /// # Panics
    /// - Could not find `theme` in syntect.
    #[must_use]
    pub(crate) fn theme_by_name(mut self, theme: &str) -> Self {
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set
            .themes
            .get(theme)
            .expect(&format!("Could not find theme {theme}"));
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

use crate::syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};
use once_cell::sync::Lazy;
use ratatui_core::style::{Color, Style};
use syntect::dumps::from_binary;
use thiserror::Error;
use crate::view::syntax_higlighting::SyntaxHighlighterError::{ExtensionNotFound, ThemeNotFound};
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

#[derive(Error, Debug)]
pub enum SyntaxHighlighterError<'a> {
    #[error("Could not find theme {0}")]
    ThemeNotFound(&'a str),

    #[error("Could not find extension {0}")]
    ExtensionNotFound(&'a str),
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
    /// ## Example
    ///
    /// ```
    /// use edtui::SyntaxHighlighter;
    ///
    /// let syntax_highlighter = SyntaxHighlighter::new("dracula", "rs");
    /// ```
    #[must_use]
    pub fn new<'a>(theme: &'a str, extension: &'a str) -> Result<Self, SyntaxHighlighterError<'a>> {
        let theme = match THEME_SET.themes.get(theme) {
            Some(v) => v.clone(),
            None => return Err(ThemeNotFound(theme))
        };

        let syntax_ref = match SYNTAX_SET.find_syntax_by_extension(extension) {
            Some(v) => v.clone(),
            None =>return Err(ExtensionNotFound(extension))
        };

        Ok(Self {
            theme,
            syntax_ref
        })
    }

    /// Creates a new [`SyntaxHighlighter`] with a given custom given Theme and SyntaxReference
    ///
    /// Syntax highlighting is currently highly experimental, and there might be breaking
    /// changes in the future.
    ///
    /// ## Example
    ///
    /// ```
    /// use syntect::highlighting::Theme;
    /// use syntect::parsing::{SyntaxDefinition, SyntaxSetBuilder};
    /// use edtui::SyntaxHighlighter;
    ///
    /// let theme = Theme::default(); // My custom theme
    /// let mut syntax_set_builder = SyntaxSetBuilder::new(); // My custom syntax set builder
    /// syntax_set_builder.add_plain_text_syntax(); // Add your syntax
    /// let syntax_set = syntax_set_builder.build(); // My custom syntax set
    /// let syntax_ref = syntax_set.syntaxes().first().unwrap().clone(); // My custom syntax reference
    /// let syntax_highlighter = SyntaxHighlighter::new_custom(theme, syntax_ref);
    /// ```
    #[must_use]
    pub fn new_custom(theme: Theme, syntax_ref: SyntaxReference) -> Self {
        Self { theme, syntax_ref }
    }

    /// Set a custom theme. If you would like to use a predefined
    /// theme use [`theme_by_name`].
    #[must_use]
    pub fn custom_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set a custom theme. If you would like to use a predefined
    /// theme use [`theme_by_name`].
    #[must_use]
    pub fn custom_syntax_ref(mut self, syntax_ref: SyntaxReference) -> Self {
        self.syntax_ref = syntax_ref;
        self
    }

    /// Set a theme by name, e.g. , e.g. "base16-ocean.dark".
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
    pub fn theme(mut self, theme: &'_ str) -> Result<Self, SyntaxHighlighterError<'_>>  {
        let theme = match THEME_SET.themes.get(theme) {
            Some(v) => v.clone(),
            None => return Err(ThemeNotFound(theme))
        };

        self.theme = theme;

        Ok(self)
    }

    /// Set the active extension for syntax highlighting, e.g. "json".
    #[must_use]
    pub(crate) fn extension(mut self, extension: &'_ str) -> Result<Self, SyntaxHighlighterError<'_>> {
        let syntax_ref = match SYNTAX_SET.find_syntax_by_extension(extension) {
            Some(v) => v.clone(),
            None =>return Err(ExtensionNotFound(extension))
        };

        self.syntax_ref = syntax_ref;

        Ok(self)
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

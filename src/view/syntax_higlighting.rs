use super::internal::InternalSpan;
use crate::syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};
use crate::view::syntax_higlighting::SyntaxHighlighterError::{ExtensionNotFound, ThemeNotFound};
use once_cell::sync::Lazy;
use ratatui_core::style::{Color, Style};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use syntect::dumps::from_binary;

pub static SYNTAX_SET: Lazy<Arc<SyntaxSet>> =
    Lazy::new(|| Arc::new(SyntaxSet::load_defaults_newlines()));
pub static THEME_SET: Lazy<Arc<ThemeSet>> = Lazy::new(|| Arc::new(load_default_themes()));

fn load_default_themes() -> ThemeSet {
    from_binary(include_bytes!("../../assets/default.themedump"))
}

/// Syntax highlighter settings including theme and syntax.
pub struct SyntaxHighlighter {
    theme: Theme,
    theme_set: Arc<ThemeSet>,
    syntax_ref: SyntaxReference,
    syntax_set: Arc<SyntaxSet>,
}

#[derive(Debug)]
pub enum SyntaxHighlighterError<'a> {
    ThemeNotFound(&'a str),
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
    pub fn new<'c>(theme: &'c str, extension: &'c str) -> Result<Self, SyntaxHighlighterError<'c>> {
        let theme_set = THEME_SET.clone();
        let syntax_set = SYNTAX_SET.clone();

        let theme = match theme_set.themes.get(theme) {
            Some(v) => v.clone(),
            None => return Err(ThemeNotFound(theme)),
        };

        let syntax_ref = match syntax_set.find_syntax_by_extension(extension) {
            Some(v) => v.clone(),
            None => return Err(ExtensionNotFound(extension)),
        };

        Ok(Self {
            theme,
            theme_set,
            syntax_ref,
            syntax_set,
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
    /// use edtui::{THEME_SET, SYNTAX_SET};
    ///
    /// let theme = Theme::default(); // My custom theme
    /// let mut syntax_set_builder = SyntaxSetBuilder::new(); // My custom syntax set builder
    /// syntax_set_builder.add_plain_text_syntax(); // Add your syntax
    /// let syntax_set = syntax_set_builder.build(); // My custom syntax set
    /// let syntax_ref = syntax_set.syntaxes().first().unwrap().clone(); // My custom syntax reference
    /// let syntax_highlighter = SyntaxHighlighter::with_sets(theme, THEME_SET.clone(), syntax_ref, SYNTAX_SET.clone());
    /// ```
    #[must_use]
    pub fn with_sets(
        theme: Theme,
        theme_set: Arc<ThemeSet>,
        syntax_ref: SyntaxReference,
        syntax_set: Arc<SyntaxSet>,
    ) -> SyntaxHighlighter {
        Self {
            theme,
            theme_set,
            syntax_ref,
            syntax_set,
        }
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
    pub fn theme(mut self, theme: &'_ str) -> Result<Self, SyntaxHighlighterError<'_>> {
        let theme = match self.theme_set.themes.get(theme) {
            Some(v) => v.clone(),
            None => return Err(ThemeNotFound(theme)),
        };

        self.theme = theme;

        Ok(self)
    }

    /// Set the active extension for syntax highlighting, e.g. "json".
    pub(crate) fn extension(
        mut self,
        extension: &'_ str,
    ) -> Result<Self, SyntaxHighlighterError<'_>> {
        let syntax_ref = match self.syntax_set.find_syntax_by_extension(extension) {
            Some(v) => v.clone(),
            None => return Err(ExtensionNotFound(extension)),
        };

        self.syntax_ref = syntax_ref;

        Ok(self)
    }

    pub(super) fn highlight_line(&self, line: &str, base_style: &Style) -> Vec<InternalSpan> {
        // Highlight lines
        let mut highlight_lines = HighlightLines::new(&self.syntax_ref, &self.theme);
        let mut spans = Vec::new();

        if let Ok(highlighted_line) = highlight_lines.highlight_line(line, &self.syntax_set) {
            // Convert the highlighted lines into spans
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
        } else {
            spans.push(InternalSpan::new(line.to_string(), base_style));
        }

        spans
    }
}

impl Display for SyntaxHighlighterError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeNotFound(theme) => write!(f, "Could not find theme {}", theme),
            ExtensionNotFound(extension) => write!(f, "Could not find extension {}", extension),
        }
    }
}

impl Error for SyntaxHighlighterError<'_> {}

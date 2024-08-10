use edtui::{EditorStatusLine, EditorTheme};
use ratatui::{
    prelude::{Alignment, Stylize},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

pub struct Theme<'a> {
    pub editor: EditorTheme<'a>,
}

impl<'a> Theme<'a> {
    pub fn new() -> Self {
        Self {
            editor: EditorTheme::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .title("Editor")
                        .title_alignment(Alignment::Center),
                )
                .base(Style::default().bg(DARK_BLUE).fg(WHITE))
                .cursor_style(Style::default().bg(WHITE).fg(DARK_BLUE))
                .selection_style(Style::default().bg(YELLOW).fg(DARK_BLUE))
                .status_line(
                    EditorStatusLine::default()
                        .style_text(Style::default().fg(LIGHT_GRAY).bg(LIGHT_PURPLE).bold())
                        .style_text(Style::default().fg(LIGHT_GRAY).bg(DARK_PURPLE))
                        .align_left(true),
                ),
        }
    }
}

pub(crate) const DARK_BLUE: Color = Color::Rgb(15, 23, 42);
pub(crate) const YELLOW: Color = Color::Rgb(250, 204, 21);
pub(crate) const WHITE: Color = Color::Rgb(248, 250, 252);
pub(crate) const LIGHT_GRAY: Color = Color::Rgb(248, 250, 252);
pub(crate) const LIGHT_PURPLE: Color = Color::Rgb(126, 34, 206);
pub(crate) const DARK_PURPLE: Color = Color::Rgb(88, 28, 135);

use edtui::EditorTheme;
use ratatui::{
    prelude::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

#[derive(Default)]
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
                        .title("|Editor|")
                        .title_alignment(Alignment::Center),
                )
                .base(Style::default().bg(DARK_NIGHT).fg(WHITE))
                .cursor_style(Style::default().bg(WHITE).fg(DARK_NIGHT))
                .selection_style(Style::default().bg(ORANGE).fg(DARK_NIGHT))
                .hide_status_line(),
        }
    }
}

pub(crate) const WHITE: Color = Color::Rgb(248, 250, 252);
pub(crate) const DARK_NIGHT: Color = Color::Rgb(16, 17, 22);
pub(crate) const ORANGE: Color = Color::Rgb(255, 153, 0);

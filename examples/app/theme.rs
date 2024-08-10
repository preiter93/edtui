use edtui::EditorTheme;
use ratatui::{
    prelude::{Alignment, Stylize},
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
                .base(EditorTheme::default().base.bold()),
        }
    }
}

use crossterm::event::KeyEvent;
use edtui::{EditorTheme, EditorView};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Widget},
};

use crate::AppContext;

pub struct Root<'a> {
    context: &'a mut AppContext,
}

impl<'a> Root<'a> {
    pub fn new(context: &'a mut AppContext) -> Self {
        Self { context }
    }

    pub fn handle_events(self, event: KeyEvent) {
        let input = &mut self.context.editor_input;
        let state = &mut self.context.editor_state;
        input.on_key(event, state)
    }
}

impl Widget for Root<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0)].as_ref())
            .split(area);
        let theme = Theme::new();

        let state = &mut self.context.editor_state;
        let editor = EditorView::new(state).theme(theme.editor);
        editor.render(chunks[0], buf)
    }
}

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

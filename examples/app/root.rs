use edtui::{EditorTheme, EditorView};
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
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

    pub fn handle_key_events(self, key: KeyEvent) {
        let event_handler = &mut self.context.event_handler;
        let state = &mut self.context.state;
        event_handler.on_key_event(key, state)
    }

    pub fn handle_mouse_events(self, mouse: MouseEvent) {
        let event_handler = &mut self.context.event_handler;
        let state = &mut self.context.state;
        event_handler.on_mouse_event(mouse, state)
    }
}

impl Widget for Root<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = Theme::new();

        let state = &mut self.context.state;
        let editor = EditorView::new(state).theme(theme.editor);
        editor.render(area, buf)
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

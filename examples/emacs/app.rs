use std::error::Error;

use edtui::{EditorEventHandler, EditorState, EditorView};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{prelude::*, widgets::Widget};

use crate::term::Term;
use crate::theme::Theme;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Default)]
pub struct App {
    pub context: AppContext,
    pub should_quit: bool,
}

#[derive(Default)]
pub struct AppContext {
    pub state: EditorState,
    pub event_handler: EditorEventHandler,
}

impl App {
    pub fn run(&mut self, term: &mut Term) -> Result<()> {
        while !self.should_quit {
            self.draw(term)?;
            self.handle_events(term)?;
        }
        Term::stop()?;
        Ok(())
    }

    pub fn draw(&mut self, term: &mut Term) -> Result<()> {
        let _ = term.draw(|f| f.render_widget(self, f.area()));
        Ok(())
    }

    pub fn handle_events(&mut self, #[allow(unused)] term: &mut Term) -> Result<()> {
        let event = event::read()?;

        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                self.should_quit = true;
                return Ok(());
            }
        };

        self.context
            .event_handler
            .on_event(event, &mut self.context.state);

        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        EditorView::new(&mut self.context.state)
            .wrap(true)
            .theme(Theme::new().editor)
            .render(area, buf)
    }
}

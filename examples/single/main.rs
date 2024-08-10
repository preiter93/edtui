use edtui::{EditorEvent, EditorState, Lines};
use ratatui::crossterm::event::{self, Event, KeyCode};
use root::Root;
use std::error::Error;
use term::Term;
mod root;
mod term;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    App::run()
}

pub struct App {
    term: Term,
    context: AppContext,
    should_quit: bool,
}

pub struct AppContext {
    editor_state: EditorState,
    editor_event: EditorEvent,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            editor_state: EditorState::new(Lines::from("\"Single line editor\"")),
            editor_event: EditorEvent::default(),
        }
    }
}

impl App {
    pub fn new() -> Result<App> {
        Ok(App {
            term: Term::new()?,
            context: AppContext::new(),
            should_quit: false,
        })
    }

    fn draw(&mut self) -> Result<()> {
        let root = Root::new(&mut self.context);
        let _ = self.term.draw(|f| f.render_widget(root, f.size()));
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        let root = Root::new(&mut self.context);
        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Char('q') => self.should_quit = true,
                _ => root.handle_events(event),
            }
        }
        Ok(())
    }

    pub fn run() -> Result<()> {
        let mut app = Self::new()?;
        while !app.should_quit {
            app.draw()?;
            app.handle_events()?;
        }
        Term::stop()?;
        Ok(())
    }
}

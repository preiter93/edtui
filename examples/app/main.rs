use edtui::{EditorEventHandler, EditorState, Lines};
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
    state: EditorState,
    event_handler: EditorEventHandler,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(Lines::from(
                "EdTUI is a light-weight vim inspired TUI editor for the RataTUI ecosystem.

Navigate right (l), left (h), up (k) and down (j), using vim motions.

Traverse words forward (w) and backward (b).

Select text (v), including selection between \"delimiters\" (ciw).

Copy and paste text: 

Built-in search using the '/' command.

This editor is under active development.
Don't hesitate to open issues or submit pull requests to contribute!
",
            )),
            event_handler: EditorEventHandler::default(),
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
        match event::read()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => self.should_quit = true,
                _ => root.handle_key_events(event),
            },
            Event::Mouse(event) => root.handle_mouse_events(event),
            _ => {}
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

use edtui::{
    actions::SwitchMode,
    events::{KeyEvent, KeyEventHandler, KeyEventRegister},
    EditorEventHandler, EditorMode, EditorState, EditorView, Lines,
};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    prelude::*,
    widgets::Widget,
    DefaultTerminal,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame| frame.render_widget(&mut app, frame.area()))?;
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                break;
            }
            app.event_handler.on_key_event(key, &mut app.state);
        }
    }
    Ok(())
}

struct App {
    state: EditorState,
    event_handler: EditorEventHandler,
}

impl App {
    fn new() -> Self {
        let mut key_handler = KeyEventHandler::vim_mode();

        key_handler.insert(
            KeyEventRegister::n(vec![KeyEvent::Ctrl('x')]),
            SwitchMode(EditorMode::Insert),
        );

        key_handler.insert(
            KeyEventRegister::i(vec![KeyEvent::Ctrl('q')]),
            SwitchMode(EditorMode::Normal),
        );

        Self {
            state: EditorState::new(Lines::from(
                "Custom Keybindings Example

This example shows how to customize keybindings:
- Ctrl+x enters insert mode (instead of 'i')
- Ctrl+q exits insert mode (instead of Esc)

All other Vim keybindings remain active.

Try it out!
",
            )),
            event_handler: EditorEventHandler::new(key_handler),
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        EditorView::new(&mut self.state)
            .wrap(true)
            .render(area, buf)
    }
}

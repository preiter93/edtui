//! Example demonstrating the system editor feature.
//!
//! Press `Ctrl+e` in normal mode to open the content in your system's
//! default text editor ($VISUAL, $EDITOR, or platform default).
//!
//! Run with: cargo run --example system_editor --features system-editor

#![cfg(feature = "system-editor")]

use edtui::{EditorEventHandler, EditorState, EditorView, Lines};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
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
    let mut state = EditorState::new(Lines::from(
        "System Editor Example

Press Ctrl+e in normal mode to open this content
in a system editor (nvim, etc.).

Press Ctrl+c to quit.
",
    ));
    let mut event_handler = EditorEventHandler::default();

    loop {
        terminal.draw(|frame| {
            EditorView::new(&mut state)
                .wrap(true)
                .render(frame.area(), frame.buffer_mut());
        })?;

        let event = event::read()?;
        if let Event::Key(key) = &event {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                break;
            }
        }
        event_handler.on_event(event, &mut state, terminal);
    }
    Ok(())
}

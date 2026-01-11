//! Example demonstrating the system editor feature.
//!
//! Press `Ctrl+e` in normal mode to open the content in your system's
//! default text editor ($VISUAL, $EDITOR, or platform default).
//!
//! Run with: cargo run --example system_editor --features system-editor

#![cfg(feature = "system-editor")]

use edtui::{system_editor, EditorEventHandler, EditorState, EditorView, Lines};
use ratatui::{
    crossterm::{
        event::{self, EnableBracketedPaste, EnableMouseCapture, Event, KeyCode, KeyModifiers},
        execute,
    },
    widgets::Widget,
    DefaultTerminal,
};
use std::{error::Error, io::stdout};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    // Enable mouse capture and bracketed paste
    execute!(stdout(), EnableMouseCapture, EnableBracketedPaste)?;

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
        event_handler.on_event(event, &mut state);

        // Check if system editor was requested and open it
        if system_editor::is_pending(&state) {
            system_editor::open(&mut state, terminal)?;

            // Restore terminal modes after returning from system editor.
            // The system editor only restores raw mode and alternate screen;
            // other modes must be re-enabled manually.
            execute!(stdout(), EnableMouseCapture, EnableBracketedPaste)?;
        }
    }
    Ok(())
}

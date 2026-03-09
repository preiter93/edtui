//! Example demonstrating custom highlights with overlapping ranges and different colors.
//!
//! This example shows how to use the `Highlight` API to mark ranges of text
//! with custom styles. It demonstrates:
//! - Multiple highlights with different colors
//! - Overlapping highlights (later highlights take precedence)
//! - Multi-line highlights
//!
//! Run with: cargo run --example highlights

use edtui::{EditorEventHandler, EditorState, EditorTheme, EditorView, Highlight, Lines};
use jagged::Index2;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    prelude::*,
    widgets::{Block, Widget},
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

        let event = event::read()?;
        if let Event::Key(key) = &event {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                break;
            }
        }
        app.event_handler.on_event(event, &mut app.state);
    }
    Ok(())
}

struct App {
    state: EditorState,
    event_handler: EditorEventHandler,
}

impl App {
    fn new() -> Self {
        let content = Lines::from(
            "Welcome to the Highlights Demo!

This example demonstrates overlapping highlights.
The word 'overlapping' has multiple styles applied.

Here are some highlighted sections:
- Error: This text is marked as an error.
- Warning: This is a warning message.
- Info: Some informational text here.

Try editing the text - highlights stay in place!
Note: Highlights are position-based, not content-based.",
        );

        let mut state = EditorState::new(content);

        // Add highlights with different colors
        // Yellow background: "overlapping highlights" on line 2
        state.add_highlight(Highlight::new(
            Index2::new(2, 26),
            Index2::new(2, 48),
            Style::default().bg(Color::Yellow).fg(Color::Black),
        ));

        // Blue background overlapping part of the yellow: "highlights" on line 2
        // This demonstrates that later highlights take precedence
        state.add_highlight(Highlight::new(
            Index2::new(2, 38),
            Index2::new(2, 48),
            Style::default().bg(Color::Blue).fg(Color::White),
        ));

        // Red background: "Error" on line 6
        state.add_highlight(Highlight::new(
            Index2::new(6, 2),
            Index2::new(6, 6),
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

        // Yellow/Orange background: "Warning" on line 7
        state.add_highlight(Highlight::new(
            Index2::new(7, 2),
            Index2::new(7, 8),
            Style::default()
                .bg(Color::Rgb(255, 165, 0))
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));

        // Cyan background: "Info" on line 8
        state.add_highlight(Highlight::new(
            Index2::new(8, 2),
            Index2::new(8, 5),
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));

        // Multi-line highlight with green underline: spans lines 10-11
        state.add_highlight(Highlight::new(
            Index2::new(10, 0),
            Index2::new(11, 50),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::UNDERLINED),
        ));

        // Overlapping multi-line: magenta on part of line 10
        state.add_highlight(Highlight::new(
            Index2::new(10, 20),
            Index2::new(10, 35),
            Style::default().bg(Color::Magenta).fg(Color::White),
        ));

        Self {
            state,
            event_handler: EditorEventHandler::default(),
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Highlights Demo (Ctrl+C to exit) ");

        EditorView::new(&mut self.state)
            .theme(EditorTheme::default().block(block))
            .wrap(true)
            .render(area, buf);
    }
}

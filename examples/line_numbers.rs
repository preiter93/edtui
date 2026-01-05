use edtui::{EditorEventHandler, EditorState, EditorTheme, EditorView, LineNumbers, Lines};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Layout},
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
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                break;
            }
            if key.code == KeyCode::Tab {
                app.toggle_focus();
            } else {
                let state = match app.focus {
                    Focus::Left => &mut app.state_absolute,
                    Focus::Right => &mut app.state_relative,
                };
                app.event_handler.on_key_event(key, state);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Left,
    Right,
}

struct App {
    state_absolute: EditorState,
    state_relative: EditorState,
    event_handler: EditorEventHandler,
    focus: Focus,
}

impl App {
    fn new() -> Self {
        let absolute_content = Lines::from(
            "Absolute Line Numbers

Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
Line 9
Line 10
Line 11",
        );

        let relative_content = Lines::from(
            "Relative Line Numbers

Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
Line 9
Line 10
Line 11",
        );

        Self {
            state_absolute: EditorState::new(absolute_content),
            state_relative: EditorState::new(relative_content),
            event_handler: EditorEventHandler::default(),
            focus: Focus::Left,
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Left => Focus::Right,
            Focus::Right => Focus::Left,
        };
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(area);

        let left_block = Block::bordered().title(" Absolute (Tab to switch) ");
        let right_block = Block::bordered().title(" Relative (Tab to switch) ");

        // Left: Absolute line numbers
        EditorView::new(&mut self.state_absolute)
            .theme(EditorTheme::default().block(left_block))
            .line_numbers(LineNumbers::Absolute)
            .wrap(true)
            .render(left, buf);

        // Right: Relative line numbers
        EditorView::new(&mut self.state_relative)
            .theme(EditorTheme::default().block(right_block))
            .line_numbers(LineNumbers::Relative)
            .wrap(true)
            .render(right, buf);
    }
}

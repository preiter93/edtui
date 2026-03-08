//! Single-line Emacs mode example.
//!
//! Run with: `cargo run --example single_line_emacs`

use edtui::{EditorEventHandler, EditorMode, EditorState, EditorTheme, EditorView, Lines};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
    DefaultTerminal,
};

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> std::io::Result<()> {
    let mut state = EditorState::new(Lines::from("Hello World"));
    state.mode = EditorMode::Insert;
    state.set_single_line(true);

    let mut event_handler = EditorEventHandler::emacs_mode();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let [help_area, _, input_area, _] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .areas(area);

            // Help text
            let help = Paragraph::new(Line::from("Ctrl+Q: Quit | Emacs keybindings"));
            frame.render_widget(help, help_area);

            // Editor
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Single-line Input ");

            let theme = EditorTheme::default()
                .block(block)
                .base(Style::default().fg(Color::White))
                .cursor_style(Style::default().bg(Color::White).fg(Color::Black))
                .hide_status_line();

            EditorView::new(&mut state)
                .theme(theme)
                .single_line(true)
                .render(input_area, frame.buffer_mut());

            // Show cursor
            if let Some(pos) = state.cursor_screen_position() {
                frame.set_cursor_position(pos);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Quit on Ctrl+Q
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }

            event_handler.on_key_event(key, &mut state);
        }
    }

    Ok(())
}

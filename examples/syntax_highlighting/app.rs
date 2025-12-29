use edtui::{EditorEventHandler, EditorState, Lines, SyntaxHighlighter};
use edtui::{EditorView, Index2};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use std::error::Error;
use syntect::highlighting::ThemeSet;

use crate::term::Term;
use crate::theme::Theme;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Default)]
pub struct App {
    pub context: AppContext,
    pub should_quit: bool,
    pub theme_set: ThemeSet,
}

#[derive(Default)]
pub struct AppContext {
    pub state: EditorState,
    pub list_state: ListState,
    pub event_handler: EditorEventHandler,
}

impl AppContext {
    pub fn new() -> Self {
        let mut state = EditorState::new(Lines::default());
        state.cursor = Index2::new(1, 0); // get cursor out of the way for demo
        let mut list_state = ListState::default();
        list_state.select_next();
        Self {
            state,
            event_handler: EditorEventHandler::default(),
            list_state,
        }
    }
}

impl App {
    pub fn run(&mut self, term: &mut Term) -> Result<()> {
        while !self.should_quit {
            self.draw(term)?;
            self.handle_events()?;
        }
        Term::stop()?;
        Ok(())
    }

    pub fn draw(&mut self, term: &mut Term) -> Result<()> {
        let _ = term.draw(|f| f.render_widget(self, f.area()));
        Ok(())
    }

    pub fn handle_events(&mut self) -> Result<()> {
        let event = event::read()?;

        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                self.should_quit = true;
                return Ok(());
            }
            if key.code == KeyCode::Down {
                self.context.list_state.select_next();
                return Ok(());
            }
            if key.code == KeyCode::Up {
                self.context.list_state.select_previous();
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
        let [left, right] =
            Layout::horizontal([Constraint::Length(40), Constraint::Min(0)]).areas(area);

        let mut list_items: Vec<ListItem> = Vec::new();

        let mut theme_names: Vec<String> = Vec::new();
        for theme_name in self.theme_set.themes.keys() {
            let line = ListItem::new(Text::raw(theme_name));
            list_items.push(line);
            theme_names.push(theme_name.to_string());
        }
        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>");
        StatefulWidget::render(list, left, buf, &mut self.context.list_state);

        let selected_theme = self
            .context
            .list_state
            .selected()
            .map(|selected| theme_names.get(selected).unwrap().to_string())
            .unwrap_or("monokai".to_string());

        self.context.state.lines = get_lines(&selected_theme);
        EditorView::new(&mut self.context.state)
            .wrap(true)
            .theme(Theme::new().editor)
            .syntax_highlighter(Some(SyntaxHighlighter::new(&selected_theme, "rs")))
            .render(right, buf)
    }
}

fn get_lines(theme: &str) -> Lines {
    let data: &str = &format!(
        "fn main() {{
    let state = EditorState::default();
    let theme = \"${theme}\";
    let highlighter = SyntaxHighlighter::new(theme, \"rs\");
    EditorView::new(&mut state)
        .wrap(true)
        .theme(Theme::new().editor)
        .syntax_highlighter(Some(highlighter))
        .render(area, buf);
}}"
    );
    Lines::from(data)
}

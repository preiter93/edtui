use app::{App, AppContext};
use edtui::{EditorEventHandler, EditorState, Lines};
use std::error::Error;
use term::Term;
mod app;
mod term;
mod theme;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut term = Term::new()?;
    let mut app = App {
        context: AppContext::new(),
        should_quit: false,
    };
    app.run(&mut term)
}

impl AppContext {
    pub fn new() -> Self {
        let mut state = EditorState::new(Lines::from(
            "EdTUI with Emacs keybindings.

Navigation:
- Ctrl+f/b: forward/backward
- Ctrl+n/p: next/previous line
- Ctrl+a/e: start/end of line
- Ctrl+v: page down, Alt+v: page up
- Alt+f/b: forward/backward word
- Alt+</>: beginning/end of buffer

Editing:
- Ctrl+d: delete forward, Ctrl+h: delete backward
- Alt+d: delete word forward
- Alt+Backspace: delete word backward
- Ctrl+k: delete to end of line
- Alt+u: delete to start of line
- Ctrl+o: open line (insert newline, stay)
- Ctrl+j/Enter: newline

Search:
- Ctrl+s: start search / go to next match
- Ctrl+r: go to previous match
- Enter: Select match
- Ctrl+g: cancel search
Undo: Ctrl+u
Redo: Ctrl+r
Paste: Ctrl+y
",
        ));
        state.mode = edtui::EditorMode::Insert;

        Self {
            state,
            event_handler: EditorEventHandler::emacs_mode(),
        }
    }
}

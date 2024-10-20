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
        Self {
            state: EditorState::new(Lines::from(
                "EdTUI is a light-weight vim inspired TUI editor for the RataTUI ecosystem.

Navigate right (l), left (h), up (k) and down (j), using vim motions.

Traverse words forward (w) and backward (b).

Select text (v), including selection between \"quotes\" (viw/vi\").

Copy and paste text:

Built-in search using the '/' command.

Supports syntax highlighting:
```
fn main() {
    let state = EditorState::default();
    let theme = \"dracula\";
    let highlighter = SyntaxHighlighter::new(theme, \"rs\");
    EditorView::new(&mut state)
        .wrap(true)
        .theme(Theme::new().editor)
        .syntax_highlighter(Some(highlighter))
        .render(area, buf);
}
```
This editor is under active development.
Don't hesitate to open issues or submit pull requests to contribute! 🙂
",
            )),
            event_handler: EditorEventHandler::default(),
        }
    }
}

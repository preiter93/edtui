use app::{App, AppContext};
use std::error::Error;
use syntect::{dumps::from_binary, highlighting::ThemeSet};
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
        theme_set: load_binary(),
    };
    app.run(&mut term)
}

pub fn load_binary() -> ThemeSet {
    from_binary(include_bytes!("../../assets/default.themedump"))
}

use app::{App, AppContext};
use std::error::Error;
use syntect::highlighting::ThemeSet;
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
        theme_set: load_from_folder(),
    };
    app.run(&mut term)
}

pub fn load_from_folder() -> ThemeSet {
    let theme_set =
        ThemeSet::load_from_folder("/Users/philippreiter/Rust/edtui/assets/sublime").unwrap();
    return theme_set;
}

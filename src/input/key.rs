use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Key {
    Char(char),
    Down,
    Up,
    Right,
    Left,
    Enter,
    Esc,
    Backspace,
    None,
}

impl From<KeyEvent> for Key {
    fn from(key: KeyEvent) -> Self {
        match key.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Down => Key::Down,
            KeyCode::Up => Key::Up,
            KeyCode::Right => Key::Right,
            KeyCode::Left => Key::Left,
            KeyCode::Esc => Key::Esc,
            KeyCode::Backspace => Key::Backspace,
            _ => Key::None,
        }
    }
}

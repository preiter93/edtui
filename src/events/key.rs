use ratatui::crossterm::event::{KeyCode, KeyEvent as CTKeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum KeyEvent {
    Char(char),
    Down,
    Up,
    Right,
    Left,
    Enter,
    Esc,
    Backspace,
    Ctrl(char),
    None,
}

impl From<CTKeyEvent> for KeyEvent {
    fn from(key: CTKeyEvent) -> Self {
        if key.modifiers == KeyModifiers::CONTROL {
            return match key.code {
                KeyCode::Char(c) => KeyEvent::Ctrl(c),
                _ => KeyEvent::None,
            };
        }

        match key.code {
            KeyCode::Char(c) => KeyEvent::Char(c),
            KeyCode::Enter => KeyEvent::Enter,
            KeyCode::Down => KeyEvent::Down,
            KeyCode::Up => KeyEvent::Up,
            KeyCode::Right => KeyEvent::Right,
            KeyCode::Left => KeyEvent::Left,
            KeyCode::Esc => KeyEvent::Esc,
            KeyCode::Backspace => KeyEvent::Backspace,
            _ => KeyEvent::None,
        }
    }
}

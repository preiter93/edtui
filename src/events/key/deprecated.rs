#![allow(deprecated)]
use crossterm::event::{KeyCode, KeyEvent as CTKeyEvent, KeyModifiers};

use crate::events::KeyInput;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[deprecated(since = "0.11.0", note = "Please use `KeyInput` instead")]
pub enum KeyEvent {
    Char(char),
    Down,
    Up,
    Right,
    Left,
    Enter,
    Esc,
    Backspace,
    Delete,
    Tab,
    Ctrl(char),
    Alt(char),
    Home,
    End,
    None,
}

impl From<CTKeyEvent> for KeyEvent {
    fn from(key: CTKeyEvent) -> Self {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char(c) => KeyEvent::Ctrl(c),
                _ => KeyEvent::None,
            };
        }

        if key.modifiers.contains(KeyModifiers::ALT) {
            return match key.code {
                KeyCode::Char(c) => KeyEvent::Alt(c),
                KeyCode::Backspace => KeyEvent::Alt('\x08'),
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
            KeyCode::Delete => KeyEvent::Delete,
            KeyCode::Tab => KeyEvent::Tab,
            KeyCode::Home => KeyEvent::Home,
            KeyCode::End => KeyEvent::End,
            _ => KeyEvent::None,
        }
    }
}

impl From<KeyEvent> for KeyInput {
    fn from(ev: KeyEvent) -> Self {
        match ev {
            KeyEvent::Char(c) => KeyInput::new(KeyCode::Char(c)),
            KeyEvent::Ctrl(c) => KeyInput::ctrl(KeyCode::Char(c)),
            KeyEvent::Alt(c) => KeyInput::alt(KeyCode::Char(c)),

            KeyEvent::Up => KeyInput::new(KeyCode::Up),
            KeyEvent::Down => KeyInput::new(KeyCode::Down),
            KeyEvent::Left => KeyInput::new(KeyCode::Left),
            KeyEvent::Right => KeyInput::new(KeyCode::Right),

            KeyEvent::Enter => KeyInput::new(KeyCode::Enter),
            KeyEvent::Esc => KeyInput::new(KeyCode::Esc),
            KeyEvent::Backspace => KeyInput::new(KeyCode::Backspace),
            KeyEvent::Delete => KeyInput::new(KeyCode::Delete),
            KeyEvent::Tab => KeyInput::new(KeyCode::Tab),
            KeyEvent::Home => KeyInput::new(KeyCode::Home),
            KeyEvent::End => KeyInput::new(KeyCode::End),

            KeyEvent::None => KeyInput::new(KeyCode::Char('\0')),
        }
    }
}

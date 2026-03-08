/// A representation of a keyboard input.
///
/// ## Examples
///
/// ### Single key
///
/// ```
/// use edtui::events::KeyInput;
///
/// let input = KeyInput::new('s');
/// ```
///
/// ### Key with one modifier
///
/// ```
/// use edtui::events::KeyInput;
/// use crossterm::event::KeyCode;
///
/// let input = KeyInput::ctrl(KeyCode::Esc);
/// ```
///
/// ### Key with multiple modifiers
///
/// ```
/// use edtui::events::KeyInput;
/// use crossterm::event::KeyModifiers;
///
/// let input = KeyInput::with_modifiers('k', KeyModifiers::CONTROL | KeyModifiers::SHIFT);
/// ```
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct KeyInput {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

impl KeyInput {
    /// Create a key input without any modifiers.
    pub fn new<K: Into<KeyCode>>(key: K) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::NONE,
        }
    }

    /// Create a key input with the ctrl modifier.
    pub fn ctrl<K: Into<KeyCode>>(key: K) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::CONTROL,
        }
    }

    /// Create a key input with the alt modifier.
    pub fn alt<K: Into<KeyCode>>(key: K) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::ALT,
        }
    }

    /// Create a key input with the shift modifier.
    pub fn shift<K: Into<KeyCode>>(key: K) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::SHIFT,
        }
    }

    /// Create a key input with a set of modifiers.
    /// Supports shift, ctrl and alt.
    pub fn with_modifiers<K: Into<KeyCode>, M: Into<Modifiers>>(key: K, modifiers: M) -> Self {
        Self {
            key: key.into(),
            modifiers: modifiers.into(),
        }
    }

    /// Normalize AltGr key events for international keyboard support.
    ///
    /// Strips Ctrl+Alt modifiers from non-alphabetic characters, allowing
    /// symbols like `[`, `]`, `{`, `}`, `\`, `@` to be typed on keyboards
    /// where these require AltGr (e.g., German, Swiss German).
    #[must_use]
    pub fn normalize_altgr(self) -> Self {
        if let KeyCode::Char(c) = self.key {
            // AltGr is typically reported as Ctrl+Alt on Windows/some terminals
            // Some terminals may report it as just Alt
            let has_altgr_modifiers = self.modifiers.is_ctrl_alt() || self.modifiers.is_alt_only();

            if has_altgr_modifiers && !c.is_ascii_alphabetic() {
                return Self {
                    key: self.key,
                    modifiers: self.modifiers.without_ctrl_alt(),
                };
            }
        }

        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum KeyCode {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Enter,
    Esc,
    Backspace,
    Delete,
    Tab,
    Home,
    End,
    PageUp,
    PageDown,
}

impl From<char> for KeyCode {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}

impl From<crossterm::event::KeyCode> for KeyCode {
    fn from(ev: crossterm::event::KeyCode) -> Self {
        use crossterm::event::KeyCode as CTKeyCode;
        match ev {
            CTKeyCode::Char(c) => KeyCode::Char(c),
            CTKeyCode::Enter => KeyCode::Enter,
            CTKeyCode::Esc => KeyCode::Esc,
            CTKeyCode::Backspace => KeyCode::Backspace,
            CTKeyCode::Delete => KeyCode::Delete,
            CTKeyCode::Tab => KeyCode::Tab,
            CTKeyCode::Left => KeyCode::Left,
            CTKeyCode::Right => KeyCode::Right,
            CTKeyCode::Up => KeyCode::Up,
            CTKeyCode::Down => KeyCode::Down,
            CTKeyCode::Home => KeyCode::Home,
            CTKeyCode::End => KeyCode::End,
            CTKeyCode::PageUp => KeyCode::PageUp,
            CTKeyCode::PageDown => KeyCode::PageDown,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Modifiers {
    ctrl: bool,
    alt: bool,
    shift: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
    };

    pub const CONTROL: Self = Self {
        ctrl: true,
        alt: false,
        shift: false,
    };

    pub const ALT: Self = Self {
        ctrl: false,
        alt: true,
        shift: false,
    };

    pub const SHIFT: Self = Self {
        ctrl: false,
        alt: false,
        shift: true,
    };

    /// Returns true if Ctrl+Alt are both pressed (typical AltGr representation).
    #[must_use]
    pub fn is_ctrl_alt(&self) -> bool {
        self.ctrl && self.alt
    }

    /// Returns true if Alt is pressed (without Ctrl).
    #[must_use]
    pub fn is_alt_only(&self) -> bool {
        self.alt && !self.ctrl
    }

    /// Returns modifiers with Ctrl and Alt stripped, preserving only Shift.
    #[must_use]
    pub fn without_ctrl_alt(&self) -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: self.shift,
        }
    }
}

impl From<crossterm::event::KeyModifiers> for Modifiers {
    fn from(ev: crossterm::event::KeyModifiers) -> Self {
        Modifiers {
            ctrl: ev.contains(crossterm::event::KeyModifiers::CONTROL),
            alt: ev.contains(crossterm::event::KeyModifiers::ALT),
            shift: ev.contains(crossterm::event::KeyModifiers::SHIFT),
        }
    }
}

impl From<crossterm::event::KeyEvent> for KeyInput {
    fn from(ev: crossterm::event::KeyEvent) -> Self {
        Self {
            key: ev.code.into(),
            modifiers: ev.modifiers.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_altgr_symbol() {
        // Ctrl+Alt+[ should normalize to just [
        let mods = Modifiers {
            ctrl: true,
            alt: true,
            shift: false,
        };
        let input = KeyInput::with_modifiers('[', mods);
        let normalized = input.normalize_altgr();
        assert_eq!(normalized.key, KeyCode::Char('['));
        assert_eq!(normalized.modifiers, Modifiers::NONE);
    }

    #[test]
    fn test_normalize_altgr_preserves_shift() {
        // Ctrl+Alt+Shift+{ should normalize to Shift+{
        let mods = Modifiers {
            ctrl: true,
            alt: true,
            shift: true,
        };
        let input = KeyInput::with_modifiers('{', mods);
        let normalized = input.normalize_altgr();
        assert_eq!(normalized.key, KeyCode::Char('{'));
        assert_eq!(normalized.modifiers, Modifiers::SHIFT);
    }

    #[test]
    fn test_normalize_altgr_keeps_letter_keybindings() {
        // Alt+f should NOT be normalized (it's a keybinding)
        let input = KeyInput::alt('f');
        let normalized = input.normalize_altgr();
        assert_eq!(normalized.modifiers, Modifiers::ALT);
    }

    #[test]
    fn test_normalize_altgr_keeps_ctrl_alt_letter() {
        // Ctrl+Alt+b should NOT be normalized (it's a keybinding)
        let mods = Modifiers {
            ctrl: true,
            alt: true,
            shift: false,
        };
        let input = KeyInput::with_modifiers('b', mods);
        let normalized = input.normalize_altgr();
        assert!(normalized.modifiers.ctrl);
        assert!(normalized.modifiers.alt);
    }
}

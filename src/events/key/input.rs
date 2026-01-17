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

pub(crate) mod deprecated;
pub(crate) mod input;

use crate::actions::cpaste::PasteOverSelection;
use crate::actions::delete::{DeleteCharForward, DeleteToEndOfLine, DeleteToFirstCharOfLine};
use crate::actions::motion::{MoveHalfPageDown, MoveToFirstRow, MoveToLastRow};
use crate::actions::search::StartSearch;
#[cfg(feature = "system-editor")]
use crate::actions::OpenSystemEditor;
use crate::actions::{
    Action, AppendCharToSearch, AppendNewline, Chainable, ChangeInnerBetween, ChangeInnerWord,
    ChangeSelection, CopyLine, CopySelection, DeleteChar, DeleteLine, DeleteSelection, Execute,
    FindFirst, FindNext, FindPrevious, InsertChar, InsertNewline, JoinLineWithLineBelow, LineBreak,
    MoveBackward, MoveDown, MoveForward, MoveHalfPageUp, MoveToEndOfLine, MoveToFirst,
    MoveToMatchinBracket, MoveToStartOfLine, MoveUp, MoveWordBackward, MoveWordForward,
    MoveWordForwardToEndOfWord, Paste, Redo, RemoveChar, RemoveCharFromSearch, SelectCurrentSearch,
    SelectInnerBetween, SelectInnerWord, SelectLine, StopSearch, SwitchMode, Undo,
};
use crate::events::KeyInput;
use crate::{EditorMode, EditorState};
use crossterm::event::KeyCode;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct KeyEventHandler {
    lookup: Vec<KeyInput>,
    register: HashMap<KeyEventRegister, Action>,
    capture_on_insert: bool,
}

impl Default for KeyEventHandler {
    fn default() -> Self {
        Self::vim_mode()
    }
}

impl KeyEventHandler {
    /// Creates a new `KeyEventHandler`.
    #[must_use]
    pub fn new(register: HashMap<KeyEventRegister, Action>, capture_on_insert: bool) -> Self {
        Self {
            lookup: Vec::new(),
            register,
            capture_on_insert,
        }
    }

    /// Creates a new `KeyEventHandler` with vim keybindings.
    #[must_use]
    pub fn vim_mode() -> Self {
        let register: HashMap<KeyEventRegister, Action> = vim_keybindings();
        Self {
            lookup: Vec::new(),
            register,
            capture_on_insert: false,
        }
    }

    // Creates a new `KeyEventHandler` with emacs keybindings.
    #[must_use]
    pub fn emacs_mode() -> Self {
        let register: HashMap<KeyEventRegister, Action> = emacs_keybindings();
        Self {
            lookup: Vec::new(),
            register,
            capture_on_insert: true,
        }
    }

    /// Insert a new callback to the registry
    pub fn insert<T>(&mut self, key: KeyEventRegister, action: T)
    where
        T: Into<Action>,
    {
        self.register.insert(key, action.into());
    }

    /// Extents the register with the contents of an iterator
    pub fn extend<T, U>(&mut self, iter: T)
    where
        U: Into<Action>,
        T: IntoIterator<Item = (KeyEventRegister, U)>,
    {
        self.register
            .extend(iter.into_iter().map(|(k, v)| (k, v.into())));
    }

    /// Remove a callback from the registry
    pub fn remove(&mut self, key: &KeyEventRegister) {
        self.register.remove(key);
    }

    /// Returns an action for a specific register key, if present.
    /// Returns an action only if there is an exact match. If there
    /// are multiple matches or an inexact match, the specified key
    /// is appended to the lookup vector.
    /// If there is an exact match or if none of the keys in the registry
    /// starts with the current sequence, the lookup sequence is reset.
    #[must_use]
    fn get(&mut self, c: KeyInput, mode: EditorMode) -> Option<Action> {
        self.lookup.push(c);
        let key = KeyEventRegister::new(self.lookup.clone(), mode);

        match self
            .register
            .keys()
            .filter(|k| k.mode == key.mode && k.keys.starts_with(&key.keys))
            .count()
        {
            0 => {
                self.lookup.clear();
                None
            }
            1 => self.register.get(&key).map(|action| {
                self.lookup.clear();
                action.clone()
            }),
            _ => None,
        }
    }
}

#[allow(clippy::too_many_lines)]
fn vim_keybindings() -> HashMap<KeyEventRegister, Action> {
    #[allow(unused_mut)]
    let mut map = HashMap::from([
        // Go into normal mode
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Esc)]),
            SwitchMode(EditorMode::Normal).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Esc)]),
            SwitchMode(EditorMode::Normal).into(),
        ),
        // Go into insert mode
        (
            KeyEventRegister::n(vec![KeyInput::new('i')]),
            SwitchMode(EditorMode::Insert).into(),
        ),
        // Go into visual mode
        (
            KeyEventRegister::n(vec![KeyInput::new('v')]),
            SwitchMode(EditorMode::Visual).into(),
        ),
        // Goes into search mode and starts of a new search.
        (
            KeyEventRegister::n(vec![KeyInput::new('/')]),
            StartSearch.chain(SwitchMode(EditorMode::Search)).into(),
        ),
        // Trigger initial search
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Enter)]),
            FindFirst.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        // Find next
        (
            KeyEventRegister::n(vec![KeyInput::new('n')]),
            FindNext.into(),
        ),
        // Find previous
        (
            KeyEventRegister::n(vec![KeyInput::shift('N')]),
            FindPrevious.into(),
        ),
        // Clear search
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Esc)]),
            StopSearch.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        // Delete last character from search
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Backspace)]),
            RemoveCharFromSearch.into(),
        ),
        // Go into insert mode and move one char forward
        (
            KeyEventRegister::n(vec![KeyInput::new('a')]),
            SwitchMode(EditorMode::Insert).chain(MoveForward(1)).into(),
        ),
        // Move cursor forward
        (
            KeyEventRegister::n(vec![KeyInput::new('l')]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('l')]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        // Move cursor backward
        (
            KeyEventRegister::n(vec![KeyInput::new('h')]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('h')]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        // Move cursor up
        (
            KeyEventRegister::n(vec![KeyInput::new('k')]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('k')]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        // Move cursor down
        (
            KeyEventRegister::n(vec![KeyInput::new('j')]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('j')]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        // Move one word forward/backward
        (
            KeyEventRegister::n(vec![KeyInput::new('w')]),
            MoveWordForward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('w')]),
            MoveWordForward(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('e')]),
            MoveWordForwardToEndOfWord(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('e')]),
            MoveWordForwardToEndOfWord(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('b')]),
            MoveWordBackward(1).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('b')]),
            MoveWordBackward(1).into(),
        ),
        // Move cursor to start/first/last position
        (
            KeyEventRegister::n(vec![KeyInput::new('0')]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('_')]),
            MoveToFirst().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('$')]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('0')]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('_')]),
            MoveToFirst().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('$')]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::ctrl('d')]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl('d')]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::ctrl('u')]),
            MoveHalfPageUp().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::ctrl('u')]),
            MoveHalfPageUp().into(),
        ),
        // `Home` and `End` go to first/last position in a line
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Home)]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Home)]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::Home)]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::End)]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::End)]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new(KeyCode::End)]),
            MoveToEndOfLine().into(),
        ),
        // `Ctrl+u` deltes from cursor to first non-whitespace character in insert mode
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('u')]),
            DeleteToFirstCharOfLine.into(),
        ),
        // Move cursor to start/first/last position and enter insert mode
        (
            KeyEventRegister::n(vec![KeyInput::shift('I')]),
            SwitchMode(EditorMode::Insert).chain(MoveToFirst()).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::shift('A')]),
            SwitchMode(EditorMode::Insert)
                .chain(MoveToEndOfLine())
                .chain(MoveForward(1))
                .into(),
        ),
        // Move cursor to start/last row in the buffer
        (
            KeyEventRegister::n(vec![KeyInput::new('g'), KeyInput::new('g')]),
            MoveToFirstRow().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('g'), KeyInput::new('g')]),
            MoveToFirstRow().into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::shift('G')]),
            MoveToLastRow().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::shift('G')]),
            MoveToLastRow().into(),
        ),
        // Move cursor to the next opening/closing bracket.
        (
            KeyEventRegister::n(vec![KeyInput::new('%')]),
            MoveToMatchinBracket().into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('%')]),
            MoveToMatchinBracket().into(),
        ),
        // Append/insert new line and switch into insert mode
        (
            KeyEventRegister::n(vec![KeyInput::new('o')]),
            SwitchMode(EditorMode::Insert)
                .chain(AppendNewline(1))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::shift('O')]),
            SwitchMode(EditorMode::Insert)
                .chain(InsertNewline(1))
                .into(),
        ),
        // Insert a line break
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Enter)]),
            LineBreak(1).into(),
        ),
        // Remove the current character
        (
            KeyEventRegister::n(vec![KeyInput::new('x')]),
            RemoveChar(1).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new(KeyCode::Delete)]),
            RemoveChar(1).into(),
        ),
        // Delete the previous character
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Backspace)]),
            DeleteChar(1).into(),
        ),
        // Delete the next character
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Delete)]),
            DeleteCharForward(1).into(),
        ),
        // Delete the current line
        (
            KeyEventRegister::n(vec![KeyInput::new('d'), KeyInput::new('d')]),
            DeleteLine(1).into(),
        ),
        // Delete from the cursor to the end of the line
        (
            KeyEventRegister::n(vec![KeyInput::shift('D')]),
            DeleteToEndOfLine.into(),
        ),
        // Delete the current selection
        (
            KeyEventRegister::v(vec![KeyInput::new('d')]),
            DeleteSelection.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        // Join the current line with the line below
        (
            KeyEventRegister::n(vec![KeyInput::shift('J')]),
            JoinLineWithLineBelow.into(),
        ),
        // Select inner word between delimiters
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('w')]),
            SelectInnerWord.into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('"')]),
            SelectInnerBetween::new('"', '"').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('\'')]),
            SelectInnerBetween::new('\'', '\'').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('(')]),
            SelectInnerBetween::new('(', ')').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new(')')]),
            SelectInnerBetween::new('(', ')').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('{')]),
            SelectInnerBetween::new('{', '}').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('}')]),
            SelectInnerBetween::new('{', '}').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new('[')]),
            SelectInnerBetween::new('[', ']').into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('i'), KeyInput::new(']')]),
            SelectInnerBetween::new('[', ']').into(),
        ),
        // Change inner word between delimiters
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('w'),
            ]),
            SwitchMode(EditorMode::Insert).chain(ChangeInnerWord).into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('"'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('"', '"'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('\''),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('\'', '\''))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('('),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('(', ')'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new(')'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('(', ')'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('{'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('{', '}'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('}'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('{', '}'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new('['),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('[', ']'))
                .into(),
        ),
        (
            KeyEventRegister::n(vec![
                KeyInput::new('c'),
                KeyInput::new('i'),
                KeyInput::new(']'),
            ]),
            SwitchMode(EditorMode::Insert)
                .chain(ChangeInnerBetween::new('[', ']'))
                .into(),
        ),
        // Change selection
        (
            KeyEventRegister::v(vec![KeyInput::new('c')]),
            SwitchMode(EditorMode::Insert).chain(ChangeSelection).into(),
        ),
        (
            KeyEventRegister::v(vec![KeyInput::new('x')]),
            ChangeSelection.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        // Select  the line
        (
            KeyEventRegister::n(vec![KeyInput::shift('V')]),
            SelectLine.into(),
        ),
        // Undo
        (KeyEventRegister::n(vec![KeyInput::new('u')]), Undo.into()),
        // Redo
        (KeyEventRegister::n(vec![KeyInput::ctrl('r')]), Redo.into()),
        // Copy
        (
            KeyEventRegister::v(vec![KeyInput::new('y')]),
            CopySelection.chain(SwitchMode(EditorMode::Normal)).into(),
        ),
        (
            KeyEventRegister::n(vec![KeyInput::new('y'), KeyInput::new('y')]),
            CopyLine.into(),
        ),
        // Paste
        (KeyEventRegister::n(vec![KeyInput::new('p')]), Paste.into()),
        (
            KeyEventRegister::v(vec![KeyInput::new('p')]),
            PasteOverSelection
                .chain(SwitchMode(EditorMode::Normal))
                .into(),
        ),
    ]);

    // Open system editor (Ctrl+e in normal mode)
    #[cfg(feature = "system-editor")]
    map.insert(
        KeyEventRegister::n(vec![KeyInput::ctrl('e')]),
        OpenSystemEditor.into(),
    );

    map
}

#[allow(clippy::too_many_lines)]
fn emacs_keybindings() -> HashMap<KeyEventRegister, Action> {
    HashMap::from([
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('s')]),
            StartSearch.chain(SwitchMode(EditorMode::Search)).into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::ctrl('s')]),
            FindNext.into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::ctrl('r')]),
            FindPrevious.into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Enter)]),
            SelectCurrentSearch
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::ctrl('g')]),
            StopSearch.chain(SwitchMode(EditorMode::Insert)).into(),
        ),
        (
            KeyEventRegister::s(vec![KeyInput::new(KeyCode::Backspace)]),
            RemoveCharFromSearch.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('f')]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Right)]),
            MoveForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('b')]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Left)]),
            MoveBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('p')]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Up)]),
            MoveUp(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('n')]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Down)]),
            MoveDown(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('f')]),
            MoveWordForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('b')]),
            MoveWordBackward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('v')]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('v')]),
            MoveHalfPageUp().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('<')]),
            MoveToFirstRow().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('>')]),
            MoveToLastRow().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('a')]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Home)]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::End)]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('e')]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('u')]),
            DeleteToFirstCharOfLine.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('k')]),
            DeleteToEndOfLine.into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('o')]),
            LineBreak(1)
                .chain(MoveUp(1))
                .chain(MoveToEndOfLine())
                .into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Enter)]),
            LineBreak(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('j')]),
            LineBreak(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Backspace)]),
            DeleteChar(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('h')]),
            DeleteChar(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::new(KeyCode::Backspace)]),
            DeleteCharForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::ctrl('d')]),
            DeleteCharForward(1).into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt('d')]),
            SwitchMode(EditorMode::Visual)
                .chain(MoveWordForwardToEndOfWord(1))
                .chain(DeleteSelection)
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (
            KeyEventRegister::i(vec![KeyInput::alt(KeyCode::Backspace)]),
            SwitchMode(EditorMode::Visual)
                .chain(MoveWordBackward(1))
                .chain(DeleteSelection)
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (KeyEventRegister::i(vec![KeyInput::ctrl('u')]), Undo.into()),
        (KeyEventRegister::i(vec![KeyInput::ctrl('r')]), Redo.into()),
        (KeyEventRegister::i(vec![KeyInput::ctrl('y')]), Paste.into()),
        #[cfg(feature = "system-editor")]
        (
            KeyEventRegister::i(vec![KeyInput::alt('e')]),
            OpenSystemEditor.into(),
        ),
    ])
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyInputSequence(Vec<KeyInput>);

impl KeyInputSequence {
    pub fn new(keys: Vec<KeyInput>) -> Self {
        KeyInputSequence(keys)
    }
}

impl From<Vec<KeyInput>> for KeyInputSequence {
    fn from(keys: Vec<KeyInput>) -> Self {
        KeyInputSequence(keys)
    }
}

#[allow(deprecated)]
impl From<Vec<deprecated::KeyEvent>> for KeyInputSequence {
    fn from(events: Vec<deprecated::KeyEvent>) -> Self {
        KeyInputSequence(events.into_iter().map(|event| event.into()).collect())
    }
}

impl From<KeyInputSequence> for Vec<KeyInput> {
    fn from(seq: KeyInputSequence) -> Self {
        seq.0
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyEventRegister {
    keys: Vec<KeyInput>,
    mode: EditorMode,
}

type RegisterCB = fn(&mut EditorState);

#[derive(Clone, Debug)]
struct RegisterVal(pub fn(&mut EditorState));

impl KeyEventRegister {
    pub fn new<T>(key: T, mode: EditorMode) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self {
            keys: key.into().into(),
            mode,
        }
    }

    pub fn n<T>(key: T) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self::new(key, EditorMode::Normal)
    }

    pub fn v<T>(key: T) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self::new(key, EditorMode::Visual)
    }

    pub fn i<T>(key: T) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self::new(key, EditorMode::Insert)
    }

    pub fn s<T>(key: T) -> Self
    where
        T: Into<KeyInputSequence>,
    {
        Self::new(key, EditorMode::Search)
    }
}

impl KeyEventHandler {
    pub(crate) fn on_event<T>(&mut self, key: T, state: &mut EditorState)
    where
        T: Into<KeyInput> + Copy + std::fmt::Debug,
    {
        let mode = state.mode;
        let key_input = key.into();

        // Always insert characters in insert mode
        if mode == EditorMode::Insert {
            if let input::KeyCode::Char(c) = key_input.key {
                if key_input.modifiers == input::Modifiers::NONE
                    || key_input.modifiers == input::Modifiers::SHIFT 
                {
                    if self.capture_on_insert {
                        state.capture();
                    }
                    InsertChar(c).execute(state);
                    return;
                }
            }

            if matches!(key_input.key, input::KeyCode::Tab)
                && key_input.modifiers == input::Modifiers::NONE
            {
                if self.capture_on_insert {
                    state.capture();
                }
                InsertChar('\t').execute(state);
                return;
            }
        }

        // Always add characters to search in search mode
        if mode == EditorMode::Search {
            if let input::KeyCode::Char(c) = key_input.key {
                if key_input.modifiers == input::Modifiers::NONE {
                    AppendCharToSearch(c).execute(state);
                    return;
                }
            }
        }

        // Else lookup an action from the register
        if let Some(mut action) = self.get(key_input, mode) {
            action.execute(state);
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::deprecated::KeyEvent;
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_key_event_register_with_key_event() {
        let register = KeyEventRegister::n(vec![KeyEvent::Ctrl('a'), KeyEvent::Char('b')]);
        assert_eq!(register.mode, EditorMode::Normal);
        assert_eq!(register.keys.len(), 2);

        assert_eq!(register.keys[0], KeyInput::ctrl('a'));
        assert_eq!(register.keys[1], KeyInput::new('b'));
    }

    #[test]
    fn test_key_event_register_with_key_input() {
        let register = KeyEventRegister::i(vec![KeyInput::ctrl('a'), KeyInput::new('b')]);
        assert_eq!(register.mode, EditorMode::Insert);
        assert_eq!(register.keys.len(), 2);

        assert_eq!(register.keys[0], KeyInput::ctrl('a'));
        assert_eq!(register.keys[1], KeyInput::new('b'));
    }

    #[test]
    fn test_key_event_register_with_crossterm() {
        use crossterm::event::{KeyCode as CTKeyCode, KeyEvent as CTKeyEvent, KeyModifiers};

        let ct_key_event = CTKeyEvent::new(CTKeyCode::Char('a'), KeyModifiers::CONTROL);
        let key_input: KeyInput = ct_key_event.into();

        let register = KeyEventRegister::v(vec![key_input, KeyInput::new(CTKeyCode::Enter)]);
        assert_eq!(register.mode, EditorMode::Visual);
        assert_eq!(register.keys.len(), 2);

        assert_eq!(register.keys[0], KeyInput::ctrl('a'));
        assert_eq!(register.keys[1], KeyInput::new(CTKeyCode::Enter));
    }

    #[test]
    fn test_insert_hello_world() {
        use crate::EditorState;

        let mut state = EditorState::default();
        state.mode = EditorMode::Insert;

        let mut handler = KeyEventHandler::default();

        let inputs = vec![
            KeyInput::shift('H'),
            KeyInput::new('e'),
            KeyInput::new('l'),
            KeyInput::new('l'),
            KeyInput::new('o'),
            KeyInput::new(' '),
            KeyInput::shift('W'),
            KeyInput::new('o'),
            KeyInput::new('r'),
            KeyInput::new('l'),
            KeyInput::new('d'),
            KeyInput::shift('!'),
        ];

        for input in inputs {
            handler.on_event(input, &mut state);
        }

        assert_eq!(state.lines.to_string(), String::from("Hello World!"));
    }
}

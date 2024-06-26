//! Handles key input events
pub mod key;
pub mod register;

use crate::actions::search::StartSearch;
use crate::actions::{
    Append, AppendCharToSearch, AppendNewline, Composed, CopySelection, DeleteChar, DeleteLine,
    DeleteSelection, Execute, FindNext, FindPrevious, InsertChar, InsertNewline, LineBreak,
    MoveBackward, MoveDown, MoveForward, MoveToEnd, MoveToFirst, MoveToStart, MoveUp,
    MoveWordBackward, MoveWordForward, Paste, Redo, RemoveChar, RemoveCharFromSearch,
    SelectBetween, SelectLine, StopSearch, SwitchMode, TriggerSearch, Undo,
};
use crate::{EditorMode, EditorState};

use self::key::Key;
use self::register::{Register, RegisterKey};

#[derive(Clone, Debug)]
pub struct EditorInput {
    register: Register,
}

impl Default for EditorInput {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        let mut r = Register::new();

        // Go into normal mode
        r.insert(
            RegisterKey::i(vec![Key::Esc]),
            SwitchMode(EditorMode::Normal),
        );
        r.insert(
            RegisterKey::v(vec![Key::Esc]),
            SwitchMode(EditorMode::Normal),
        );

        // Go into insert mode
        r.insert(
            RegisterKey::n(vec![Key::Char('i')]),
            SwitchMode(EditorMode::Insert),
        );

        // Go into visual mode
        r.insert(
            RegisterKey::n(vec![Key::Char('v')]),
            SwitchMode(EditorMode::Visual),
        );

        // Goes into search mode and starts of a new search.
        r.insert(RegisterKey::n(vec![Key::Char('/')]), StartSearch);
        // Trigger initial search
        r.insert(RegisterKey::s(vec![Key::Enter]), TriggerSearch);
        // Find next
        r.insert(RegisterKey::n(vec![Key::Char('n')]), FindNext);
        // Find previous
        r.insert(RegisterKey::n(vec![Key::Char('N')]), FindPrevious);
        // Clear search
        r.insert(RegisterKey::s(vec![Key::Esc]), StopSearch);
        // Delete last character from search
        r.insert(RegisterKey::s(vec![Key::Backspace]), RemoveCharFromSearch);

        // Go into insert mode and move one char forward
        r.insert(RegisterKey::n(vec![Key::Char('a')]), Append);

        // Move cursor right
        r.insert(RegisterKey::n(vec![Key::Char('l')]), MoveForward(1));
        r.insert(RegisterKey::v(vec![Key::Char('l')]), MoveForward(1));
        r.insert(RegisterKey::n(vec![Key::Right]), MoveForward(1));
        r.insert(RegisterKey::v(vec![Key::Right]), MoveForward(1));
        r.insert(RegisterKey::i(vec![Key::Right]), MoveForward(1));

        // Move cursor left
        r.insert(RegisterKey::n(vec![Key::Char('h')]), MoveBackward(1));
        r.insert(RegisterKey::v(vec![Key::Char('h')]), MoveBackward(1));
        r.insert(RegisterKey::n(vec![Key::Left]), MoveBackward(1));
        r.insert(RegisterKey::v(vec![Key::Left]), MoveBackward(1));
        r.insert(RegisterKey::i(vec![Key::Left]), MoveBackward(1));

        // Move cursor up
        r.insert(RegisterKey::n(vec![Key::Char('k')]), MoveUp(1));
        r.insert(RegisterKey::v(vec![Key::Char('k')]), MoveUp(1));
        r.insert(RegisterKey::n(vec![Key::Up]), MoveUp(1));
        r.insert(RegisterKey::v(vec![Key::Up]), MoveUp(1));
        r.insert(RegisterKey::i(vec![Key::Up]), MoveUp(1));

        // Move cursor down
        r.insert(RegisterKey::n(vec![Key::Char('j')]), MoveDown(1));
        r.insert(RegisterKey::v(vec![Key::Char('j')]), MoveDown(1));
        r.insert(RegisterKey::n(vec![Key::Down]), MoveDown(1));
        r.insert(RegisterKey::v(vec![Key::Down]), MoveDown(1));
        r.insert(RegisterKey::i(vec![Key::Down]), MoveDown(1));

        // Move one word forward/backward
        r.insert(RegisterKey::n(vec![Key::Char('w')]), MoveWordForward(1));
        r.insert(RegisterKey::n(vec![Key::Char('b')]), MoveWordBackward(1));
        r.insert(RegisterKey::v(vec![Key::Char('w')]), MoveWordForward(1));
        r.insert(RegisterKey::v(vec![Key::Char('b')]), MoveWordBackward(1));

        // Move cursor to start/first/last position
        r.insert(RegisterKey::n(vec![Key::Char('0')]), MoveToStart());
        r.insert(RegisterKey::n(vec![Key::Char('_')]), MoveToFirst());
        r.insert(RegisterKey::n(vec![Key::Char('$')]), MoveToEnd());
        r.insert(RegisterKey::v(vec![Key::Char('0')]), MoveToStart());
        r.insert(RegisterKey::v(vec![Key::Char('_')]), MoveToFirst());
        r.insert(RegisterKey::v(vec![Key::Char('$')]), MoveToEnd());

        // Move cursor to start/first/last position and enter insert mode
        r.insert(
            RegisterKey::n(vec![Key::Char('I')]),
            Composed::new(MoveToFirst()).chain(SwitchMode(EditorMode::Insert)),
        );
        r.insert(
            RegisterKey::n(vec![Key::Char('A')]),
            Composed::new(MoveToEnd()).chain(Append),
        );

        // Append/insert new line and switch into insert mode
        r.insert(
            RegisterKey::n(vec![Key::Char('o')]),
            Composed::new(AppendNewline(1)).chain(SwitchMode(EditorMode::Insert)),
        );
        r.insert(
            RegisterKey::n(vec![Key::Char('O')]),
            Composed::new(InsertNewline(1)).chain(SwitchMode(EditorMode::Insert)),
        );

        // Insert a line break
        r.insert(RegisterKey::i(vec![Key::Enter]), LineBreak(1));

        // Remove the current character
        r.insert(RegisterKey::n(vec![Key::Char('x')]), RemoveChar(1));

        // Delete the previous character
        r.insert(RegisterKey::i(vec![Key::Backspace]), DeleteChar(1));

        // Delete the current line
        r.insert(
            RegisterKey::n(vec![Key::Char('d'), Key::Char('d')]),
            DeleteLine(1),
        );

        // Delete the current selection
        r.insert(RegisterKey::v(vec![Key::Char('d')]), DeleteSelection);

        // Select inner word between delimiters
        r.insert(
            RegisterKey::n(vec![Key::Char('c'), Key::Char('i'), Key::Char('w')]),
            SelectBetween('"'),
        );

        // Select  the line
        r.insert(RegisterKey::n(vec![Key::Char('V')]), SelectLine);

        // Undo
        r.insert(RegisterKey::n(vec![Key::Char('u')]), Undo);

        // Redo
        r.insert(RegisterKey::n(vec![Key::Ctrl('r')]), Redo);

        // Copy
        r.insert(RegisterKey::v(vec![Key::Char('y')]), CopySelection);

        // Paste
        r.insert(RegisterKey::n(vec![Key::Char('p')]), Paste);
        r.insert(RegisterKey::v(vec![Key::Char('p')]), Paste);

        Self { register: r }
    }
}

impl EditorInput {
    pub fn on_key<T>(&mut self, key: T, state: &mut EditorState)
    where
        T: Into<Key> + Copy,
    {
        let mode = state.mode;

        match key.into() {
            // Always insert characters in insert mode
            Key::Char(c) if mode == EditorMode::Insert => InsertChar(c).execute(state),
            // Always add characters to search in search mode
            Key::Char(c) if mode == EditorMode::Search => AppendCharToSearch(c).execute(state),
            // Else lookup an action from the register
            _ => {
                if let Some(mut action) = self.register.get(key.into(), mode) {
                    action.execute(state);
                }
            }
        }
    }
}

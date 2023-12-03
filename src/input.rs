pub mod key;
pub mod register;

use crate::actions::insert::Insert;
use crate::actions::{
    Append, Delete, DeleteChar, DeleteLine, DeleteSelection, Execute, InsertChar, InsertNewline,
    Move, MoveBackward, MoveDown, MoveForward, MoveUp, Redo, Remove, Select, SelectBetween,
    SwitchMode, Undo,
};
use crate::{EditorMode, EditorState};

use self::register::RegisterKey;
use self::{key::Key, register::Register};

#[derive(Clone, Debug)]
pub struct Input {
    register: Register,
}

impl Default for Input {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        let mut register = Register::new();

        // Go into normal mode
        register.insert(
            RegisterKey::i(vec![Key::Esc]),
            SwitchMode(EditorMode::Normal),
        );
        register.insert(
            RegisterKey::v(vec![Key::Esc]),
            SwitchMode(EditorMode::Normal),
        );

        // Go into insert mode
        register.insert(
            RegisterKey::n(vec![Key::Char('i')]),
            SwitchMode(EditorMode::Insert),
        );

        // Go into visual mode
        register.insert(
            RegisterKey::n(vec![Key::Char('v')]),
            SwitchMode(EditorMode::Visual),
        );

        // Go into insert mode and move one char forward
        register.insert(RegisterKey::n(vec![Key::Char('a')]), Append);

        // Move cursor right
        let action = Move::Forward(MoveForward(1));
        register.insert(RegisterKey::n(vec![Key::Char('l')]), action);
        register.insert(RegisterKey::v(vec![Key::Char('l')]), action);
        register.insert(RegisterKey::i(vec![Key::Right]), action);

        // Move cursor left
        let action = Move::Backward(MoveBackward(1));
        register.insert(RegisterKey::n(vec![Key::Char('h')]), action);
        register.insert(RegisterKey::v(vec![Key::Char('h')]), action);
        register.insert(RegisterKey::i(vec![Key::Left]), action);

        // Move cursor up
        let action = Move::Up(MoveUp(1));
        register.insert(RegisterKey::n(vec![Key::Char('k')]), action);
        register.insert(RegisterKey::v(vec![Key::Char('k')]), action);
        register.insert(RegisterKey::i(vec![Key::Up]), action);

        // Move cursor down
        let action = Move::Down(MoveDown(1));
        register.insert(RegisterKey::n(vec![Key::Char('j')]), action);
        register.insert(RegisterKey::v(vec![Key::Char('j')]), action);
        register.insert(RegisterKey::i(vec![Key::Down]), action);

        // Insert new line
        register.insert(
            RegisterKey::i(vec![Key::Enter]),
            Insert::Newline(InsertNewline(1)),
        );

        // Remove the current character
        register.insert(
            RegisterKey::n(vec![Key::Char('x')]),
            Delete::Remove(Remove(1)),
        );

        // Delete the previous character
        register.insert(
            RegisterKey::i(vec![Key::Backspace]),
            Delete::Char(DeleteChar(1)),
        );

        // Delete the current line
        register.insert(
            RegisterKey::n(vec![Key::Char('d'), Key::Char('d')]),
            Delete::Line(DeleteLine(1)),
        );

        // Delete the current selection
        register.insert(
            RegisterKey::v(vec![Key::Char('d')]),
            Delete::Selection(DeleteSelection),
        );

        // Select inner word between delimiters
        register.insert(
            RegisterKey::n(vec![Key::Char('c'), Key::Char('w')]),
            Select::Between(SelectBetween('"')),
        );

        // Undo
        register.insert(RegisterKey::n(vec![Key::Char('u')]), Undo);

        // Redo
        register.insert(RegisterKey::n(vec![Key::Char('r')]), Redo);

        Self { register }
    }
}

// impl Default for Input {
//     #[allow(clippy::too_many_lines)]
//     fn default() -> Self {
//         let mut register = Register::new();
//
//         // Go into normal mode
//         register.insert(
//             RegisterKey::i(vec![Key::Esc]),
//             RegisterVal(EditorState::normal_mode),
//         );
//         register.insert(
//             RegisterKey::v(vec![Key::Esc]),
//             RegisterVal(EditorState::normal_mode),
//         );
//
//         // Go into insert mode
//         register.insert(
//             RegisterKey::n(vec![Key::Char('i')]),
//             RegisterVal(EditorState::insert_mode),
//         );
//
//         // Go into visual mode
//         register.insert(
//             RegisterKey::n(vec![Key::Char('v')]),
//             RegisterVal(EditorState::visual_mode),
//         );
//
//         // Go into insert mode and move one char forward
//         register.insert(
//             RegisterKey::n(vec![Key::Char('a')]),
//             RegisterVal(EditorState::append_mode),
//         );
//
//         // Create a new line below and switch to insert mode
//         register.insert(
//             RegisterKey::n(vec![Key::Char('o')]),
//             RegisterVal(EditorState::new_line_below_and_insert_mode),
//         );
//
//         // Create a new line above and switch to insert mode
//         register.insert(
//             RegisterKey::n(vec![Key::Char('O')]),
//             RegisterVal(EditorState::new_line_above_and_insert_mode),
//         );
//
//         // Move cursor left
//         register.insert(
//             RegisterKey::n(vec![Key::Char('h')]),
//             RegisterVal(EditorState::move_cursor_left),
//         );
//         register.insert(
//             RegisterKey::v(vec![Key::Char('h')]),
//             RegisterVal(EditorState::move_cursor_left),
//         );
//         register.insert(
//             RegisterKey::i(vec![Key::Left]),
//             RegisterVal(EditorState::move_cursor_left),
//         );
//
//         // Move cursor right
//         register.insert(
//             RegisterKey::n(vec![Key::Char('l')]),
//             RegisterVal(EditorState::move_cursor_right),
//         );
//         register.insert(
//             RegisterKey::v(vec![Key::Char('l')]),
//             RegisterVal(EditorState::move_cursor_right),
//         );
//         register.insert(
//             RegisterKey::i(vec![Key::Right]),
//             RegisterVal(EditorState::move_cursor_right),
//         );
//
//         // Move cursor up
//         register.insert(
//             RegisterKey::n(vec![Key::Char('k')]),
//             RegisterVal(EditorState::move_cursor_up),
//         );
//         register.insert(
//             RegisterKey::v(vec![Key::Char('k')]),
//             RegisterVal(EditorState::move_cursor_up),
//         );
//         register.insert(
//             RegisterKey::i(vec![Key::Up]),
//             RegisterVal(EditorState::move_cursor_up),
//         );
//
//         // Move cursor down
//         register.insert(
//             RegisterKey::n(vec![Key::Char('j')]),
//             RegisterVal(EditorState::move_cursor_down),
//         );
//         register.insert(
//             RegisterKey::v(vec![Key::Char('j')]),
//             RegisterVal(EditorState::move_cursor_down),
//         );
//         register.insert(
//             RegisterKey::i(vec![Key::Down]),
//             RegisterVal(EditorState::move_cursor_down),
//         );
//
//         // Insert new line
//         register.insert(
//             RegisterKey::i(vec![Key::Enter]),
//             RegisterVal(EditorState::insert_newline),
//         );
//
//         // Remove the current character
//         register.insert(
//             RegisterKey::n(vec![Key::Char('x')]),
//             RegisterVal(EditorState::remove_char),
//         );
//
//         // Delete the previous character
//         register.insert(
//             RegisterKey::i(vec![Key::Backspace]),
//             RegisterVal(EditorState::delete_char),
//         );
//
//         // Delete the current line
//         register.insert(
//             RegisterKey::n(vec![Key::Char('d'), Key::Char('d')]),
//             RegisterVal(EditorState::delete_line),
//         );
//
//         // Delete the current selection
//         register.insert(
//             RegisterKey::v(vec![Key::Char('d')]),
//             RegisterVal(|b: &mut EditorState| {
//                 b.delete_selection();
//                 b.normal_mode();
//             }),
//         );
//
//         // Select inner word between delimiters
//         register.insert(
//             RegisterKey::n(vec![Key::Char('c'), Key::Char('w')]),
//             RegisterVal(|b: &mut EditorState| {
//                 b.select_between_delimiters(&['"']);
//                 b.mode = EditorMode::Visual;
//             }),
//         );
//
//         // Undo
//         register.insert(
//             RegisterKey::n(vec![Key::Char('u')]),
//             RegisterVal(EditorState::undo),
//         );
//
//         // Redo
//         register.insert(
//             RegisterKey::n(vec![Key::Char('r')]),
//             RegisterVal(EditorState::redo),
//         );
//
//         Self { register }
//     }
// }

impl Input {
    pub fn on_key<T>(&mut self, key: T, state: &mut EditorState)
    where
        T: Into<Key> + Copy,
    {
        let mode = state.mode;

        match key.into() {
            // Always insert characters in insert mode
            Key::Char(c) if mode == EditorMode::Insert => InsertChar(c).execute(state),
            // Else lookup an action from the register
            _ => {
                if let Some(mut action) = self.register.get(key.into(), mode) {
                    action.execute(state);
                }
            }
        }
    }
}

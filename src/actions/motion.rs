use super::Execute;
use crate::{EditorMode, EditorState};

#[derive(Clone, Debug, Copy)]
pub struct MoveForward(pub usize);

impl Execute for MoveForward {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.column >= state.len_col().saturating_sub(1) {
                break;
            }
            state.cursor.column += 1;
        }
        if state.mode == EditorMode::Visual {
            state.set_selection_end(state.cursor);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveBackward(pub usize);

impl Execute for MoveBackward {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.column == 0 {
                break;
            }
            state.cursor.column -= 1;
        }
        if state.mode == EditorMode::Visual {
            state.set_selection_end(state.cursor);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveUp(pub usize);

impl Execute for MoveUp {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.line == 0 {
                break;
            }
            state.cursor.line -= 1;
            state.cursor.column = state.cursor.column.min(state.len_col());
        }
        if state.mode == EditorMode::Visual {
            state.set_selection_end(state.cursor);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveDown(pub usize);

impl Execute for MoveDown {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.line >= state.len().saturating_sub(1) {
                break;
            }
            state.cursor.line += 1;
            state.cursor.column = state.cursor.column.min(state.len_col());
        }
        if state.mode == EditorMode::Visual {
            state.set_selection_end(state.cursor);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{state::position::Position, Lines};

    use super::*;
    fn test_state() -> EditorState {
        let mut state = EditorState::new();
        let mut data = Lines::from("Hello World!\n\n123.");
        state.lines.append(&mut data);
        state
    }

    #[test]
    fn test_move_forward() {
        let mut state = test_state();

        MoveForward(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 1));

        MoveForward(10).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 11));

        MoveForward(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 11));
    }

    #[test]
    fn test_move_backward() {
        let mut state = test_state();
        state.set_cursor_position(0, 11);

        MoveBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 10));

        MoveBackward(10).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 0));

        MoveBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 0));
    }
}

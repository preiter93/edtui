use enum_dispatch::enum_dispatch;

use super::Execute;
use crate::{EditorMode, EditorState};

#[enum_dispatch(Execute)]
#[derive(Clone, Debug, Copy)]
pub enum Move {
    Forward(MoveForward),
    Backward(MoveBackward),
    Up(MoveUp),
    Down(MoveDown),
}

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

use super::Execute;
use crate::EditorState;

#[derive(Clone, Debug, Copy)]
pub struct InsertChar(pub char);

impl Execute for InsertChar {
    fn execute(&mut self, state: &mut EditorState) {
        let ch = self.0;
        if state.lines.is_empty() {
            state.lines.push(Vec::new());
        }
        if ch == '\n' {
            InsertNewline(1).execute(state);
        } else {
            state.lines.insert(state.cursor.as_index(), ch);
            state.cursor.column += 1;
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct InsertNewline(pub usize);

impl Execute for InsertNewline {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.column == 0 {
                state.insert(state.cursor.line, "");
            } else {
                let split_at = state.cursor.as_index();
                let mut rest = state.lines.split_off(split_at);
                state.lines.append(&mut rest);
            }
            state.cursor.line += 1;
            state.cursor.column = 0;
        }
    }
}

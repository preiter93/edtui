use jagged::Index2;

use crate::Lines;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SearchState {
    pub(crate) pattern: String,
    matches: Vec<Index2>,
    start_cursor: Index2,
    selected_index: Option<usize>,
}

impl SearchState {
    pub(crate) fn pattern_len(&self) -> usize {
        self.pattern.len()
    }

    pub(crate) fn push_char(&mut self, ch: char) {
        self.pattern.push(ch);
    }

    pub(crate) fn trigger_search(&mut self, lines: &Lines) {
        let pattern: Vec<char> = self.pattern.chars().collect();
        self.matches = lines
            .match_indices(&pattern)
            .map(|(_, index)| index)
            .collect();
    }

    pub(crate) fn remove_char(&mut self) {
        self.pattern.pop();
    }

    pub(crate) fn clear(&mut self) {
        self.pattern.clear();
        self.matches.clear();
    }

    pub(crate) fn find_first(&mut self) -> Option<&Index2> {
        for (i, index) in self.matches.iter().enumerate() {
            if index >= &self.start_cursor {
                self.selected_index = Some(i);
                return Some(index);
            }
        }
        match self.matches.first() {
            Some(index) => {
                self.selected_index = Some(0);
                Some(index)
            }
            None => None,
        }
    }

    pub(crate) fn find_next(&mut self) -> Option<&Index2> {
        if let Some(selected) = self.selected_index {
            let new_selected = if selected + 1 >= self.matches.len() {
                0
            } else {
                selected + 1
            };
            self.selected_index = Some(new_selected);
            return self.matches.get(new_selected);
        }
        None
    }

    pub(crate) fn find_previous(&mut self) -> Option<&Index2> {
        if let Some(selected) = self.selected_index {
            let new_selected = if selected == 0 {
                self.matches.len().saturating_sub(0)
            } else {
                selected - 1
            };
            self.selected_index = Some(new_selected);
            return self.matches.get(new_selected);
        }
        None
    }
}

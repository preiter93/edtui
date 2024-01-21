use jagged::Index2;

use crate::Lines;

/// Represents the state of a search operation, including the search pattern,
/// matched indices, and selected index.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SearchState {
    pub(crate) start_cursor: Index2,
    pub(crate) pattern: String,
    matches: Vec<Index2>,
    selected_index: Option<usize>,
}

impl SearchState {
    /// Returns the length of the current search pattern.
    pub(crate) fn pattern_len(&self) -> usize {
        self.pattern.len()
    }

    /// Starts a search by setting the start index and clearing all previous state.
    pub(crate) fn start(&mut self, start_cursor: Index2) {
        self.clear();
        self.start_cursor = start_cursor;
    }

    /// Clears both the search pattern and matched indices.
    pub(crate) fn clear(&mut self) {
        self.pattern.clear();
        self.matches.clear();
    }

    /// Triggers a search based on the current pattern in the provided text.
    pub(crate) fn trigger_search(&mut self, lines: &Lines) {
        let pattern: Vec<char> = self.pattern.chars().collect();
        self.matches = lines
            .match_indices(&pattern)
            .map(|(_, index)| index)
            .collect();
    }

    /// Appends a character to the search pattern.
    pub(crate) fn push_char(&mut self, ch: char) {
        self.pattern.push(ch);
    }

    /// Removes the last character from the search pattern.
    pub(crate) fn remove_char(&mut self) {
        self.pattern.pop();
    }

    /// Finds and returns the next matched index after the selected index.
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

    /// Finds and returns the previous matched index before the selected index.
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

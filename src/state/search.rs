use jagged::Index2;

use crate::Lines;

use super::selection::Selection;

/// Represents the state of a search operation
/// Including the search pattern, matched indices and selected index.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SearchState {
    pub(crate) start_cursor: Index2,
    pub(crate) pattern: String,
    pub(crate) matches: Vec<Index2>,
    pub(crate) selected_index: Option<usize>,
}

impl SearchState {
    pub(crate) fn len(&self) -> usize {
        self.pattern.len()
    }

    pub(crate) fn start(&mut self, start_cursor: Index2) {
        self.clear();
        self.start_cursor = start_cursor;
    }

    pub(crate) fn clear(&mut self) {
        self.pattern.clear();
        self.matches.clear();
    }

    pub(crate) fn trigger_search(&mut self, lines: &Lines) {
        let pattern: Vec<char> = self.pattern.chars().collect();
        self.matches = lines
            .match_indices(&pattern)
            .map(|(_, index)| index)
            .collect();
    }

    pub(crate) fn push_char(&mut self, ch: char) {
        self.pattern.push(ch);
    }

    pub(crate) fn remove_char(&mut self) {
        self.pattern.pop();
    }

    pub(crate) fn first(&mut self) -> Option<&Index2> {
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

    pub(crate) fn current(&self) -> Option<&Index2> {
        self.selected_index.and_then(|i| self.matches.get(i))
    }

    pub(crate) fn next(&mut self) -> Option<&Index2> {
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

    pub(crate) fn previous(&mut self) -> Option<&Index2> {
        let len = self.matches.len();
        if len == 0 {
            return None;
        }

        let new_selected = match self.selected_index {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };

        self.selected_index = Some(new_selected);
        self.matches.get(new_selected)
    }
}

impl From<&SearchState> for Option<Selection> {
    fn from(value: &SearchState) -> Self {
        value
            .selected_index
            .and_then(|index| value.matches.get(index))
            .map(|&start| {
                let end = Index2::new(start.row, start.col + value.len().saturating_sub(1));
                Selection::new(start, end)
            })
    }
}
